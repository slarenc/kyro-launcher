//! Curated optimization mods; compatibility and required dependencies come from Modrinth.
use crate::{
    app_context::AppContext,
    managed_mods, net,
    store::{safe_segment, Installed, Settings, State},
};
use serde_json::{json, Value};
use std::{collections::HashMap, fs};

const MODS: &[(&str, &str, &str)] = &[
    ("AANobbMI", "Sodium", "Быстрая отрисовка мира"),
    ("sk9rgfiA", "Embeddium", "Быстрая отрисовка для Forge"),
    ("gvQqBUqZ", "Lithium", "Оптимизация игровой логики"),
    ("uXXizFIs", "FerriteCore", "Меньше расход памяти"),
    (
        "5ZwdcRci",
        "ImmediatelyFast",
        "Быстрая отрисовка интерфейса и объектов",
    ),
    ("NNAgCjsB", "Entity Culling", "Не рисует скрытые объекты"),
    ("nmDcB62a", "ModernFix", "Оптимизация загрузки и памяти"),
];

/// Enabled optimization mods in the current local scan, including manually installed ones.
pub fn enabled_count(scan: &Value) -> usize {
    scan["files"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|f| {
            f["enabled"] == true
                && MODS.iter().any(|(id, name, _)| {
                    if let Some(project) = f["project_id"].as_str() {
                        return project == *id;
                    }
                    let filename = f["filename"].as_str().unwrap_or("").to_lowercase();
                    let name = name.to_lowercase().replace(' ', "");
                    if name == "sodium" && filename.starts_with("sodium-extra") {
                        return false;
                    }
                    filename.starts_with(&format!("{name}-"))
                        || filename.starts_with(&format!("{name}_"))
                        || filename == format!("{name}.jar")
                })
        })
        .count()
}

fn settings(state: &State) -> Result<Settings, String> {
    Ok(state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .settings
        .clone())
}
fn scope(s: &Settings) -> String {
    format!("{}|{}|{}", s.version, s.loader, s.instance_id)
}
fn valid_version(v: &Value, s: &Settings) -> Result<(), String> {
    if !v["game_versions"]
        .as_array()
        .is_some_and(|a| a.contains(&json!(s.version)))
        || !v["loaders"]
            .as_array()
            .is_some_and(|a| a.contains(&json!(s.loader.to_lowercase())))
    {
        return Err("Зависимость не подходит к выбранной версии и ядру".into());
    }
    Ok(())
}
fn file(v: &Value) -> Result<&Value, String> {
    let f = v["files"]
        .as_array()
        .and_then(|a| {
            a.iter()
                .find(|f| f["primary"] == true)
                .or_else(|| a.first())
        })
        .ok_or("Нет файла мода")?;
    let name = f["filename"].as_str().ok_or("Нет имени файла")?;
    safe_segment(name)?;
    if !name.ends_with(".jar")
        || !f["url"]
            .as_str()
            .is_some_and(|s| s.starts_with("https://cdn.modrinth.com/"))
        || !f["hashes"]["sha1"]
            .as_str()
            .is_some_and(|s| s.len() == 40 && s.bytes().all(|c| c.is_ascii_hexdigit()))
    {
        return Err("Некорректный файл или контрольная сумма Modrinth".into());
    }
    Ok(f)
}

