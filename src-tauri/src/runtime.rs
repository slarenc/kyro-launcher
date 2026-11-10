use crate::{game, net, store::Settings};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub async fn java_major(path: &str) -> Option<u64> {
    let mut cmd = tokio::process::Command::new(path);
    cmd.kill_on_drop(true).arg("-version");
    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    let output = cmd.output().await.ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stderr);
    parse_major(&text)
}
fn parse_major(text: &str) -> Option<u64> {
    let version = text.split('"').nth(1)?;
    let mut parts = version.split('.');
    let first = parts.next()?.parse().ok()?;
    if first == 1 {
        parts.next()?.parse().ok()
    } else {
        Some(first)
    }
}
fn extract(bytes: &[u8], dest: &Path) -> Result<PathBuf, String> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let mut total = 0u64;
    let mut java = None;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
        let path = entry.enclosed_name().ok_or("Небезопасный путь Java")?;
        if entry.name().contains(['\\', ':'])
            || entry.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000)
        {
            return Err("Небезопасный файл Java".into());
        }
        total = total
            .checked_add(entry.size())
            .ok_or("Архив Java слишком большой")?;
        if total > 2 * 1024 * 1024 * 1024 {
            return Err("Архив Java слишком большой".into());
        }
        let target = dest.join(&path);
        if entry.is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            continue;
        }
        fs::create_dir_all(target.parent().ok_or("Нет папки Java")?).map_err(|e| e.to_string())?;
        std::io::copy(
            &mut entry,
            &mut fs::File::create(&target).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        if target.file_name().is_some_and(|n| n == "java.exe")
            && target.parent().is_some_and(|p| p.ends_with("bin"))
        {
            java = Some(path);
        }
    }
    java.ok_or("В архиве Java нет bin/java.exe".into())
}
pub async fn resolve(
    app: &crate::app_context::AppContext,
    settings: &Settings,
    major: u64,
    force: bool,
) -> Result<String, String> {
    if !force {
        if !settings.auto_java {
            return match java_major(&settings.java_path).await {
                Some(v) if v >= major => Ok(settings.java_path.clone()),
                _ => Err(format!(
                    "Нужна Java {major}. Включите автоматическую Java в настройках."
                )),
            };
        }
        if java_major(&settings.java_path).await == Some(major) {
            return Ok(settings.java_path.clone());
        }
    }
    let state = app.state();
    let root = state.root.join("runtimes").join(format!("java-{major}"));
    if let Ok(path) = fs::read_to_string(root.join("java-path.txt")) {
        let java = root.join(path.trim());
        if java_major(&java.to_string_lossy()).await == Some(major) {
            return Ok(java.to_string_lossy().into_owned());
        }
    }
    game::progress(
        app,
        &format!("Автоустановка Java {major}: получение Temurin"),
        0.025,
    );
    let releases=net::json(&format!("https://api.adoptium.net/v3/assets/latest/{major}/hotspot?architecture=x64&image_type=jre&os=windows&vendor=eclipse")).await?;
    let package = &releases
        .as_array()
        .and_then(|a| a.first())
        .ok_or("Нет Java для Windows x64")?["binary"]["package"];
    let url = package["link"]
        .as_str()
        .filter(|s| s.starts_with("https://github.com/adoptium/"))
        .ok_or("Неизвестный источник Java")?;
    let hash = package["checksum"].as_str().ok_or("Нет SHA-256 Java")?;
    let archive = state
        .root
        .join("runtimes")
        .join(format!("temurin-{major}.zip"));
    net::download_sha256(url, &archive, hash).await?;
    let staging = state.root.join("runtimes").join(format!(
        ".java-{major}-{}",
        chrono::Utc::now().timestamp_micros()
    ));
    fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
    let outcome = async {
        let relative = extract(&fs::read(&archive).map_err(|e| e.to_string())?, &staging)?;
        if java_major(&staging.join(&relative).to_string_lossy()).await != Some(major) {
            return Err("Установленная Java не прошла проверку".into());
        }
        fs::write(
            staging.join("java-path.txt"),
            relative.to_string_lossy().as_bytes(),
        )
        .map_err(|e| e.to_string())?;
        if root.exists() {
            fs::rename(
                &root,
                root.with_file_name(format!(
                    "java-{major}-backup-{}",
                    chrono::Utc::now().timestamp_micros()
                )),
            )
            .map_err(|e| e.to_string())?;
        }
        fs::rename(&staging, &root).map_err(|e| e.to_string())?;
        Ok(root.join(relative).to_string_lossy().into_owned())
    }
    .await;
    if staging.exists() {
        let _ = fs::remove_dir_all(staging);
    }
    outcome
}

pub async fn install_java(app: crate::app_context::AppContext) -> Result<String, String> {
    let state = app.state();
    {
        let mut busy = state.busy.lock().map_err(|e| e.to_string())?;
        if *busy {
            return Err("Дождитесь завершения операции".into());
        }
        *busy = true;
    }
    let result = async {
        let settings = state
            .data
            .lock()
            .map_err(|e| e.to_string())?
            .settings
            .clone();
        let (manifest,_) = net::minecraft_manifest(&state.root,Some(&settings.version)).await?;
        let item = manifest["versions"]
            .as_array()
            .and_then(|v| v.iter().find(|v| v["id"] == settings.version))
            .ok_or("Нет версии Minecraft")?;
        let meta = net::minecraft_version(&state.root,item).await?;
        let major = meta["javaVersion"]["majorVersion"].as_u64().unwrap_or(17);
        let path = resolve(&app, &settings, major, true).await?;
        {
            let mut data = state.data.lock().map_err(|e| e.to_string())?;
            data.settings.java_path = path.clone();
            data.settings.auto_java = true;
        }
        state.save()?;
        Ok(path)
    }
    .await;
    *state.busy.lock().map_err(|e| e.to_string())? = false;
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn java_versions() {
        assert_eq!(parse_major("openjdk version \"21.0.13\""), Some(21));
        assert_eq!(parse_major("java version \"1.8.0_402\""), Some(8));
        assert_eq!(parse_major("bad"), None);
    }
}
