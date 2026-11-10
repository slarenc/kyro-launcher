use crate::{
    auth, cores, net,
    store::{safe_segment, Session, Settings},
};
use futures_util::{stream, StreamExt, TryStreamExt};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Stdio,
    time::Instant,
};

fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v[key]
        .as_str()
        .ok_or_else(|| format!("В манифесте нет поля {key}"))
}
pub fn progress(app: &crate::app_context::AppContext, message: &str, fraction: f64) {
    let _ = app.emit("progress", json!({"message":message,"fraction":fraction}));
}
pub fn allowed(v: &Value) -> bool {
    let Some(rules) = v["rules"].as_array() else {
        return true;
    };
    let mut yes = false;
    for rule in rules {
        let matches_os = rule["os"].is_null()
            || (rule["os"]["name"].as_str().is_none_or(|s| s == "windows")
                && rule["os"]["arch"].as_str().is_none_or(|s| s == "x86_64"));
        if matches_os && rule["features"].is_null() {
            yes = rule["action"] == "allow";
        }
    }
    yes
}
fn maven_path(name: &str) -> Result<String, String> {
    let p: Vec<_> = name.split(':').collect();
    if p.len() < 3
        || p.iter()
            .any(|s| s.contains('/') || s.contains('\\') || *s == "..")
    {
        return Err("Некорректная Maven координата".into());
    }
    let suffix = if p.len() > 3 {
        format!("-{}", p[3])
    } else {
        String::new()
    };
    Ok(format!(
        "{}/{}/{}/{}-{}{}.jar",
        p[0].replace('.', "/"),
        p[1],
        p[2],
        p[1],
        p[2],
        suffix
    ))
}
fn child_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let p = Path::new(relative);
    if p.is_absolute()
        || p.components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err("Небезопасный путь в манифесте".into());
    }
    Ok(root.join(p))
}
pub async fn prepare(
    app: &crate::app_context::AppContext,
    settings: &Settings,
) -> Result<(Value, Vec<PathBuf>, PathBuf, String, String), String> {
    safe_segment(&settings.version)?;
    let state = app.state();
    let root = &state.root;
    progress(app, "Получение манифеста Minecraft", 0.02);
    let (manifest,_) = net::minecraft_manifest(root,Some(&settings.version)).await?;
    let item = manifest["versions"]
        .as_array()
        .and_then(|vs| vs.iter().find(|v| v["id"] == settings.version))
        .ok_or("Версия не найдена")?;
    let mut v = net::minecraft_version(root,item).await?;
    if v["arguments"].is_null() {
        return Err("Поддерживаются версии Minecraft 1.13 и новее".into());
    }
    let major = v["javaVersion"]["majorVersion"].as_u64().unwrap_or(17);
    let mut resolved = settings.clone();
    resolved.java_path = crate::runtime::resolve(app, settings, major, false).await?;
    let settings = &resolved;
    let client = &v["downloads"]["client"];
    let jar = root
        .join("versions")
        .join(&settings.version)
        .join(format!("{}.jar", settings.version));
    net::download(text(client, "url")?, &jar, client["sha1"].as_str()).await?;
    match settings.loader.as_str() {
        "Fabric" | "Quilt" => {
            progress(app, &format!("Подготовка {}", settings.loader), 0.08);
            let profile = if settings.loader_version.is_empty() {
                cores::light_profile(&settings.loader, &settings.version).await?
            } else {
                cores::light_profile_version(
                    &settings.loader,
                    &settings.version,
                    &settings.loader_version,
                )
                .await?
            };
            cores::merge_profile(&mut v, &profile)?;
        }
        "NeoForge" => {
            progress(
                app,
                "Установка NeoForge: загрузка библиотек и подготовка клиента",
                0.08,
            );
            let profile = cores::neoforge_profile(root, settings, &v).await?;
            cores::merge_profile(&mut v, &profile)?;
        }
        "Forge" => {
            progress(app, "Установка Forge официальным установщиком", 0.08);
            let profile = cores::forge_profile(root, settings, &v).await?;
            cores::merge_profile(&mut v, &profile)?;
        }
        "Vanilla" => {}
        _ => return Err("Неизвестное ядро".into()),
    }
    let natives = root
        .join("versions")
        .join(&settings.version)
        .join("natives");
    fs::create_dir_all(&natives).map_err(|e| e.to_string())?;
    let mut jobs: Vec<(String, PathBuf, Option<String>, bool)> = Vec::new();
    let mut cp = Vec::new();
    for lib in v["libraries"].as_array().ok_or("Нет библиотек")? {
        if !allowed(lib) {
            continue;
        }
        let artifact = &lib["downloads"]["artifact"];
        if let Some(url) = artifact["url"].as_str().filter(|s| !s.is_empty()) {
            let path = child_path(&root.join("libraries"), text(artifact, "path")?)?;
            cp.push(path.clone());
            jobs.push((
                url.into(),
                path,
                artifact["sha1"].as_str().map(str::to_owned),
                false,
            ));
        } else if let Some(relative) = artifact["path"].as_str() {
            let path = child_path(&root.join("libraries"), relative)?;
            if !path.exists() {
                return Err(format!(
                    "Не найдена локальная библиотека ядра: {relative}. Повторите установку ядра."
                ));
            }
            cp.push(path);
        } else if lib["downloads"].is_null() {
            let relative = maven_path(text(lib, "name")?)?;
            let path = child_path(&root.join("libraries"), &relative)?;
            cp.push(path.clone());
            jobs.push((
                format!(
                    "{}{relative}",
                    lib["url"]
                        .as_str()
                        .unwrap_or("https://libraries.minecraft.net/")
                ),
                path,
                None,
                false,
            ));
        }
        if let Some(classifier) = lib["natives"]["windows"].as_str() {
            let key = classifier.replace("${arch}", "64");
            let native = &lib["downloads"]["classifiers"][&key];
            let path = child_path(&root.join("libraries"), text(native, "path")?)?;
            jobs.push((
                text(native, "url")?.into(),
                path,
                native["sha1"].as_str().map(str::to_owned),
                true,
            ));
        }
    }
    progress(app, "Загрузка и проверка библиотек", 0.15);
    let native_paths: Vec<_> = jobs.iter().filter(|j| j.3).map(|j| j.1.clone()).collect();
    let total = jobs.len().max(1);
    let done = std::sync::atomic::AtomicUsize::new(0);
    let counter = &done;
    stream::iter(jobs.into_iter().map(|(url, path, sha, _)| async move {
        net::download(&url, &path, sha.as_deref()).await?;
        let count = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        progress(
            app,
            &format!("Библиотеки: {count} / {total}"),
            0.15 + 0.35 * count as f64 / total as f64,
        );
        Ok::<(), String>(())
    }))
    .buffer_unordered(settings.download_threads.clamp(1, 16))
    .try_collect::<Vec<_>>()
    .await?;
    for path in native_paths {
        let file = fs::File::open(path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        for i in 0..archive.len() {
            let mut f = archive.by_index(i).map_err(|e| e.to_string())?;
            let Some(name) = f.enclosed_name() else {
                continue;
            };
            if !name.to_string_lossy().ends_with(".dll") {
                continue;
            }
            let dest = natives.join(name.file_name().ok_or("Нет имени DLL")?);
            let mut output = fs::File::create(dest).map_err(|e| e.to_string())?;
            std::io::copy(&mut f, &mut output).map_err(|e| e.to_string())?;
        }
    }
    progress(app, "Загрузка индекса ресурсов", 0.55);
    let index_id = text(&v["assetIndex"], "id")?.to_owned();
    safe_segment(&index_id)?;
    let index_path = root.join("assets/indexes").join(format!("{index_id}.json"));
    net::download(
        text(&v["assetIndex"], "url")?,
        &index_path,
        v["assetIndex"]["sha1"].as_str(),
    )
    .await?;
    let index: Value = serde_json::from_slice(&fs::read(index_path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let objects = index["objects"].as_object().ok_or("Нет списка ресурсов")?;
    let mut unique = std::collections::HashSet::new();
    let mut assets = Vec::new();
    for obj in objects.values() {
        let hash = text(obj, "hash")?;
        if hash.len() != 40 || !hash.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err("Некорректный хеш ресурса".into());
        }
        if unique.insert(hash.to_owned()) {
            assets.push((
                format!(
                    "https://resources.download.minecraft.net/{}/{hash}",
                    &hash[..2]
                ),
                root.join("assets/objects").join(&hash[..2]).join(hash),
                hash.to_owned(),
            ));
        }
    }
    progress(
        app,
        &format!(
            "Проверка {} ресурсов — первый запуск может занять несколько минут",
            assets.len()
        ),
        0.65,
    );
    let total = assets.len().max(1);
    let done = std::sync::atomic::AtomicUsize::new(0);
    let counter = &done;
    stream::iter(assets.into_iter().map(|(url, path, sha)| async move {
        net::download(&url, &path, Some(&sha)).await?;
        let count = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        if count % 20 == 0 || count == total {
            progress(
                app,
                &format!("Ресурсы Minecraft: {count} / {total}"),
                0.65 + 0.30 * count as f64 / total as f64,
            );
        }
        Ok::<(), String>(())
    }))
    .buffer_unordered(settings.download_threads.clamp(1, 16))
    .try_collect::<Vec<_>>()
    .await?;
    cp.push(jar);
    fs::create_dir_all(root.join("profiles")).map_err(|e| e.to_string())?;
    let record = json!({"version":settings.version,"loader":settings.loader,"prepared_at":chrono::Utc::now().to_rfc3339()});
    fs::write(
        root.join("profiles")
            .join(format!("{}-{}.json", settings.version, settings.loader)),
        serde_json::to_vec(&record).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok((v, cp, natives, index_id, settings.java_path.clone()))
}
fn arguments(list: &Value, vars: &HashMap<String, String>) -> Vec<String> {
    let mut args = Vec::new();
    for item in list.as_array().into_iter().flatten() {
        if let Some(s) = item.as_str() {
            args.push(s.to_owned());
        } else if allowed(item) {
            if let Some(s) = item["value"].as_str() {
                args.push(s.into());
            } else if let Some(a) = item["value"].as_array() {
                args.extend(a.iter().filter_map(Value::as_str).map(str::to_owned));
            }
        }
    }
    args.into_iter()
        .map(|mut s| {
            for (key, value) in vars {
                s = s.replace(&format!("${{{key}}}"), value);
            }
            s
        })
        .collect()
}
pub async fn launch_to(
    app: crate::app_context::AppContext,
    server: Option<&str>,
    attempt: &crate::launch_control::LaunchAttempt,
) -> Result<(), String> {
    let state = app.state();
    let (mut cmd, settings) = attempt
        .prepare(async {
            let settings = state
                .data
                .lock()
                .map_err(|e| e.to_string())?
                .settings
                .clone();
            let offline = {
                let d = state.data.lock().map_err(|e| e.to_string())?;
                if let Some(id) = d
                    .selected_account
                    .as_deref()
                    .filter(|s| s.starts_with("offline:"))
                {
                    Some(
                        d.offline_accounts
                            .iter()
                            .find(|a| format!("offline:{}", a.uuid) == id)
                            .cloned()
                            .ok_or("Офлайн-аккаунт не найден. Выберите аккаунт в профиле")?,
                    )
                } else {
                    None
                }
            };
            let is_offline = offline.is_some();
            let account = if let Some(a) = offline {
                auth::Account {
                    name: a.name,
                    uuid: a.uuid,
                    access_token: "0".into(),
                    refresh_token: String::new(),
                    expires: i64::MAX,
                }
            } else {
                auth::current(&settings.microsoft_client_id).await?
            };
            let (v, cp, natives, index, java_path) = prepare(&app, &settings).await?;
            let game_dir = state.instance_dir(&settings)?;
            fs::create_dir_all(&game_dir).map_err(|e| e.to_string())?;
            let cp = std::env::join_paths(cp)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned();
            let mut vars = HashMap::new();
            for (key, value) in [
                ("auth_player_name", account.name),
                ("auth_uuid", account.uuid),
                ("auth_access_token", account.access_token),
                (
                    "user_type",
                    if is_offline { "legacy" } else { "msa" }.into(),
                ),
                ("version_name", settings.version.clone()),
                ("version_type", "release".into()),
                ("game_directory", game_dir.to_string_lossy().into_owned()),
                (
                    "assets_root",
                    state.root.join("assets").to_string_lossy().into_owned(),
                ),
                ("assets_index_name", index),
                ("natives_directory", natives.to_string_lossy().into_owned()),
                ("classpath", cp),
                ("launcher_name", "KYRO".into()),
                ("launcher_version", env!("CARGO_PKG_VERSION").into()),
                ("user_properties", "{}".into()),
                ("clientid", settings.microsoft_client_id.clone()),
                ("auth_xuid", String::new()),
                ("classpath_separator", ";".into()),
                (
                    "library_directory",
                    state.root.join("libraries").to_string_lossy().into_owned(),
                ),
            ] {
                vars.insert(key.to_owned(), value);
            }
            let mut cmd = tokio::process::Command::new(&java_path);
            cmd.current_dir(&game_dir)
                .arg(format!("-Xmx{}G", settings.memory_gb.clamp(2, 32)))
                .arg("-Xms512M");
            if let Some(config) = v["logging"]["client"]["file"].as_object() {
                let name = config
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or("Нет имени конфигурации логирования")?;
                safe_segment(name)?;
                let path = state.root.join("assets/log_configs").join(name);
                net::download(
                    config
                        .get("url")
                        .and_then(Value::as_str)
                        .ok_or("Нет URL конфигурации логирования")?,
                    &path,
                    config.get("sha1").and_then(Value::as_str),
                )
                .await?;
                if let Some(arg) = v["logging"]["client"]["argument"].as_str() {
                    cmd.arg(arg.replace("${path}", &path.to_string_lossy()));
                }
            }
            // Arguments are passed directly to Java, never through a shell.
            cmd.args(settings.jvm_args.split_whitespace())
                .args(arguments(&v["arguments"]["jvm"], &vars));
            // Built-in HUD is independent of Minecraft mappings and the selected mod loader.
            let overlay = crate::hud::prepare(state, &settings)?;
            cmd.arg(format!("-Dkyro.hud.player={}", vars.get("auth_player_name").ok_or("Нет имени игрока")?));
            cmd.arg(format!("-Dkyro.hud.uuid={}", vars.get("auth_uuid").ok_or("Нет UUID игрока")?));
            cmd.arg("-Dkyro.hud.presence=https://kyro-client-badges.vinousrhubarb4.chatgpt.site");
            crate::hud::prepare_mappings(state, &settings, &v).await;
            cmd.args(overlay);
            cmd.arg(text(&v, "mainClass")?)
                .args(arguments(&v["arguments"]["game"], &vars));
            if let Some(address) = server {
                cmd.args(crate::servers::connect_args(&settings.version, address)?);
            }
            let log = fs::File::create(state.root.join("game.log")).map_err(|e| e.to_string())?;
            cmd.stdout(Stdio::from(log.try_clone().map_err(|e| e.to_string())?))
                .stderr(Stdio::from(log));
            #[cfg(windows)]
            {
                cmd.creation_flags(0x08000000);
            }
            Ok((cmd, settings))
        })
        .await?;
    attempt.start_game()?;
    let _ = app.emit("launch-state", json!({"can_cancel":false,"running":true}));
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Не удалось запустить Java: {e}"))?;
    progress(&app, "Minecraft запущен", 1.0);
    app.presence
        .game_started(&settings, server, state.root.join("game.log"));
    let start = Instant::now();
    let started = chrono::Utc::now().to_rfc3339();
    match settings.launch_behavior.as_str() {
        "hide" => app.window(crate::app_context::WindowAction::Hide),
        "minimize" => app.window(crate::app_context::WindowAction::Minimize),
        _ => {}
    }
    let result = child.wait().await;
    app.presence.game_finished();
    app.window(crate::app_context::WindowAction::Restore);
    let exit = result.map_err(|e| e.to_string())?;
    state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .sessions
        .push(Session {
            started,
            seconds: start.elapsed().as_secs(),
            version: settings.version,
            loader: settings.loader,
            exit_code: exit.code(),
        });
    state.save()?;
    if !exit.success() {
        return Err(format!(
            "Игра завершилась с кодом {:?}. Подробности в game.log (Настройки → Лог игры)",
            exit.code()
        ));
    }
    progress(&app, "Сессия сохранена", 1.0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rules_respect_order_and_features() {
        assert!(allowed(
            &json!({"rules":[{"action":"allow"},{"action":"disallow","os":{"name":"osx"}}]})
        ));
        assert!(!allowed(
            &json!({"rules":[{"action":"allow","features":{"is_demo_user":true}}]})
        ));
    }
    #[test]
    fn paths_cannot_escape_root() {
        assert!(child_path(Path::new("C:/root"), "../bad").is_err());
        assert!(maven_path("org.example:lib:1.0")
            .unwrap()
            .ends_with("lib-1.0.jar"));
    }
    #[test]
    fn expansion_preserves_spaces() {
        let vars = HashMap::from([("game_directory".into(), "C:/My Games".into())]);
        assert_eq!(
            arguments(&json!(["--gameDir", "${game_directory}"]), &vars),
            vec!["--gameDir", "C:/My Games"]
        );
    }
}