pub async fn plan(state: &State) -> Result<Value, String> {
    let s = settings(state)?;
    if s.loader == "Vanilla" {
        return Err(
            "Для FPS-буста выбери Fabric, Quilt, Forge или NeoForge. Vanilla не загружает моды."
                .into(),
        );
    }
    let local = managed_mods::scan_mods(state, false).await?;
    let files = local["files"].as_array().cloned().unwrap_or_default();
    let identified = if files.is_empty() {
        json!({})
    } else {
        managed_mods::identify(&files).await?
    };
    let mut existing = HashMap::<String, Value>::new();
    for f in &files {
        let v = &identified[f["hash"].as_str().unwrap_or("")];
        if let Some(pid) = v["project_id"].as_str() {
            existing.insert(pid.into(), v.clone());
        }
    }
    let renderer_exists = ["AANobbMI", "sk9rgfiA", "rubidium", "optifine"]
        .iter()
        .any(|p| existing.contains_key(*p))
        || files.iter().any(|f| {
            ["sodium", "embeddium", "rubidium", "optifine"]
                .iter()
                .any(|p| {
                    f["filename"]
                        .as_str()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(p)
                })
        });
    let mut renderer = renderer_exists;
    let mut rows = Vec::new();
    let mut queue = Vec::new();
    for &(id, name, description) in MODS {
        let rendering = id == "AANobbMI" || id == "sk9rgfiA";
        let installed = existing.contains_key(id)
            || files.iter().any(|f| {
                f["filename"]
                    .as_str()
                    .unwrap_or("")
                    .to_lowercase()
                    .starts_with(&name.to_lowercase().replace(' ', ""))
            });
        if installed || (rendering && renderer) {
            rows.push(json!({"name":name,"description":description,"status":if installed {"Уже установлен (включая отключённые)"} else {"Уже выбран другой мод отрисовки"}}));
            continue;
        }
        match crate::compatible(id, "mod", &s).await {
            Ok(v) => {
                if rendering { renderer = true; }
                rows.push(json!({"name":name,"description":description,"status":"Будет установлен"}));
                queue.push((v, name.to_string()));
            }
            Err(e) if e.contains("Нет совместимой версии") => rows.push(json!({"name":name,"description":description,"status":"Нет версии для этого Minecraft и ядра"})),
            Err(e) => return Err(format!("Не удалось проверить {name}: {e}")),
        }
    }
    let mut selected = HashMap::<String, String>::new();
    let mut items = Vec::new();
    while let Some((v, name)) = queue.pop() {
        valid_version(&v, &s)?;
        let pid = v["project_id"]
            .as_str()
            .ok_or("Нет проекта Modrinth")?
            .to_string();
        let vid = v["id"].as_str().ok_or("Нет версии Modrinth")?.to_string();
        if let Some(old) = existing.get(&pid) {
            if files.iter().any(|f| {
                identified[f["hash"].as_str().unwrap_or("")]["project_id"] == pid
                    && f["enabled"] == false
            }) {
                return Err(format!(
                    "Зависимость {name} отключена. Включи её в менеджере модов."
                ));
            }
            if old["id"] != vid {
                return Err(format!("{name}: требуется другая версия уже установленной зависимости. Обнови её в менеджере модов."));
            }
            continue;
        }
        if let Some(previous) = selected.get(&pid) {
            if previous != &vid {
                return Err(format!("Конфликт версий зависимости: {name}"));
            }
            continue;
        }
        if selected.len() >= 100 {
            return Err("Слишком много зависимостей".into());
        }
        selected.insert(pid.clone(), vid.clone());
        let f = file(&v)?.clone();
        if files.iter().any(|old| {
            old["filename"] == f["filename"]
                || old["filename"] == format!("{}.disabled", f["filename"].as_str().unwrap_or(""))
        }) {
            return Err(format!(
                "Файл {name} уже существует. Проверь его в менеджере модов."
            ));
        }
        for dep in v["dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|d| d["dependency_type"] == "required")
        {
            if let Some(exact) = dep["version_id"].as_str() {
                queue.push((
                    net::json(&format!(
                        "https://api.modrinth.com/v2/version/{}",
                        safe_segment(exact)?
                    ))
                    .await?,
                    "Зависимость".into(),
                ));
            } else if let Some(project) = dep["project_id"].as_str() {
                if existing.contains_key(project) {
                    // A disabled required dependency cannot satisfy the install.
                    if files.iter().any(|f| {
                        identified[f["hash"].as_str().unwrap_or("")]["project_id"] == project
                            && f["enabled"] == false
                    }) {
                        return Err(
                            "Нужная зависимость отключена. Включи её в менеджере модов.".into()
                        );
                    }
                } else {
                    queue.push((
                        crate::compatible(project, "mod", &s).await?,
                        "Зависимость".into(),
                    ));
                }
            }
        }
        items.push(json!({"project":pid,"version":vid,"name":name,"file":f,"metadata":v}));
    }
    for item in &items {
        for d in item["metadata"]["dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|d| d["dependency_type"] == "incompatible")
        {
            let pid = d["project_id"].as_str().unwrap_or("");
            let exact = d["version_id"].as_str();
            if existing
                .get(pid)
                .is_some_and(|v| exact.is_none_or(|e| v["id"] == e))
                || items
                    .iter()
                    .any(|v| v["project"] == pid && exact.is_none_or(|e| v["version"] == e))
            {
                return Err(format!(
                    "{} конфликтует с модом в профиле. Проверь совместимость в менеджере.",
                    item["name"].as_str().unwrap_or("Мод")
                ));
            }
        }
    }
    for old in existing.values() {
        for d in old["dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|d| d["dependency_type"] == "incompatible")
        {
            if items.iter().any(|v| {
                v["project"] == d["project_id"]
                    && (d["version_id"].is_null() || v["version"] == d["version_id"])
            }) {
                return Err("Установленный мод объявляет конфликт с FPS-бустом. Проверь совместимость в менеджере.".into());
            }
        }
    }
    items.sort_by_key(|v| v["project"].as_str().unwrap_or("").to_owned());
    if scope(&settings(state)?) != scope(&s) {
        return Err("Профиль изменился. Открой FPS-буст снова.".into());
    }
    Ok(
        json!({"scope":scope(&s),"label":format!("{} · {}",s.version,s.loader),"rows":rows,"items":items,"local":files,
        "unknown":files.iter().filter(|f| identified[f["hash"].as_str().unwrap_or("")].is_null()).count()}),
    )
}

pub async fn install(app: AppContext, approved: Value) -> Result<Value, String> {
    {
        let mut busy = app.state().busy.lock().map_err(|e| e.to_string())?;
        if *busy {
            return Err("Дождись окончания игры или установки".into());
        }
        *busy = true;
    }
    let result = install_inner(&app, &approved).await;
    *app.state().busy.lock().map_err(|e| e.to_string())? = false;
    let _ = app.emit("state-changed", ());
    result
}
async fn install_inner(app: &AppContext, approved: &Value) -> Result<Value, String> {
    let state = app.state();
    if approved["scope"] != scope(&settings(state)?) {
        return Err("Выбран другой профиль. Повтори подбор модов.".into());
    }
    let fresh = plan(state).await?;
    // Download counters in upstream metadata change independently of the plan.
    let signature = |plan: &Value| -> Vec<Value> {
        plan["items"].as_array().into_iter().flatten().map(|item| {
            json!({"project":item["project"],"version":item["version"],"file":item["file"]})
        }).collect()
    };
    if signature(&fresh) != signature(approved) || fresh["local"] != approved["local"] {
        return Err("Список модов изменился. Повтори подбор перед установкой.".into());
    }
    let items = fresh["items"].as_array().ok_or("Нет плана установки")?;
    let s = settings(state)?;
    let instance = state.instance_dir(&s)?;
    let stage = instance.join(format!(
        ".fps-stage-{}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    fs::create_dir_all(&stage).map_err(|e| e.to_string())?;
    let mut created = Vec::new();
    let result = async {
        for (index, item) in items.iter().enumerate() {
            crate::game::progress(
                app,
                &format!(
                    "FPS-буст: {}",
                    item["name"].as_str().unwrap_or("Зависимость")
                ),
                index as f64 / items.len().max(1) as f64,
            );
            let f = &item["file"];
            net::download(
                f["url"].as_str().ok_or("Нет URL")?,
                &stage.join(f["filename"].as_str().ok_or("Нет имени")?),
                f["hashes"]["sha1"].as_str(),
            )
            .await?;
        }
        let mods = instance.join("mods");
        fs::create_dir_all(&mods).map_err(|e| e.to_string())?;
        for item in items {
            let filename = item["file"]["filename"].as_str().ok_or("Нет имени")?;
            let dest = mods.join(filename);
            // Create without replacing a file added externally during download.
            fs::hard_link(stage.join(filename), &dest).map_err(|e| e.to_string())?;
            created.push(dest);
        }
        let previous = {
            let mut data = state.data.lock().map_err(|e| e.to_string())?;
            let previous = data.installed.clone();
            for item in items {
                data.installed.push(Installed {
                    instance_id: s.instance_id.clone(),
                    project_id: item["project"].as_str().unwrap().into(),
                    version_id: item["version"].as_str().unwrap().into(),
                    name: item["name"].as_str().unwrap().into(),
                    filename: item["file"]["filename"].as_str().unwrap().into(),
                    kind: "mod".into(),
                    game_version: s.version.clone(),
                    loader: s.loader.clone(),
                });
            }
            previous
        };
        if let Err(e) = state.save() {
            state.data.lock().map_err(|e| e.to_string())?.installed = previous;
            return Err(e);
        }
        Ok(json!({"installed":items.len()}))
    }
    .await;
    if result.is_err() {
        for path in created {
            let _ = fs::remove_file(path);
        }
    }
    let _ = fs::remove_dir_all(&stage);
    if result.is_ok() {
        state.notify(
            "FPS-буст установлен",
            &format!(
                "{} · {}: установлено {} файлов",
                s.version,
                s.loader,
                items.len()
            ),
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn highlight_requires_enabled_optimization_mods() {
        assert_eq!(
            enabled_count(&json!({"files":[
                {"filename":"sodium-fabric-1.21.1.jar","enabled":true},
                {"filename":"lithium-fabric.jar.disabled","enabled":false},
                {"filename":"sodium-extra-1.0.jar","project_id":"other","enabled":true}
            ]})),
            1
        );
        assert_eq!(
            enabled_count(
                &json!({"files":[{"filename":"ferritecore-7.jar.disabled","enabled":false}]})
            ),
            0
        );
        assert_eq!(enabled_count(&Value::Null), 0);
    }
    #[test]
    #[ignore = "downloads real Modrinth files into an isolated target directory"]
    fn installs_real_mods_and_skips_them_on_second_pass() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../target")
            .join(format!(
                "fps-integration-{}",
                chrono::Utc::now().timestamp_micros()
            ));
        let mut data = crate::store::Data::default();
        data.settings.version = "1.21.1".into();
        data.settings.loader = "Fabric".into();
        let state = State {
            root,
            data: std::sync::Mutex::new(data),
            busy: std::sync::Mutex::new(false),
        };
        let app = AppContext::headless(std::sync::Arc::new(state));
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let approved = plan(app.state()).await.unwrap();
            assert!(approved["items"].as_array().unwrap().len() >= 4);
            let mut stale = approved.clone();
            stale["scope"] = json!("other-profile");
            assert!(install(app.clone(), stale).await.is_err());
            assert!(!*app.state().busy.lock().unwrap());
            let result = install(app.clone(), approved.clone()).await.unwrap();
            assert_eq!(
                result["installed"].as_u64().unwrap() as usize,
                approved["items"].as_array().unwrap().len()
            );
            let dir = app
                .state()
                .instance_dir(&settings(app.state()).unwrap())
                .unwrap();
            for item in approved["items"].as_array().unwrap() {
                let path = dir
                    .join("mods")
                    .join(item["file"]["filename"].as_str().unwrap());
                assert!(path.is_file());
                use sha1::{Digest, Sha1};
                assert_eq!(
                    format!("{:x}", Sha1::digest(fs::read(path).unwrap())),
                    item["file"]["hashes"]["sha1"].as_str().unwrap()
                );
            }
            let second = plan(app.state()).await.unwrap();
            assert!(second["items"].as_array().unwrap().is_empty(), "{second}");
            assert!(!dir.join("options.txt").exists());
            assert!(!*app.state().busy.lock().unwrap());
        });
    }
    #[test]
    fn reject_wrong_loader_or_game_and_unsafe_downloads() {
        let mut s = Settings::default();
        s.version = "1.21.1".into();
        s.loader = "Fabric".into();
        let v = json!({"game_versions":["1.21.1"],"loaders":["fabric"]});
        assert!(valid_version(&v, &s).is_ok());
        s.loader = "NeoForge".into();
        assert!(valid_version(&v, &s).is_err());
        s.loader = "Fabric".into();
        s.version = "1.20.1".into();
        assert!(valid_version(&v, &s).is_err());
        let mut v = json!({"files":[{"filename":"test.jar","primary":true,"url":"https://cdn.modrinth.com/data/test.jar","hashes":{"sha1":"a".repeat(40)}}]});
        assert!(file(&v).is_ok());
        v["files"][0]["filename"] = json!("../test.jar");
        assert!(file(&v).is_err());
        v["files"][0]["filename"] = json!("test.jar");
        v["files"][0]["url"] = json!("https://example.com/test.jar");
        assert!(file(&v).is_err());
    }
}
