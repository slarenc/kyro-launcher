use crate::{net, store::Settings};
use serde_json::{json, Value};

fn class(kind: &str) -> Result<u32, String> {
    match kind {
        "mod" => Ok(6), "resourcepack" => Ok(12), "shader" => Ok(6552),
        _ => Err("Сборки CurseForge пока не поддерживаются. Выберите Modrinth для установки сборок.".into()),
    }
}
fn loader(value: &str) -> Result<u32, String> {
    match value {
        "Forge" => Ok(1), "Fabric" => Ok(4), "Quilt" => Ok(5), "NeoForge" => Ok(6),
        _ => Err("Для модов выберите Fabric, Quilt, Forge или NeoForge".into()),
    }
}
async fn get(path: &str, query: &[(&str, String)], settings: &Settings) -> Result<Value, String> {
    let key = std::env::var("CURSEFORGE_API_KEY").ok().filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| settings.curseforge_api_key.clone());
    if key.trim().is_empty() {
        return Err("Укажите API-ключ CurseForge в Настройки → Загрузки.".into());
    }
    let mut header = reqwest::header::HeaderValue::from_str(key.trim()).map_err(|_| "Некорректный API-ключ CurseForge")?;
    header.set_sensitive(true);
    let response = net::client()?.get(format!("https://api.curseforge.com/v1/{path}"))
        .header("x-api-key", header).query(query).send().await.map_err(|e| e.to_string())?;
    if matches!(response.status().as_u16(), 401 | 403) {
        return Err("CurseForge отклонил доступ. Проверьте API-ключ в Настройки → Загрузки.".into());
    }
    response.error_for_status().map_err(|e| e.to_string())?.json().await.map_err(|e| e.to_string())
}
pub async fn search(query: String, kind: String, version: String, core: String, offset: u32, settings: &Settings) -> Result<Value, String> {
    if offset > 9984 { return Err("Достигнут предел выдачи CurseForge".into()); }
    let mut params = vec![("gameId", "432".into()), ("classId", class(&kind)?.to_string()),
        ("searchFilter", query), ("gameVersion", version), ("index", offset.to_string()),
        ("pageSize", "12".into()), ("sortField", "6".into()), ("sortOrder", "desc".into())];
    if kind == "mod" { params.push(("modLoaderType", loader(&core)?.to_string())); }
    let result = get("mods/search", &params, settings).await?;
    let hits: Vec<_> = result["data"].as_array().ok_or("Некорректный ответ CurseForge")?.iter().map(|m| json!({
        "project_id":format!("cf-{}", m["id"].as_u64().unwrap_or_default()),
        "title":m["name"], "description":m["summary"], "icon_url":m["logo"]["thumbnailUrl"],
        "downloads":m["downloadCount"], "author":m["authors"][0]["name"]
    })).collect();
    Ok(json!({"hits":hits, "total_hits":result["pagination"]["totalCount"].as_u64().unwrap_or(0).min(9996)}))
}
pub fn allowed_url(address: &str) -> bool {
    reqwest::Url::parse(address).is_ok_and(|u| u.scheme() == "https" && u.username().is_empty() && u.password().is_none()
        && matches!(u.host_str(), Some("edge.forgecdn.net" | "mediafilez.forgecdn.net")))
}
fn normalize(file: &Value, kind: &str, settings: &Settings) -> Result<Value, String> {
    let versions = file["gameVersions"].as_array().ok_or("Нет совместимости файла CurseForge")?;
    if file["isAvailable"] != true || !versions.contains(&json!(settings.version))
        || (kind == "mod" && !versions.contains(&json!(settings.loader))) {
        return Err("Несовместимый или недоступный файл CurseForge".into());
    }
    let url = file["downloadUrl"].as_str().filter(|u| allowed_url(u))
        .ok_or("Автор ограничил скачивание через сторонние лаунчеры или файл недоступен на CDN CurseForge")?;
    let sha = file["hashes"].as_array().and_then(|a| a.iter().find(|h| h["algo"] == 1))
        .and_then(|h| h["value"].as_str()).filter(|s| s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or("Нет SHA-1 файла CurseForge")?.to_lowercase();
    let filename = file["fileName"].as_str().ok_or("Нет имени файла CurseForge")?;
    crate::safe_segment(filename)?;
    let extension = if kind == "mod" { ".jar" } else { ".zip" };
    if !filename.to_lowercase().ends_with(extension) { return Err("Неподдерживаемый формат файла CurseForge".into()); }
    let dependencies: Vec<_> = file["dependencies"].as_array().into_iter().flatten().filter(|d| d["relationType"] == 3)
        .filter_map(|d| d["modId"].as_u64()).map(|id| json!({"project_id":format!("cf-{id}"),"dependency_type":"required"})).collect();
    Ok(json!({"id":format!("cf-{}",file["id"].as_u64().ok_or("Нет ID файла CurseForge")?),
        "game_versions":[settings.version], "loaders":[settings.loader.to_lowercase()], "dependencies":dependencies,
        "files":[{"primary":true,"filename":filename,"url":url,"hashes":{"sha1":sha}}]}))
}
pub async fn compatible(project: &str, kind: &str, settings: &Settings) -> Result<Value, String> {
    let id = project.strip_prefix("cf-").and_then(|s| s.parse::<u64>().ok()).filter(|id| *id > 0).ok_or("Неверный ID CurseForge")?;
    let metadata = get(&format!("mods/{id}"), &[], settings).await?;
    if metadata["data"]["gameId"] != 432 || metadata["data"]["classId"] != class(kind)? { return Err("Неподходящая категория CurseForge".into()); }
    if metadata["data"]["allowModDistribution"] == false { return Err("Автор запретил скачивание через сторонние лаунчеры".into()); }
    let mut params = vec![("gameVersion", settings.version.clone()), ("pageSize", "50".into())];
    if kind == "mod" { params.push(("modLoaderType", loader(&settings.loader)?.to_string())); }
    let result = get(&format!("mods/{id}/files"), &params, settings).await?;
    let files = result["data"].as_array().ok_or("Некорректный ответ CurseForge")?;
    let file = files.iter().filter(|f| f["isAvailable"] == true && f["gameVersions"].as_array().is_some_and(|a|
        a.contains(&json!(settings.version)) && (kind != "mod" || a.contains(&json!(settings.loader)))))
        .max_by_key(|f| f["fileDate"].as_str().unwrap_or("")).ok_or("Нет совместимого файла CurseForge")?;
    normalize(file, kind, settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verifies_files_and_dependencies() {
        let s = Settings::default();
        let mut f = json!({"id":42,"fileName":"mod.jar","isAvailable":true,"gameVersions":[s.version,s.loader],
            "downloadUrl":"https://edge.forgecdn.net/files/1/2/mod.jar", "hashes":[{"algo":1,"value":"A".repeat(40)}],
            "dependencies":[{"modId":10,"relationType":3},{"modId":11,"relationType":2}]});
        let v = normalize(&f,"mod",&s).unwrap();
        assert_eq!(v["dependencies"].as_array().unwrap().len(),1);
        assert_eq!(v["dependencies"][0]["project_id"],"cf-10");
        assert_eq!(v["files"][0]["hashes"]["sha1"],"a".repeat(40));
        f["downloadUrl"] = Value::Null; assert!(normalize(&f,"mod",&s).is_err());
        f["downloadUrl"] = json!("https://edge.forgecdn.net/files/mod.jar");
        f["gameVersions"] = json!(["1.0","Forge"]); assert!(normalize(&f,"mod",&s).is_err());
    }
    #[test]
    fn rejects_untrusted_hosts_and_unknown_filters() {
        for u in ["http://edge.forgecdn.net/a","https://edge.forgecdn.net.evil.org/a","https://evil.org/a"] { assert!(!allowed_url(u)); }
        assert!(class("modpack").is_err()); assert!(loader("Vanilla").is_err());
        assert_eq!(loader("NeoForge").unwrap(),6);
    }
}
