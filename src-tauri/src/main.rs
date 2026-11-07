#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod auth;
mod compatibility;
mod cores;
mod customization;
mod diagnostics;
mod fps_boost;
mod game;
mod managed_mods;
mod modpacks;
mod net;
mod playtools;
mod presence;
mod runtime;
mod servers;
mod store;
mod updates;
use serde_json::{json, Value};
use std::collections::HashSet;
use store::{safe_segment, Installed, Settings, State};

fn snapshot(state: &State) -> Result<Value, String> {
    let d = state.data.lock().map_err(|e| e.to_string())?.clone();
    let microsoft_account =
        auth::load().map(|a| json!({"name":a.name,"uuid":a.uuid,"kind":"microsoft"}));
    let account = if let Some(id) = d
        .selected_account
        .as_deref()
        .filter(|s| s.starts_with("offline:"))
    {
        d.offline_accounts
            .iter()
            .find(|a| format!("offline:{}", a.uuid) == id)
            .map(|a| json!({"name":a.name,"uuid":a.uuid,"kind":"offline"}))
    } else {
        microsoft_account.clone()
    };
    Ok(
        json!({"paths":{"launcher":state.target_path("launcher",&d.settings)?,"data":state.root,"profile":state.target_path("instance",&d.settings)?},"data":d,"account":account,"microsoft_account":microsoft_account,"root":state.root,"busy":*state.busy.lock().map_err(|e|e.to_string())?}),
    )
}

fn offline_account(state: &State, action: String, value: String) -> Result<(), String> {
    let busy = state.busy.lock().map_err(|e| e.to_string())?;
    if *busy {
        return Err("Дождитесь завершения игры или установки".into());
    }
    {
        let mut d = state.data.lock().map_err(|e| e.to_string())?;
        match action.as_str() {
            "add" => d.add_offline(&value)?,
            "select" if value == "microsoft" => {
                if auth::load().is_none() {
                    return Err("Сначала войдите в Microsoft".into());
                }
                d.selected_account = Some(value);
            }
            "select" => d.select_offline(&value)?,
            "remove" => d.remove_offline(&value)?,
            _ => return Err("Неизвестное действие".into()),
        }
    }
    state.save()
}

fn save_settings(state: &State, mut settings: Settings) -> Result<(), String> {
    customization::validate_image(&settings.home_background)?;
    if !["orbit", "card"].contains(&settings.home_layout.as_str()) {
        return Err("Неизвестный вид главного экрана".into());
    }
    if *state.busy.lock().map_err(|e| e.to_string())? {
        return Err("Дождитесь завершения операции".into());
    }
    if !settings.instance_id.is_empty() {
        let data = state.data.lock().map_err(|e| e.to_string())?;
        let pack = data
            .modpacks
            .iter()
            .find(|p| p.id == settings.instance_id)
            .ok_or("Сборка не найдена")?;
        if settings.version != pack.game_version || settings.loader != pack.loader {
            settings.instance_id.clear();
            settings.loader_version.clear();
        } else {
            settings.loader_version = pack.loader_version.clone();
        }
    } else {
        settings.loader_version.clear();
    }
    safe_segment(&settings.version)?;
    if !cores::LOADERS.contains(&settings.loader.as_str()) {
        return Err("Неизвестное ядро".into());
    }
    settings.memory_gb = settings.memory_gb.clamp(2, 32);
    settings.download_threads = settings.download_threads.clamp(1, 16);
    if settings.java_path.trim().is_empty() {
        return Err("Укажите путь к Java".into());
    }
    if settings.accent.len() != 7
        || !settings.accent.starts_with('#')
        || !settings.accent[1..].bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("Некорректный цвет".into());
    }
    state.data.lock().map_err(|e| e.to_string())?.settings = settings;
    state.save()
}

fn notices(state: &State, action: String, id: Option<String>) -> Result<(), String> {
    {
        let mut d = state.data.lock().map_err(|e| e.to_string())?;
        match action.as_str() {
            "clear" => d.notices.clear(),
            "read" => d.notices.iter_mut().for_each(|n| n.unread = false),
            "dismiss" => d.notices.retain(|n| Some(&n.id) != id.as_ref()),
            _ => return Err("Неизвестное действие".into()),
        }
    }
    state.save()
}

async fn versions() -> Result<Value, String> {
    let v = net::json("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json").await?;
    Ok(json!(v["versions"]
        .as_array()
        .ok_or("Нет списка версий")?
        .iter()
        .filter(|v| v["type"] == "release")
        .map(|v| v["id"].clone())
        .collect::<Vec<_>>()))
}

