use crate::net;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::{fs, path::Path, process::Command};
const API: &str = "https://api.github.com/repos/slarenc/kyro-launcher/releases/latest";
fn version(v: &str) -> Option<(u64, u64, u64)> {
    let mut parts = v.trim_start_matches('v').split('.');
    let result = (
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    );
    if parts.next().is_some() {
        None
    } else {
        Some(result)
    }
}
fn release_info(release: &Value) -> Result<Value, String> {
    let tag = release["tag_name"].as_str().ok_or("В релизе нет версии")?;
    let available = version(tag).ok_or("Неподдерживаемый номер релиза")?
        > version(env!("CARGO_PKG_VERSION")).ok_or("Неверная версия KYRO")?;
    let asset = release["assets"].as_array().and_then(|a| {
        a.iter().find(|a| {
            a["name"]
                .as_str()
                .is_some_and(|n| n.to_lowercase().ends_with(".exe"))
        })
    });
    let (url, digest) = asset
        .map(|a| {
            (
                a["browser_download_url"].as_str().unwrap_or(""),
                a["digest"].as_str().unwrap_or(""),
            )
        })
        .unwrap_or(("", ""));
    let installable = available
        && url.starts_with("https://github.com/slarenc/kyro-launcher/releases/download/")
        && digest
            .strip_prefix("sha256:")
            .is_some_and(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()));
    Ok(
        json!({"current":env!("CARGO_PKG_VERSION"),"version":tag,"available":available,"installable":installable,"notes":release["body"],"url":url,"digest":digest,"page":release["html_url"]}),
    )
}

pub async fn check_launcher_update() -> Result<Value, String> {
    release_info(&net::json(API).await?)
}

pub fn notify_release(data: &mut crate::store::Data, info: &Value) -> bool {
    let Some(tag) = info["version"].as_str() else { return false; };
    if info["available"] != true || data.notified_launcher_version == tag {
        return false;
    }
    data.notified_launcher_version = tag.into();
    data.notices.insert(0, crate::store::Notice {
        id: format!("launcher-update-{tag}"),
        title: format!("Доступна новая версия KYRO {tag}"),
        body: "На GitHub вышло обновление лаунчера. Открой «Настройки» → «Обновления KYRO», чтобы обновиться.".into(),
        time: chrono::Utc::now().to_rfc3339(),
        unread: true,
    });
    data.notices.truncate(200);
    true
}

