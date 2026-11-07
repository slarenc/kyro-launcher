use crate::{
    game, net,
    store::{safe_segment, Modpack, State},
};
use futures_util::{stream, StreamExt, TryStreamExt};
use serde::Deserialize;
use sha1::{Digest, Sha1};
use sha2::Sha512;
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Index {
    format_version: u32,
    game: String,
    dependencies: HashMap<String, String>,
    files: Vec<PackFile>,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackFile {
    path: String,
    hashes: HashMap<String, String>,
    #[serde(default)]
    env: HashMap<String, String>,
    downloads: Vec<String>,
    file_size: u64,
}
const MAX_FILE: u64 = 1024 * 1024 * 1024;
const MAX_TOTAL: u64 = 8 * MAX_FILE;

fn relative(path: &str) -> Result<PathBuf, String> {
    // Windows also interprets backslashes, ADS colons and trailing dots/spaces.
    if path.is_empty() || path.contains(['\\', ':', '\0']) || path.starts_with('/') {
        return Err(format!("Небезопасный путь сборки: {path}"));
    }
    for part in path.split('/') {
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        if part.is_empty()
            || part == "."
            || part == ".."
            || part.ends_with(['.', ' '])
            || part
                .chars()
                .any(|c| c.is_control() || "<>\"|?*".contains(c))
            || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || ((stem.starts_with("COM") || stem.starts_with("LPT"))
                && stem.len() == 4
                && stem.ends_with(|c: char| c.is_ascii_digit()))
        {
            return Err(format!("Небезопасный путь сборки: {path}"));
        }
    }
    Ok(PathBuf::from(path))
}
fn allowed_url(url: &str) -> bool {
    reqwest::Url::parse(url).is_ok_and(|u| {
        u.scheme() == "https"
            && u.username().is_empty()
            && u.password().is_none()
            && u.port().is_none_or(|p| p == 443)
            && matches!(
                u.host_str(),
                Some(
                    "cdn.modrinth.com" | "github.com" | "raw.githubusercontent.com" | "gitlab.com"
                )
            )
    })
}
fn dependencies(index: &Index) -> Result<(String, String, String), String> {
    if index.format_version != 1 || index.game != "minecraft" {
        return Err("Неподдерживаемый формат сборки".into());
    }
    for key in index.dependencies.keys() {
        if ![
            "minecraft",
            "fabric-loader",
            "quilt-loader",
            "forge",
            "neoforge",
        ]
        .contains(&key.as_str())
        {
            return Err(format!("Ядро или зависимость {key} пока не поддерживается. Доступны Vanilla, Fabric, Quilt, Forge и NeoForge."));
        }
    }
    let game = index
        .dependencies
        .get("minecraft")
        .ok_or("В сборке нет версии Minecraft")?
        .clone();
    safe_segment(&game)?;
    let loaders: Vec<_> = [
        ("fabric-loader", "Fabric"),
        ("quilt-loader", "Quilt"),
        ("forge", "Forge"),
        ("neoforge", "NeoForge"),
    ]
    .into_iter()
    .filter_map(|(key, name)| index.dependencies.get(key).map(|v| (name, v)))
    .collect();
    if loaders.len() > 1 {
        return Err("В сборке указано несколько ядер".into());
    }
    let (loader, pinned) = loaders
        .first()
        .map(|(l, v)| (l.to_string(), v.to_string()))
        .unwrap_or(("Vanilla".into(), String::new()));
    if !pinned.is_empty() {
        safe_segment(&pinned)?;
    }
    Ok((game, loader, pinned))
}
fn read_pack(bytes: &[u8], stage: &Path) -> Result<Index, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    if archive.len() > 30000 {
        return Err("Слишком много файлов в архиве".into());
    }
    let index: Index = {
        let mut file = archive
            .by_name("modrinth.index.json")
            .map_err(|_| "В архиве нет modrinth.index.json")?;
        if file.size() > 16 * 1024 * 1024 {
            return Err("Манифест слишком большой".into());
        }
        let mut json = Vec::new();
        file.read_to_end(&mut json).map_err(|e| e.to_string())?;
        serde_json::from_slice(&json).map_err(|e| format!("Некорректный манифест сборки: {e}"))?
    };
    dependencies(&index)?;
    let mut total = 0u64;
    let mut paths = HashSet::new();
    for file in &index.files {
        relative(&file.path)?;
        if !paths.insert(file.path.to_lowercase()) {
            return Err("Повторяющийся путь в сборке".into());
        }
        if file.file_size > MAX_FILE {
            return Err("Слишком большой файл сборки".into());
        }
        total = total
            .checked_add(file.file_size)
            .ok_or("Сборка слишком большая")?;
        for (key, len) in [("sha1", 40), ("sha512", 128)] {
            if !file
                .hashes
                .get(key)
                .is_some_and(|h| h.len() == len && h.bytes().all(|b| b.is_ascii_hexdigit()))
            {
                return Err(format!("Нет корректной суммы {key}: {}", file.path));
            }
        }
        if file
            .env
            .get("client")
            .is_some_and(|e| !["required", "optional", "unsupported"].contains(&e.as_str()))
        {
            return Err("Неизвестное клиентское окружение".into());
        }
        if !file.downloads.iter().any(|url| allowed_url(url))
            && file.env.get("client").map(String::as_str) != Some("unsupported")
        {
            return Err(format!("Нет разрешённого HTTPS-адреса: {}", file.path));
        }
    }
    // Client overrides deliberately win over shared overrides. Server files are excluded.
    for prefix in ["overrides/", "client-overrides/"] {
        let mut layer = HashSet::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let Some(name) = file.name().strip_prefix(prefix) else {
                continue;
            };
            if file.is_dir() {
                continue;
            }
            let path = relative(name)?;
            if !layer.insert(name.to_lowercase()) {
                return Err("Повторяющийся override в архиве".into());
            }
            if file.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) {
                return Err("Ссылки в архиве не поддерживаются".into());
            }
            total = total
                .checked_add(file.size())
                .ok_or("Сборка слишком большая")?;
            if file.size() > MAX_FILE || total > MAX_TOTAL {
                return Err("Сборка превышает допустимый размер".into());
            }
            let dest = stage.join(path);
            fs::create_dir_all(dest.parent().ok_or("Нет папки")?).map_err(|e| e.to_string())?;
            let mut out = fs::File::create(dest).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut out).map_err(|e| e.to_string())?;
        }
    }
    if total > MAX_TOTAL {
        return Err("Сборка превышает допустимый размер".into());
    }
    Ok(index)
}
fn verify(file: &PackFile, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() as u64 != file.file_size
        || !format!("{:x}", Sha1::digest(bytes)).eq_ignore_ascii_case(&file.hashes["sha1"])
        || !format!("{:x}", Sha512::digest(bytes)).eq_ignore_ascii_case(&file.hashes["sha512"])
    {
        return Err(format!(
            "Размер или контрольная сумма не совпали: {}",
            file.path
        ));
    }
    Ok(())
}
async fn fetch(file: &PackFile) -> Result<Vec<u8>, String> {
    let mut last = "Нет доступного зеркала".to_string();
    for url in file.downloads.iter().filter(|u| allowed_url(u)) {
        let attempt = async {
            let mut response = net::client()?
                .get(url)
                .send()
                .await
                .map_err(|e| e.to_string())?
                .error_for_status()
                .map_err(|e| e.to_string())?;
            if response.url().scheme() != "https" {
                return Err("Зеркало перенаправило на HTTP".into());
            }
            let mut bytes = Vec::new();
            while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
                if (bytes.len() + chunk.len()) as u64 > file.file_size {
                    return Err("Размер загрузки превышает манифест".into());
                }
                bytes.extend_from_slice(&chunk);
            }
            verify(file, &bytes)?;
            Ok(bytes)
        }
        .await;
        match attempt {
            Ok(bytes) => return Ok(bytes),
            Err(e) => last = e,
        }
    }
    Err(format!("{}: {last}", file.path))
}
struct Staging(PathBuf);
impl Drop for Staging {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub async fn install(
    app: &crate::app_context::AppContext,
    project: &str,
    name: &str,
) -> Result<(), String> {
    let state = app.state();
    let mut settings = state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .settings
        .clone();
    game::progress(app, "Получение версии сборки Modrinth", 0.02);
    let version = super::compatible(project, "modpack", &settings).await?;
    let version_id = version["id"].as_str().ok_or("Нет ID версии сборки")?;
    safe_segment(version_id)?;
    safe_segment(project)?;
    let id = format!("{project}-{version_id}");
    if state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .modpacks
        .iter()
        .any(|p| p.id == id)
    {
        return activate(&state, &id);
    }
    let file = version["files"]
        .as_array()
        .and_then(|a| {
            a.iter()
                .find(|f| {
                    f["primary"] == true
                        && f["filename"]
                            .as_str()
                            .is_some_and(|s| s.ends_with(".mrpack"))
                })
                .or_else(|| {
                    a.iter().find(|f| {
                        f["filename"]
                            .as_str()
                            .is_some_and(|s| s.ends_with(".mrpack"))
                    })
                })
        })
        .ok_or("У сборки нет файла .mrpack")?;
    let packfile: PackFile = serde_json::from_value(serde_json::json!({"path":"pack.mrpack","hashes":file["hashes"],"downloads":[file["url"]],"fileSize":file["size"]})).map_err(|e| e.to_string())?;
    if packfile.file_size > MAX_FILE
        || !packfile.hashes.contains_key("sha1")
        || !packfile.hashes.contains_key("sha512")
    {
        return Err("Некорректный архив сборки".into());
    }
    let bytes = fetch(&packfile).await?;
    let packs_root = state.root.join("instances/modpacks");
    fs::create_dir_all(&packs_root).map_err(|e| e.to_string())?;
    let stage = Staging(packs_root.join(format!(
        ".staging-{}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    )));
    fs::create_dir(&stage.0).map_err(|e| e.to_string())?;
    let index = read_pack(&bytes, &stage.0)?;
    let (game_version, loader, pinned) = dependencies(&index)?;
    if game_version != settings.version || loader != settings.loader {
        return Err("Версия Minecraft или ядро архива не совпали с выбранным фильтром".into());
    }
    // Move overrides aside: index downloads must be applied first, overrides last.
    let overrides = stage.0.join(".kyro-overrides");
    if overrides.exists() {
        return Err("Зарезервированный путь в сборке".into());
    }
    fs::create_dir(&overrides).map_err(|e| e.to_string())?;
    let entries: Vec<_> = fs::read_dir(&stage.0)
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    for entry in entries {
        if entry.path() != overrides {
            fs::rename(entry.path(), overrides.join(entry.file_name()))
                .map_err(|e| e.to_string())?;
        }
    }
    let files: Vec<_> = index
        .files
        .into_iter()
        .filter(|f| f.env.get("client").map(String::as_str) != Some("unsupported"))
        .collect();
    let total = files.len().max(1);
    let done = std::sync::atomic::AtomicUsize::new(0);
    let stage_path = &stage.0;
    let counter = &done;
    stream::iter(files.into_iter().map(|file| async move {
        if file
            .path
            .split('/')
            .next()
            .is_some_and(|p| p.eq_ignore_ascii_case(".kyro-overrides"))
        {
            return Err("Зарезервированный путь в сборке".into());
        }
        let dest = stage_path.join(relative(&file.path)?);
        let bytes = fetch(&file).await?;
        tokio::fs::create_dir_all(dest.parent().ok_or("Нет папки")?)
            .await
            .map_err(|e| e.to_string())?;
        tokio::fs::write(dest, bytes)
            .await
            .map_err(|e| e.to_string())?;
        let count = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        game::progress(
            app,
            &format!("Файлы сборки: {count} / {total}"),
            0.05 + 0.5 * count as f64 / total as f64,
        );
        Ok::<(), String>(())
    }))
    .buffer_unordered(settings.download_threads.clamp(1, 16))
    .try_collect::<Vec<_>>()
    .await?;
    copy_overrides(&overrides, &stage.0)?;
    fs::remove_dir_all(&overrides).map_err(|e| e.to_string())?;
    settings.version = game_version.clone();
    settings.loader = loader.clone();
    settings.loader_version = pinned.clone();
    settings.instance_id = id.clone();
    game::progress(app, "Подготовка Minecraft и точной версии ядра сборки", 0.6);
    game::prepare(app, &settings).await?;
    let pack = Modpack {
        cover_image: String::new(),
        id: id.clone(),
        project_id: project.into(),
        version_id: version_id.into(),
        name: name.into(),
        version_name: version["version_number"]
            .as_str()
            .unwrap_or(version_id)
            .into(),
        game_version,
        loader,
        loader_version: pinned,
    };
    let dest = state.instance_dir(&settings)?;
    if dest.exists() {
        return Err("Папка сборки уже существует. Данные не перезаписаны.".into());
    }
    fs::rename(&stage.0, &dest).map_err(|e| e.to_string())?;
    {
        let mut data = state.data.lock().map_err(|e| e.to_string())?;
        data.modpacks.push(pack);
        data.settings = settings;
    }
    state.save()?;
    state.notify(
        "Сборка установлена",
        &format!("{name} готова к запуску. Профиль сборки выбран."),
    );
    Ok(())
}
fn copy_overrides(source: &Path, dest: &Path) -> Result<(), String> {
    for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let target = dest.join(entry.file_name());
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            copy_overrides(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
fn activate(state: &State, id: &str) -> Result<(), String> {
    let mut data = state.data.lock().map_err(|e| e.to_string())?;
    if id.is_empty() {
        data.settings.instance_id.clear();
        data.settings.loader_version.clear();
    } else {
        let pack = data
            .modpacks
            .iter()
            .find(|p| p.id == id)
            .cloned()
            .ok_or("Сборка не найдена")?;
        if !state
            .root
            .join("instances/modpacks")
            .join(safe_segment(&pack.id)?)
            .is_dir()
        {
            return Err("Папка сборки не найдена".into());
        }
        data.settings.version = pack.game_version;
        data.settings.loader = pack.loader;
        data.settings.loader_version = pack.loader_version;
        data.settings.instance_id = pack.id;
    }
    drop(data);
    state.save()
}

pub fn select_modpack(state: &State, id: String) -> Result<(), String> {
    let busy = state.busy.lock().map_err(|e| e.to_string())?;
    if *busy {
        return Err("Дождитесь завершения текущей операции".into());
    }
    activate(&state, &id)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_windows_path_escapes() {
        for path in [
            "../x",
            "/x",
            "C:/x",
            "mods\\..\\x",
            "mods/x:ads",
            "mods/CON.jar",
            "mods/x.",
            "mods//x",
            "mods/../x",
        ] {
            assert!(relative(path).is_err(), "{path}");
        }
        assert!(relative("mods/Some Mod+1.0.jar").is_ok());
        assert!(!allowed_url("https://cdn.modrinth.com.evil.org/a"));
        assert!(!allowed_url("http://cdn.modrinth.com/a"));
    }
    #[test]
    fn pins_loader_and_rejects_unsupported_dependencies() {
        let mut index: Index = serde_json::from_value(serde_json::json!({"formatVersion":1,"game":"minecraft","dependencies":{"minecraft":"1.21.1","fabric-loader":"0.16.9"},"files":[]})).unwrap();
        assert_eq!(
            dependencies(&index).unwrap(),
            ("1.21.1".into(), "Fabric".into(), "0.16.9".into())
        );
        index.dependencies.insert("forge".into(), "1".into());
        assert!(dependencies(&index).is_err());
    }
    #[test]
    fn verifies_both_hashes_and_size() {
        let bytes = b"real mod bytes";
        let mut f = PackFile {
            path: "mods/x.jar".into(),
            hashes: HashMap::from([
                ("sha1".into(), format!("{:x}", Sha1::digest(bytes))),
                ("sha512".into(), format!("{:x}", Sha512::digest(bytes))),
            ]),
            env: HashMap::new(),
            downloads: vec![],
            file_size: bytes.len() as u64,
        };
        verify(&f, bytes).unwrap();
        assert!(verify(&f, b"corrupted").is_err());
        f.hashes.insert("sha512".into(), "0".repeat(128));
        assert!(verify(&f, bytes).is_err());
    }
    fn archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
        use std::io::Write;
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in entries {
            writer
                .start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }
    #[test]
    fn extracts_client_overrides_and_blocks_archive_traversal() {
        let manifest = br#"{"formatVersion":1,"game":"minecraft","dependencies":{"minecraft":"1.21.1"},"files":[]}"#;
        let root = std::env::temp_dir().join(format!(
            "kyro-mrpack-test-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        fs::create_dir(&root).unwrap();
        let stage = Staging(root);
        let bytes = archive(&[
            ("modrinth.index.json", manifest),
            ("overrides/config/a.txt", b"shared"),
            ("client-overrides/config/a.txt", b"client"),
            ("server-overrides/config/server.txt", b"server"),
        ]);
        read_pack(&bytes, &stage.0).unwrap();
        assert_eq!(fs::read(stage.0.join("config/a.txt")).unwrap(), b"client");
        assert!(!stage.0.join("config/server.txt").exists());
        let evil = archive(&[
            ("modrinth.index.json", manifest),
            ("overrides/../../escape.txt", b"bad"),
        ]);
        assert!(read_pack(&evil, &stage.0).is_err());
    }
}