async fn version_library(state: &State) -> Result<Value, String> {
    let root = state.root.clone();
    let sessions = state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .sessions
        .clone();
    let saved_version = state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .settings
        .version
        .clone();
    let mut warning = String::new();
    let cache = root.join("metadata/version-library.json");
    let manifest = match net::json(
        "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
    )
    .await
    {
        Ok(v) => {
            if let Some(parent) = cache.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(bytes) = serde_json::to_vec(&v) {
                let _ = std::fs::write(&cache, bytes);
            }
            v
        }
        Err(_error) => {
            warning = "Не удалось обновить список Minecraft. Проверь подключение и нажми «Обновить список». Показаны сохранённые версии.".into();
            std::fs::read(&cache)
                .ok()
                .and_then(|bytes| serde_json::from_slice(&bytes).ok())
                .unwrap_or_else(|| json!({"versions":[]}))
        }
    };
    let mut releases: Vec<_> = manifest["versions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|v| v["type"] == "release")
        .filter(|v| {
            let id = v["id"].as_str().unwrap_or("");
            !id.starts_with("1.")
                || id
                    .split('.')
                    .nth(1)
                    .and_then(|s| s.parse::<u32>().ok())
                    .is_some_and(|n| n >= 13)
        })
        .map(|v| json!({"id":v["id"],"date":v["releaseTime"]}))
        .collect();
    let mut prepared = Vec::new();
    let mut seen = std::collections::HashSet::new();
    if let Ok(files) = std::fs::read_dir(root.join("profiles")) {
        for file in files.flatten() {
            if file.path().extension().is_some_and(|s| s == "json") {
                if let Ok(bytes) = std::fs::read(file.path()) {
                    if let Ok(v) = serde_json::from_slice::<Value>(&bytes) {
                        let version = v["version"].as_str().unwrap_or("");
                        let loader = v["loader"].as_str().unwrap_or("");
                        if safe_segment(version).is_ok()
                            && cores::LOADERS.contains(&loader)
                            && seen.insert(format!("{version}/{loader}"))
                        {
                            prepared.push(v);
                        }
                    }
                }
            }
        }
    }
    for session in sessions {
        if session.exit_code == Some(0)
            && root
                .join("versions")
                .join(&session.version)
                .join(format!("{}.jar", session.version))
                .exists()
            && seen.insert(format!("{}/{}", session.version, session.loader))
        {
            prepared.push(json!({"version":session.version,"loader":session.loader,"prepared_at":session.started}));
        }
    }
    for profile in &prepared {
        if !releases.iter().any(|r| r["id"] == profile["version"]) {
            releases.push(json!({"id":profile["version"],"date":null}));
        }
    }
    if !releases.iter().any(|r| r["id"] == saved_version) {
        releases.push(json!({"id":saved_version,"date":null}));
    }
    Ok(json!({"releases":releases,"prepared":prepared,"warning":warning}))
}

async fn search_modrinth(
    query: String,
    kind: String,
    version: String,
    loader: String,
    offset: u32,
) -> Result<Value, String> {
    if !["mod", "modpack", "shader", "resourcepack"].contains(&kind.as_str()) {
        return Err("Неизвестная категория".into());
    }
    let mut facets = vec![
        vec![format!("project_type:{kind}")],
        vec![format!("versions:{version}")],
    ];
    if kind == "mod" || kind == "modpack" {
        facets.push(vec![format!("categories:{}", loader.to_lowercase())]);
    }
    net::client()?
        .get("https://api.modrinth.com/v2/search")
        .query(&[
            ("query", query),
            (
                "facets",
                serde_json::to_string(&facets).map_err(|e| e.to_string())?,
            ),
            ("index", "downloads".into()),
            ("limit", "12".into()),
            ("offset", offset.to_string()),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}
async fn compatible(project: &str, kind: &str, s: &Settings) -> Result<Value, String> {
    safe_segment(project)?;
    let mut query = vec![("game_versions", json!([s.version]).to_string())];
    if kind == "mod" || kind == "modpack" {
        query.push(("loaders", json!([s.loader.to_lowercase()]).to_string()));
    }
    let v: Value = net::client()?
        .get(format!(
            "https://api.modrinth.com/v2/project/{project}/version"
        ))
        .query(&query)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    v.as_array()
        .and_then(|v| v.first())
        .cloned()
        .ok_or("Нет совместимой версии для выбранного Minecraft и загрузчика".into())
}
async fn install_impl(
    app: &crate::app_context::AppContext,
    project: String,
    name: String,
    kind: String,
) -> Result<(), String> {
    if kind == "modpack" {
        return modpacks::install(app, &project, &name).await;
    }
    if !["mod", "shader", "resourcepack"].contains(&kind.as_str()) {
        return Err("Сборки .mrpack пока не поддерживаются. Можно устанавливать моды, шейдеры и ресурспаки.".into());
    }
    let state = app.state();
    let s = state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .settings
        .clone();
    if kind == "mod" && s.loader == "Vanilla" {
        return Err("Для модов выберите Fabric, Quilt, Forge или NeoForge".into());
    }
    let mut queue = vec![(project, name, None::<String>)];
    let mut seen = HashSet::new();
    let mut installed = Vec::new();
    while let Some((id, title, exact)) = queue.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        if seen.len() > 100 {
            return Err("Слишком много зависимостей".into());
        }
        let v = if let Some(exact) = exact {
            safe_segment(&exact)?;
            net::json(&format!("https://api.modrinth.com/v2/version/{exact}")).await?
        } else {
            compatible(&id, &kind, &s).await?
        };
        if !v["game_versions"]
            .as_array()
            .is_some_and(|a| a.contains(&json!(s.version)))
        {
            return Err(format!("Несовместимая зависимость: {title}"));
        }
        if kind == "mod"
            && !v["loaders"]
                .as_array()
                .is_some_and(|a| a.contains(&json!(s.loader.to_lowercase())))
        {
            return Err(format!("Неподходящий загрузчик: {title}"));
        }
        let file = v["files"]
            .as_array()
            .and_then(|fs| {
                fs.iter()
                    .find(|f| f["primary"] == true)
                    .or_else(|| fs.first())
            })
            .ok_or("Нет файла загрузки")?;
        let filename = file["filename"].as_str().ok_or("Нет имени файла")?;
        safe_segment(filename)?;
        let url = file["url"].as_str().ok_or("Нет URL файла")?;
        if !url.starts_with("https://cdn.modrinth.com/") {
            return Err("Файл должен находиться на CDN Modrinth".into());
        }
        let sha = file["hashes"]["sha1"]
            .as_str()
            .ok_or("Нет контрольной суммы файла")?;
        let folder = match kind.as_str() {
            "mod" => "mods",
            "shader" => "shaderpacks",
            _ => "resourcepacks",
        };
        game::progress(app, &format!("Установка: {title}"), 0.5);
        let dest = state.instance_dir(&s)?.join(folder).join(filename);
        net::download(url, &dest, Some(sha)).await?;
        installed.push(Installed {
            instance_id: s.instance_id.clone(),
            project_id: id,
            version_id: v["id"].as_str().ok_or("Нет версии проекта")?.into(),
            name: title,
            filename: filename.into(),
            kind: kind.clone(),
            game_version: s.version.clone(),
            loader: s.loader.clone(),
        });
        for dep in v["dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|d| d["dependency_type"] == "required")
        {
            if let Some(pid) = dep["project_id"].as_str() {
                queue.push((
                    pid.into(),
                    pid.into(),
                    dep["version_id"].as_str().map(str::to_owned),
                ));
            } else if let Some(vid) = dep["version_id"].as_str() {
                let dv = net::json(&format!(
                    "https://api.modrinth.com/v2/version/{}",
                    safe_segment(vid)?
                ))
                .await?;
                let pid = dv["project_id"].as_str().ok_or("Нет ID зависимости")?;
                queue.push((pid.into(), pid.into(), Some(vid.into())));
            }
        }
    }
    {
        let mut d = state.data.lock().map_err(|e| e.to_string())?;
        for item in installed {
            let old: Vec<_> = d
                .installed
                .iter()
                .filter(|o| {
                    o.project_id == item.project_id
                        && o.instance_id == s.instance_id
                        && o.game_version == s.version
                        && o.loader == s.loader
                        && o.filename != item.filename
                })
                .cloned()
                .collect();
            for o in old {
                let folder = match o.kind.as_str() {
                    "mod" => "mods",
                    "shader" => "shaderpacks",
                    _ => "resourcepacks",
                };
                let _ = std::fs::remove_file(state.instance_dir(&s)?.join(folder).join(o.filename));
            }
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
        "Установка завершена",
        "Проект и обязательные зависимости установлены в выбранный профиль",
    );
    Ok(())
}

async fn install_mod(
    app: crate::app_context::AppContext,
    project: String,
    name: String,
    kind: String,
) -> Result<(), String> {
    {
        let state = app.state();
        let mut busy = state.busy.lock().map_err(|e| e.to_string())?;
        if *busy {
            return Err("Дождитесь завершения текущей операции".into());
        }
        *busy = true;
    }
    let result = install_impl(&app, project, name, kind).await;
    *app.state().busy.lock().map_err(|e| e.to_string())? = false;
    let _ = app.emit("state-changed", ());
    result
}

async fn launch_game(app: crate::app_context::AppContext) -> Result<(), String> {
    run_launch(app, None).await
}
pub(crate) async fn run_launch(
    app: crate::app_context::AppContext,
    server: Option<String>,
) -> Result<(), String> {
    {
        let state = app.state();
        let mut busy = state.busy.lock().map_err(|e| e.to_string())?;
        if *busy {
            return Err("Игра уже запущена или идёт установка".into());
        }
        *busy = true;
    }
    let record = diagnostics::start(&app);
    let result = game::launch_to(app.clone(), server.as_deref()).await;
    diagnostics::finish(&app, record, &result);
    let state = app.state();
    *state.busy.lock().map_err(|e| e.to_string())? = false;
    if let Err(e) = &result {
        state.notify("Не удалось запустить Minecraft", e);
    }
    let _ = app.emit("state-changed", ());
    result
}

async fn verify_files(app: crate::app_context::AppContext) -> Result<(), String> {
    let s = {
        let state = app.state();
        let mut busy = state.busy.lock().map_err(|e| e.to_string())?;
        if *busy {
            return Err("Дождитесь текущей операции".into());
        }
        *busy = true;
        let s = state
            .data
            .lock()
            .map_err(|e| e.to_string())?
            .settings
            .clone();
        s
    };
    let result = game::prepare(&app, &s).await.map(|_| ());
    let state = app.state();
    *state.busy.lock().map_err(|e| e.to_string())? = false;
    if result.is_ok() {
        state.notify("Файлы проверены", "Minecraft готов к запуску");
    }
    result
}

fn open_folder(state: &State, target: String) -> Result<String, String> {
    let settings = state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .settings
        .clone();
    let path = state.target_path(&target, &settings)?;
    if target == "log" || target == "core-log" || target == "profile-log" {
        if !path.exists() {
            return Err("Лог появится после первого запуска игры".into());
        }
        std::process::Command::new("notepad.exe")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    } else {
        std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
        let canonical = std::fs::canonicalize(&path).map_err(|e| e.to_string())?;
        let normalized = canonical
            .to_string_lossy()
            .trim_start_matches("\\\\?\\")
            .to_string();
        std::process::Command::new("explorer.exe")
            .arg(normalized)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(path.to_string_lossy().into_owned())
}

async fn login_begin(state: &State) -> Result<Value, String> {
    let id = state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .settings
        .microsoft_client_id
        .clone();
    auth::begin(&id).await
}

async fn login_poll(state: &State, code: String) -> Result<Value, String> {
    let id = state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .settings
        .microsoft_client_id
        .clone();
    let result = auth::poll(&id, &code).await?;
    if result["status"] == "complete" {
        state
            .data
            .lock()
            .map_err(|e| e.to_string())?
            .selected_account = Some("microsoft".into());
        state.save()?;
    }
    Ok(result)
}

fn logout() -> Result<(), String> {
    auth::logout()
}

fn open_microsoft_login(url: String) -> Result<(), String> {
    if ![
        "https://www.microsoft.com/link",
        "https://microsoft.com/devicelogin",
        "https://www.microsoft.com/devicelogin",
    ]
    .contains(&url.as_str())
    {
        return Err("Разрешена только страница входа Microsoft".into());
    }
    std::process::Command::new("explorer.exe")
        .arg(url)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
mod app_context;
mod native_ui;
fn main() {
    if updates::handle_cli() {
        return;
    }
    if let Err(e) = native_ui::run() {
        eprintln!("KYRO: {e}");
        rfd::MessageDialog::new()
            .set_title("KYRO — ошибка запуска")
            .set_description(e.to_string())
            .set_level(rfd::MessageLevel::Error)
            .show();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    #[ignore = "requires live Mojang / Modrinth / Fabric APIs"]
    async fn upstream_apis_and_verified_download() {
        let releases = versions().await.expect("Mojang releases");
        assert!(releases.as_array().unwrap().contains(&json!("1.21.1")));
        let results = search_modrinth(
            "sodium".into(),
            "mod".into(),
            "1.21.1".into(),
            "Fabric".into(),
            0,
        )
        .await
        .expect("Modrinth search");
        assert!(!results["hits"].as_array().unwrap().is_empty());
        let selected = compatible("AANobbMI", "mod", &Settings::default())
            .await
            .expect("Sodium 1.21.1 Fabric");
        let file = &selected["files"][0];
        let target = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../target/integration-fixtures/sodium.jar");
        net::download(
            file["url"].as_str().unwrap(),
            &target,
            file["hashes"]["sha1"].as_str(),
        )
        .await
        .expect("verified Modrinth download");
        assert!(target.metadata().unwrap().len() > 0);
        let loaders = net::json("https://meta.fabricmc.net/v2/versions/loader/1.21.1")
            .await
            .expect("Fabric Meta");
        assert!(!loaders.as_array().unwrap().is_empty());
    }
}