pub async fn install_launcher_update(app: crate::app_context::AppContext) -> Result<(), String> {
    let state = app.state();
    {
        let mut busy = state.busy.lock().map_err(|e| e.to_string())?;
        if *busy {
            return Err("Закройте Minecraft и дождитесь окончания загрузок".into());
        }
        *busy = true;
    }
    let result = async {
        let info = check_launcher_update().await?;
        if info["installable"] != true {
            return Err("Нет более новой версии с проверяемым EXE".into());
        }
        let updates = state.root.join("updates");
        fs::create_dir_all(&updates).map_err(|e| e.to_string())?;
        let source = updates.join("downloaded-update.exe");
        let hash = info["digest"]
            .as_str()
            .and_then(|s| s.strip_prefix("sha256:"))
            .ok_or("Нет SHA-256 релиза")?;
        crate::game::progress(&app, "Загрузка обновления KYRO", 0.2);
        net::download_sha256(info["url"].as_str().ok_or("Нет URL")?, &source, hash).await?;
        let target = std::env::current_exe().map_err(|e| e.to_string())?;
        let helper = updates.join(format!(
            "kyro-update-helper-{}.exe",
            chrono::Utc::now().timestamp_micros()
        ));
        fs::copy(&target, &helper).map_err(|e| e.to_string())?;
        let plan = updates.join("update-plan.json");
        fs::write(
            &plan,
            serde_json::to_vec(
                &json!({"target":target,"source":source,"hash":hash,"data":state.root}),
            )
            .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let mut cmd = Command::new(helper);
        cmd.arg("--apply-update").arg(plan);
        #[cfg(windows)]
        cmd.creation_flags(0x08000000);
        cmd.spawn().map_err(|e| e.to_string())?;
        Ok(())
    }
    .await;
    *state.busy.lock().map_err(|e| e.to_string())? = false;
    if result.is_ok() {
        app.exit(0);
    }
    result
}
fn apply(plan: &Value) -> Result<(), String> {
    apply_files(plan, true)
}
fn apply_files(plan: &Value, relaunch: bool) -> Result<(), String> {
    let target = Path::new(plan["target"].as_str().ok_or("Нет target")?);
    let source = Path::new(plan["source"].as_str().ok_or("Нет source")?);
    let hash = plan["hash"].as_str().ok_or("Нет hash")?;
    if !target.is_absolute()
        || target
            .extension()
            .is_none_or(|e| !e.to_string_lossy().eq_ignore_ascii_case("exe"))
    {
        return Err("Неверная цель обновления".into());
    }
    let bytes = fs::read(source).map_err(|e| e.to_string())?;
    if !format!("{:x}", Sha256::digest(&bytes)).eq_ignore_ascii_case(hash) {
        return Err("Файл обновления повреждён".into());
    }
    let new = target.with_extension("update-new");
    fs::write(&new, bytes).map_err(|e| e.to_string())?;
    let backup = target.with_extension(format!(
        "previous-{}.exe",
        chrono::Utc::now().timestamp_micros()
    ));
    let mut moved = false;
    for _ in 0..120 {
        if fs::rename(target, &backup).is_ok() {
            moved = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    if !moved {
        return Err("Не удалось заменить EXE: закройте другие окна KYRO".into());
    }
    if let Err(e) = fs::rename(&new, target) {
        let _ = fs::rename(&backup, target);
        return Err(e.to_string());
    }
    if !relaunch {
        return Ok(());
    }
    let mut cmd = Command::new(target);
    if let Some(parent) = target.parent() {
        cmd.current_dir(parent);
    }
    if let Some(root) = plan["data"].as_str() {
        cmd.env("KYRO_DATA_DIR", root);
    }
    if let Err(e) = cmd.spawn() {
        let failed = target.with_extension("failed-update.exe");
        let _ = fs::rename(target, failed);
        let _ = fs::rename(&backup, target);
        return Err(format!(
            "Не удалось перезапустить KYRO; прежний EXE восстановлен: {e}"
        ));
    }
    Ok(())
}
pub fn handle_cli() -> bool {
    let args: Vec<_> = std::env::args_os().collect();
    if args.get(1).is_none_or(|s| s != "--apply-update") {
        return false;
    }
    if let Some(path) = args.get(2) {
        let result = fs::read(path)
            .map_err(|e| e.to_string())
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).map_err(|e| e.to_string()))
            .and_then(|plan| apply(&plan));
        if let Err(e) = result {
            let log = Path::new(path).with_file_name("update-error.log");
            let _ = fs::write(log, e);
        }
    }
    true
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn notifies_once_per_release_even_after_clearing_and_reloading() {
        let mut data = crate::store::Data::default();
        assert!(!notify_release(&mut data, &json!({"available":false,"version":"0.0.1"})));
        let info = json!({"available":true,"version":"v99.0.0"});
        assert!(notify_release(&mut data, &info));
        assert!(data.notices[0].unread);
        assert!(!notify_release(&mut data, &info));
        data.notices.clear();
        let mut restored = serde_json::from_value(serde_json::to_value(data).unwrap()).unwrap();
        assert!(!notify_release(&mut restored, &info));
        assert!(notify_release(&mut restored, &json!({"available":true,"version":"v99.0.1"})));
        assert_eq!(restored.notices.len(), 1);
    }
    #[test]
    fn checks_release_version_and_origin() {
        assert!(version("v0.10.0") > version("0.6.0"));
        let info=release_info(&json!({"tag_name":"v99.0.0","assets":[{"name":"KYRO.exe","browser_download_url":"https://evil.test/app.exe","digest":format!("sha256:{}","a".repeat(64))}]})).unwrap();
        assert_eq!(info["installable"], false);
    }
    #[test]
    fn replaces_only_executable_and_preserves_backup_and_data() {
        let root = std::env::temp_dir().join(format!(
            "kyro-update-test-{}",
            chrono::Utc::now().timestamp_micros()
        ));
        fs::create_dir_all(&root).unwrap();
        let target = root.join("KYRO.exe");
        let source = root.join("download.exe");
        let data = root.join("state.json");
        fs::write(&target, b"old executable").unwrap();
        fs::write(&source, b"new executable").unwrap();
        fs::write(&data, b"private settings").unwrap();
        let mut plan = json!({"target":target,"source":source,"hash":"bad"});
        assert!(apply_files(&plan, false).is_err());
        assert_eq!(fs::read(&target).unwrap(), b"old executable");
        plan["hash"] = json!(format!("{:x}", Sha256::digest(b"new executable")));
        apply_files(&plan, false).unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"new executable");
        assert_eq!(fs::read(&data).unwrap(), b"private settings");
        let backup = fs::read_dir(&root)
            .unwrap()
            .flatten()
            .find(|e| e.file_name().to_string_lossy().contains("previous-"))
            .unwrap();
        assert_eq!(fs::read(backup.path()).unwrap(), b"old executable");
        let _ = fs::remove_dir_all(root);
    }
}
