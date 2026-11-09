use crate::{
    net,
    store::{safe_segment, Installed, State},
};
use serde_json::{json, Value};
use sha1::{Digest, Sha1};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

fn local_name(path: &Path) -> String {
    let fallback = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .trim_end_matches(".disabled")
        .trim_end_matches(".jar")
        .to_string();
    let Ok(file) = fs::File::open(path) else {
        return fallback;
    };
    let Ok(mut zip) = zip::ZipArchive::new(file) else {
        return fallback;
    };
    let Ok(mut meta) = zip.by_name("fabric.mod.json") else {
        return fallback;
    };
    if meta.size() > 1024 * 1024 {
        return fallback;
    }
    let mut bytes = Vec::new();
    if std::io::Read::read_to_end(&mut meta, &mut bytes).is_err() {
        return fallback;
    }
    serde_json::from_slice::<Value>(&bytes)
        .ok()
        .and_then(|v| v["name"].as_str().map(str::to_string))
        .unwrap_or(fallback)
}
fn local_files(dir: &Path) -> Result<Vec<Value>, String> {
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_file() {
            continue;
        }
        let filename = entry.file_name().to_string_lossy().into_owned();
        if !(filename.ends_with(".jar") || filename.ends_with(".jar.disabled")) {
            continue;
        }
        safe_segment_relaxed(&filename)?;
        let bytes = fs::read(entry.path()).map_err(|e| e.to_string())?;
        files.push(json!({"filename":filename,"name":local_name(&entry.path()),"enabled":!filename.ends_with(".disabled"),"hash":format!("{:x}",Sha1::digest(&bytes)),"size":bytes.len(),"dependencies":[],"update":null}));
    }
    files.sort_by_key(|v| v["name"].as_str().unwrap_or("").to_lowercase());
    Ok(files)
}
fn safe_segment_relaxed(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains(['/', '\\', ':'])
        || name.ends_with([' ', '.'])
        || name.chars().any(|c| c.is_control())
    {
        Err("Недопустимое имя мода".into())
    } else {
        Ok(())
    }
}
pub(crate) async fn identify(files: &[Value]) -> Result<Value, String> {
    net::client()?.post("https://api.modrinth.com/v2/version_files").json(&json!({"hashes":files.iter().map(|v|&v["hash"]).collect::<Vec<_>>(),"algorithm":"sha1"})).send().await.map_err(|e|e.to_string())?.error_for_status().map_err(|e|e.to_string())?.json().await.map_err(|e|e.to_string())
}
async fn updates(files: &[Value], game: &str, loader: &str) -> Result<Value, String> {
    net::client()?.post("https://api.modrinth.com/v2/version_files/update").json(&json!({"hashes":files.iter().map(|v|&v["hash"]).collect::<Vec<_>>(),"algorithm":"sha1","loaders":[loader.to_lowercase()],"game_versions":[game]})).send().await.map_err(|e|e.to_string())?.error_for_status().map_err(|e|e.to_string())?.json().await.map_err(|e|e.to_string())
}

pub async fn scan_mods(state: &State, refresh: bool) -> Result<Value, String> {
    let settings = state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .settings
        .clone();
    let mut files = local_files(&state.instance_dir(&settings)?.join("mods"))?;
    let mut warning = String::new();
    if refresh && !files.is_empty() {
        match identify(&files).await {
            Ok(versions) => {
                let newer = updates(&files, &settings.version, &settings.loader)
                    .await
                    .unwrap_or_else(|e| {
                        warning = format!("Не удалось проверить обновления: {e}");
                        json!({})
                    });
                let projects: HashMap<String, String> = files
                    .iter()
                    .filter_map(|f| {
                        let v = &versions[f["hash"].as_str()?];
                        Some((v["project_id"].as_str()?.into(), f["name"].as_str()?.into()))
                    })
                    .collect();
                for f in &mut files {
                    let hash = f["hash"].as_str().unwrap_or("").to_string();
                    let v = &versions[&hash];
                    if !v.is_null() {
                        f["project_id"] = v["project_id"].clone();
                        f["version_id"] = v["id"].clone();
                        f["version_number"] = v["version_number"].clone();
                        f["dependencies"]=json!(v["dependencies"].as_array().into_iter().flatten().filter(|d|d["dependency_type"]=="required").map(|d|{let id=d["project_id"].as_str().unwrap_or("");json!({"id":id,"name":projects.get(id).cloned().unwrap_or_else(||id.to_string())})}).collect::<Vec<_>>());
                        if !newer[&hash].is_null() && newer[&hash]["id"] != v["id"] {
                            f["update"] = json!({"id":newer[&hash]["id"],"version":newer[&hash]["version_number"]});
                        }
                    }
                }
            }
            Err(e) => {
                warning = format!("Modrinth недоступен: {e}. Локальные переключатели работают.")
            }
        }
    }
    Ok(json!({"files":files,"warning":warning,"profile":settings.instance_id}))
}

