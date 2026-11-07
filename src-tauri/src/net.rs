use serde_json::Value;
use sha1::{Digest, Sha1};
use std::{path::Path, time::Duration};
pub fn client() -> Result<reqwest::Client, String> {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    if let Some(client) = CLIENT.get() {
        return Ok(client.clone());
    }
    let client = reqwest::Client::builder()
        .user_agent(concat!(
            "KyroLauncher/",
            env!("CARGO_PKG_VERSION"),
            " (native Minecraft launcher)"
        ))
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(15))
        .pool_max_idle_per_host(16)
        .build()
        .map_err(|e| e.to_string())?;
    let _ = CLIENT.set(client.clone());
    Ok(client)
}
pub async fn json(url: &str) -> Result<Value, String> {
    client()?
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}
pub async fn download(url: &str, path: &Path, sha: Option<&str>) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("Загрузка разрешена только по HTTPS".into());
    }
    if let Ok(bytes) = tokio::fs::read(path).await {
        if sha.is_some_and(|hash| format!("{:x}", Sha1::digest(&bytes)) == hash) {
            return Ok(());
        }
    }
    let bytes = client()?
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;
    if let Some(hash) = sha {
        if format!("{:x}", Sha1::digest(&bytes)) != hash {
            return Err(format!("Контрольная сумма не совпала: {}", path.display()));
        }
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }
    let part = path.with_extension("kyro-part");
    tokio::fs::write(&part, bytes)
        .await
        .map_err(|e| e.to_string())?;
    if path.exists() {
        tokio::fs::remove_file(path)
            .await
            .map_err(|e| e.to_string())?;
    }
    tokio::fs::rename(part, path)
        .await
        .map_err(|e| e.to_string())
}
pub async fn download_sha256(url: &str, path: &Path, hash: &str) -> Result<(), String> {
    use sha2::Sha256;
    if hash.len() != 64
        || !hash.bytes().all(|b| b.is_ascii_hexdigit())
        || !url.starts_with("https://")
    {
        return Err("Некорректный адрес или SHA-256".into());
    }
    if let Ok(bytes) = tokio::fs::read(path).await {
        if format!("{:x}", Sha256::digest(&bytes)).eq_ignore_ascii_case(hash) {
            return Ok(());
        }
    }
    let mut response = client()?
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    if response.url().scheme() != "https" {
        return Err("Перенаправление на небезопасный адрес".into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        if bytes.len() + chunk.len() > 1024 * 1024 * 1024 {
            return Err("Файл слишком большой".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    if !format!("{:x}", Sha256::digest(&bytes)).eq_ignore_ascii_case(hash) {
        return Err("SHA-256 загруженного файла не совпал".into());
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }
    let part = path.with_extension("part");
    tokio::fs::write(&part, bytes)
        .await
        .map_err(|e| e.to_string())?;
    if path.exists() {
        tokio::fs::remove_file(path)
            .await
            .map_err(|e| e.to_string())?;
    }
    tokio::fs::rename(part, path)
        .await
        .map_err(|e| e.to_string())
}