pub fn toggle_mod(state: &State, filename: String, enabled: bool) -> Result<(), String> {
    let busy = state.busy.lock().map_err(|e| e.to_string())?;
    if *busy {
        return Err("Закройте игру и дождитесь окончания загрузок".into());
    }
    safe_segment_relaxed(&filename)?;
    if !(filename.ends_with(".jar") || filename.ends_with(".jar.disabled")) {
        return Err("Ожидался JAR-мод".into());
    }
    let s = state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .settings
        .clone();
    let dir = state.instance_dir(&s)?.join("mods");
    let target = if enabled {
        filename.trim_end_matches(".disabled").to_string()
    } else if filename.ends_with(".disabled") {
        filename.clone()
    } else {
        format!("{filename}.disabled")
    };
    if target == filename {
        return Ok(());
    }
    if dir.join(&target).exists() {
        return Err("Уже существует мод с таким именем".into());
    }
    if fs::symlink_metadata(dir.join(&filename))
        .map_err(|e| e.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err("Ссылки не поддерживаются".into());
    }
    fs::rename(dir.join(filename), dir.join(target)).map_err(|e| e.to_string())
}
pub fn remove_mod(state: &State, filename: String) -> Result<(), String> {
    let busy = state.busy.lock().map_err(|e| e.to_string())?;
    if *busy {
        return Err("Закройте Minecraft и дождитесь окончания загрузок".into());
    }
    safe_segment_relaxed(&filename)?;
    let base = filename.strip_suffix(".disabled").unwrap_or(&filename);
    if !base.ends_with(".jar") {
        return Err("Ожидался JAR-мод".into());
    }
    let settings = state.data.lock().map_err(|e| e.to_string())?.settings.clone();
    let profile = state.instance_dir(&settings)?;
    let dir = profile.join("mods");
    if dir.exists() && !fs::canonicalize(&dir).map_err(|e| e.to_string())?
        .starts_with(fs::canonicalize(&profile).map_err(|e| e.to_string())?) {
        return Err("Папка модов находится вне профиля".into());
    }
    let mut moved = Vec::new();
    for name in [base.to_string(), format!("{base}.disabled")] {
        let source = dir.join(&name);
        let meta = match fs::symlink_metadata(&source) {
            Ok(meta) => meta,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.to_string()),
        };
        if !meta.file_type().is_file() {
            return Err("Удаление ссылок и папок не поддерживается".into());
        }
        moved.push((source, name));
    }
    // Keep removed mods recoverable without touching dependencies or other profiles.
    let trash = profile.join("mod-trash").join(chrono::Utc::now().timestamp_micros().to_string());
    if !moved.is_empty() {
        fs::create_dir_all(&trash).map_err(|e| e.to_string())?;
    }
    let mut archived: Vec<(PathBuf, PathBuf)> = Vec::new();
    for (source, name) in moved {
        let destination = trash.join(name);
        if let Err(e) = fs::rename(&source, &destination) {
            for (from, to) in &archived { let _ = fs::rename(to, from); }
            return Err(e.to_string());
        }
        archived.push((source, destination));
    }
    let previous = {
        let mut data = state.data.lock().map_err(|e| e.to_string())?;
        let previous = data.installed.clone();
        data.installed.retain(|item| !(item.kind == "mod"
            && item.game_version == settings.version && item.loader == settings.loader
            && item.instance_id == settings.instance_id
            && item.filename.trim_end_matches(".disabled") == base));
        previous
    };
    if let Err(e) = state.save() {
        state.data.lock().map_err(|e| e.to_string())?.installed = previous;
        for (from, to) in &archived { let _ = fs::rename(to, from); }
        return Err(e);
    }
    Ok(())
}
async fn update_impl(app: &crate::app_context::AppContext, filename: &str) -> Result<(), String> {
    safe_segment_relaxed(filename)?;
    let state = app.state();
    let s = state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .settings
        .clone();
    let instance = state.instance_dir(&s)?;
    let dir = instance.join("mods");
    let files = local_files(&dir)?;
    let chosen = files
        .iter()
        .find(|f| f["filename"] == filename)
        .ok_or("Мод не найден")?;
    let versions = identify(&files).await?;
    let hash = chosen["hash"].as_str().ok_or("Нет хеша")?;
    let original = &versions[hash];
    let new = updates(&[chosen.clone()], &s.version, &s.loader).await?;
    let first = new[hash].clone();
    if first.is_null() || first["id"] == original["id"] {
        return Err("Обновлений для выбранного мода нет".into());
    }
    let stage = instance.join(format!(
        ".mod-update-{}",
        chrono::Utc::now().timestamp_micros()
    ));
    fs::create_dir_all(&stage).map_err(|e| e.to_string())?;
    let result = async {
        let mut queue = vec![first];
        let mut seen = HashSet::new();
        let mut plan: Vec<(PathBuf, PathBuf, Option<PathBuf>, Installed)> = vec![];
        while let Some(v) = queue.pop() {
            let pid = v["project_id"].as_str().ok_or("Нет проекта мода")?;
            if !seen.insert(pid.to_string()) {
                continue;
            }
            if seen.len() > 100 {
                return Err("Слишком много зависимостей".into());
            }
            if !v["game_versions"]
                .as_array()
                .is_some_and(|a| a.contains(&json!(s.version)))
                || !v["loaders"]
                    .as_array()
                    .is_some_and(|a| a.contains(&json!(s.loader.to_lowercase())))
            {
                return Err("Несовместимая версия зависимости".into());
            }
            let old = files
                .iter()
                .find(|f| versions[f["hash"].as_str().unwrap_or("")]["project_id"] == pid);
            if old.is_some_and(|f| versions[f["hash"].as_str().unwrap_or("")]["id"] == v["id"]) {
                continue;
            }
            let file = v["files"]
                .as_array()
                .and_then(|a| {
                    a.iter()
                        .find(|f| f["primary"] == true)
                        .or_else(|| a.first())
                })
                .ok_or("Нет JAR-файла обновления")?;
            let name = file["filename"].as_str().ok_or("Нет имени файла")?;
            safe_segment_relaxed(name)?;
            if !name.ends_with(".jar") {
                return Err("Обновление должно быть JAR".into());
            }
            let url = file["url"]
                .as_str()
                .filter(|u| u.starts_with("https://cdn.modrinth.com/"))
                .ok_or("Неверный источник обновления")?;
            let temp = stage.join(format!("{}-{name}", seen.len()));
            net::download(
                url,
                &temp,
                Some(file["hashes"]["sha1"].as_str().ok_or("Нет SHA-1")?),
            )
            .await?;
            let disabled = old.is_some_and(|f| f["enabled"] == false);
            let target = dir.join(if disabled {
                format!("{name}.disabled")
            } else {
                name.to_string()
            });
            let oldpath = old
                .and_then(|f| f["filename"].as_str())
                .map(|f| dir.join(f));
            if target.exists() && oldpath.as_ref() != Some(&target) {
                return Err(format!("Файл {name} уже существует"));
            }
            if plan.iter().any(|(_, p, _, _)| p == &target) {
                return Err("Зависимости содержат одинаковые имена файлов".into());
            }
            plan.push((
                temp,
                target,
                oldpath,
                Installed {
                    instance_id: s.instance_id.clone(),
                    project_id: pid.into(),
                    version_id: v["id"].as_str().ok_or("Нет версии")?.into(),
                    name: old.and_then(|f| f["name"].as_str()).unwrap_or(pid).into(),
                    filename: name.into(),
                    kind: "mod".into(),
                    game_version: s.version.clone(),
                    loader: s.loader.clone(),
                },
            ));
            for dep in v["dependencies"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|d| d["dependency_type"] == "required")
            {
                if let Some(vid) = dep["version_id"].as_str() {
                    queue.push(
                        net::json(&format!(
                            "https://api.modrinth.com/v2/version/{}",
                            safe_segment(vid)?
                        ))
                        .await?,
                    );
                } else if let Some(pid) = dep["project_id"].as_str() {
                    if !files.iter().any(|f| {
                        f["enabled"] == true
                            && versions[f["hash"].as_str().unwrap_or("")]["project_id"] == pid
                    }) {
                        queue.push(super::compatible(pid, "mod", &s).await?);
                    }
                }
            }
        }
        let backup = instance
            .join("mod-backups")
            .join(chrono::Utc::now().timestamp_micros().to_string());
        fs::create_dir_all(&backup).map_err(|e| e.to_string())?;
        let mut moved: Vec<(PathBuf, PathBuf)> = vec![];
        let mut published: Vec<PathBuf> = vec![];
        let commit = (|| {
            for (temp, target, old, _) in &plan {
                if let Some(old) = old {
                    if old.exists() {
                        let save = backup.join(old.file_name().ok_or("Нет имени")?);
                        fs::rename(old, &save).map_err(|e| e.to_string())?;
                        moved.push((save, old.clone()));
                    }
                }
                fs::rename(temp, target).map_err(|e| e.to_string())?;
                published.push(target.clone());
            }
            Ok::<(), String>(())
        })();
        if let Err(e) = commit {
            for p in published {
                let _ = fs::remove_file(p);
            }
            for (b, p) in moved.into_iter().rev() {
                let _ = fs::rename(b, p);
            }
            return Err(e);
        }
        {
            let mut d = state.data.lock().map_err(|e| e.to_string())?;
            for (_, _, _, item) in plan {
                d.installed.retain(|o| {
                    !(o.project_id == item.project_id
                        && o.instance_id == s.instance_id
                        && o.game_version == s.version
                        && o.loader == s.loader)
                });
                d.installed.push(item);
            }
        }
        state.save()?;
        state.notify(
            "Мод обновлён",
            "Старая версия сохранена в mod-backups внутри папки профиля.",
        );
        Ok(())
    }
    .await;
    let _ = fs::remove_dir_all(stage);
    result
}

pub async fn update_mod(
    app: crate::app_context::AppContext,
    filename: String,
) -> Result<(), String> {
    let state = app.state();
    {
        let mut busy = state.busy.lock().map_err(|e| e.to_string())?;
        if *busy {
            return Err("Дождитесь завершения операции".into());
        }
        *busy = true;
    }
    let result = update_impl(&app, &filename).await;
    *state.busy.lock().map_err(|e| e.to_string())? = false;
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn removes_disabled_mod_from_only_the_active_profile() {
        let root = std::env::temp_dir().join(format!("kyro-remove-mod-{}", chrono::Utc::now().timestamp_micros()));
        let mut data = crate::store::Data::default();
        data.settings.instance_id = "active-pack".into();
        let item = Installed { instance_id: "active-pack".into(), project_id: "sodium".into(),
            version_id: "v1".into(), name: "Sodium".into(), filename: "sodium.jar".into(),
            kind: "mod".into(), game_version: data.settings.version.clone(), loader: data.settings.loader.clone() };
        let mut other = item.clone();
        other.instance_id = "other-pack".into();
        data.installed = vec![item, other];
        let state = State { root: root.clone(), data: std::sync::Mutex::new(data), busy: std::sync::Mutex::new(false) };
        let settings = state.data.lock().unwrap().settings.clone();
        let profile = state.instance_dir(&settings).unwrap();
        fs::create_dir_all(profile.join("mods")).unwrap();
        fs::write(profile.join("mods/sodium.jar.disabled"), b"mod").unwrap();
        fs::write(profile.join("mods/dependency.jar"), b"dependency").unwrap();
        *state.busy.lock().unwrap() = true;
        assert!(remove_mod(&state, "sodium.jar".into()).is_err());
        *state.busy.lock().unwrap() = false;
        assert!(remove_mod(&state, "../outside.jar".into()).is_err());
        remove_mod(&state, "sodium.jar".into()).unwrap();
        assert!(!profile.join("mods/sodium.jar.disabled").exists());
        assert!(profile.join("mods/dependency.jar").exists());
        assert_eq!(state.data.lock().unwrap().installed.len(), 1);
        assert_eq!(state.data.lock().unwrap().installed[0].instance_id, "other-pack");
        let entry = fs::read_dir(profile.join("mod-trash")).unwrap().next().unwrap().unwrap();
        assert_eq!(fs::read(entry.path().join("sodium.jar.disabled")).unwrap(), b"mod");
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn names_are_single_files() {
        for name in ["../mod.jar", "x:y.jar", "x/y.jar", "x\\y.jar"] {
            assert!(safe_segment_relaxed(name).is_err());
        }
        assert!(safe_segment_relaxed("Sodium + 1.0.jar").is_ok());
    }
}
