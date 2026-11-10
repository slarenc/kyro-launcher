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
const MANIFEST_URLS:[&str;2]=["https://piston-meta.mojang.com/mc/game/version_manifest_v2.json","https://launchermeta.mojang.com/mc/game/version_manifest_v2.json"];
fn cached_value(path:&Path,sha:Option<&str>,valid:&impl Fn(&Value)->bool)->Option<Value>{
    let bytes=std::fs::read(path).ok()?;if sha.is_some_and(|hash|!format!("{:x}",Sha1::digest(&bytes)).eq_ignore_ascii_case(hash)){return None;}
    let value=serde_json::from_slice(&bytes).ok()?;valid(&value).then_some(value)
}
async fn cached_metadata(path:&Path,urls:&[&str],sha:Option<&str>,cache_first:bool,valid:impl Fn(&Value)->bool)->Result<(Value,bool),String>{
    let cached=cached_value(path,sha,&valid);if cache_first{if let Some(value)=&cached{return Ok((value.clone(),true));}}
    let mut failures=Vec::new();
    for url in urls{
        let result:Result<Value,String>=async{
            let response=client()?.get(*url).timeout(Duration::from_secs(8)).send().await.map_err(|e|format!("{e:?}"))?.error_for_status().map_err(|e|format!("{e:?}"))?;
            let bytes=response.bytes().await.map_err(|e|e.to_string())?;if bytes.len()>8*1024*1024{return Err("Слишком большой манифест".into());}
            if sha.is_some_and(|hash|!format!("{:x}",Sha1::digest(&bytes)).eq_ignore_ascii_case(hash)){return Err("Контрольная сумма метаданных Minecraft не совпала".into());}
            let value:Value=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;if !valid(&value){return Err("Неверные метаданные Minecraft".into());}
            // A failed cache write must not prevent an otherwise valid online launch.
            if let Some(parent)=path.parent(){if std::fs::create_dir_all(parent).is_ok(){let part=path.with_extension(format!("tmp-{}",chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()));if std::fs::write(&part,&bytes).is_ok(){if std::fs::rename(&part,path).is_err(){let _=std::fs::remove_file(part);}}}}
            Ok(value)
        }.await;
        match result{Ok(value)=>return Ok((value,false)),Err(error)=>failures.push(error)}
    }
    if let Some(value)=cached{return Ok((value,true));}
    Err(format!("Не удалось получить метаданные Minecraft с серверов Mojang, а сохранённой копии пока нет. Повтори запуск после восстановления соединения. {}",failures.join("; ")))
}
pub async fn minecraft_manifest(root:&Path,version:Option<&str>)->Result<(Value,bool),String>{
    cached_metadata(&root.join("metadata/version-library.json"),&MANIFEST_URLS,None,version.is_some(),|v|v["versions"].as_array().is_some_and(|items|!items.is_empty()&&items.iter().all(|item|item["id"].is_string()&&item["url"].as_str().is_some_and(|url|url.starts_with("https://"))&&item["sha1"].as_str().is_some_and(|hash|hash.len()==40&&hash.bytes().all(|c|c.is_ascii_hexdigit())))&&version.is_none_or(|id|items.iter().any(|item|item["id"]==id)))).await
}
pub async fn minecraft_version(root:&Path,item:&Value)->Result<Value,String>{
    let address=item["url"].as_str().ok_or("Нет адреса метаданных Minecraft")?;let url=reqwest::Url::parse(address).map_err(|e|e.to_string())?;
    if url.scheme()!="https"||!url.host_str().is_some_and(|host|host.ends_with(".mojang.com")){return Err("Неверный адрес метаданных Minecraft".into());}
    let hash=item["sha1"].as_str().filter(|hash|hash.len()==40&&hash.bytes().all(|c|c.is_ascii_hexdigit())).ok_or("Нет SHA-1 метаданных Minecraft")?;
    let mut reserve=url.clone();if url.host_str()==Some("piston-meta.mojang.com"){let _=reserve.set_host(Some("launchermeta.mojang.com"));}else if url.host_str()==Some("launchermeta.mojang.com"){let _=reserve.set_host(Some("piston-meta.mojang.com"));}
    let urls=if reserve!=url{vec![address,reserve.as_str()]}else{vec![address]};
    cached_metadata(&root.join("metadata/minecraft").join(format!("{hash}.json")),&urls,Some(hash),true,|v|v["id"]==item["id"]&&!v["downloads"]["client"].is_null()).await.map(|(value,_)|value)
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

#[cfg(test)]
mod tests{
    use super::*;
    use tokio::io::{AsyncReadExt,AsyncWriteExt};
    fn folder()->std::path::PathBuf{let path=std::env::temp_dir().join(format!("kyro-metadata-{}",chrono::Utc::now().timestamp_nanos_opt().unwrap()));std::fs::create_dir_all(&path).unwrap();path}
    async fn serve(body:&str)->(String,tokio::task::JoinHandle<()>){
        let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();let body=body.to_owned();
        let task=tokio::spawn(async move{let(mut stream,_)=listener.accept().await.unwrap();let mut request=[0u8;4096];let _=stream.read(&mut request).await;let reply=format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body);stream.write_all(reply.as_bytes()).await.unwrap();});
        (format!("http://{address}/metadata"),task)
    }
    #[tokio::test]
    async fn failed_primary_uses_reserve_then_cache_without_network(){
        let root=folder();let file=root.join("manifest.json");let(url,task)=serve(r#"{"ok":true}"#).await;
        let(value,cached)=cached_metadata(&file,&["http://127.0.0.1:0/unavailable",&url],None,false,|v|v["ok"]==true).await.unwrap();assert_eq!(value["ok"],true);assert!(!cached);task.await.unwrap();
        let(_,cached)=cached_metadata(&file,&["http://127.0.0.1:0/unavailable"],None,false,|v|v["ok"]==true).await.unwrap();assert!(cached);
        let(_,cached)=cached_metadata(&file,&[],None,true,|v|v["ok"]==true).await.unwrap();assert!(cached);std::fs::remove_dir_all(root).unwrap();
    }
    #[tokio::test]
    async fn corrupt_cache_and_wrong_checksums_are_rejected(){
        let root=folder();let file=root.join("version.json");std::fs::write(&file,b"bad json").unwrap();assert!(cached_metadata(&file,&[],None,true,|_|true).await.is_err());
        let body=r#"{"id":"test"}"#;std::fs::write(&file,body).unwrap();let expected=format!("{:x}",Sha1::digest(b"different"));assert!(cached_metadata(&file,&[],Some(&expected),true,|_|true).await.is_err());
        let(url,task)=serve(body).await;assert!(cached_metadata(&file,&[&url],Some(&expected),false,|_|true).await.is_err());task.await.unwrap();std::fs::remove_dir_all(root).unwrap();
    }
    #[tokio::test]
    async fn verified_version_metadata_loads_from_cache(){
        let root=folder();let bytes=br#"{"id":"1.21.11","downloads":{"client":{"url":"https://example.invalid/client.jar"}}}"#;let hash=format!("{:x}",Sha1::digest(bytes));let dir=root.join("metadata/minecraft");std::fs::create_dir_all(&dir).unwrap();std::fs::write(dir.join(format!("{hash}.json")),bytes).unwrap();
        let item=serde_json::json!({"id":"1.21.11","url":"https://piston-meta.mojang.com/v1/packages/test/1.21.11.json","sha1":hash});assert_eq!(minecraft_version(&root,&item).await.unwrap()["id"],"1.21.11");std::fs::remove_dir_all(root).unwrap();
    }
    #[tokio::test]
    #[ignore="requires official Mojang endpoints"]
    async fn official_metadata_roundtrip(){
        let root=folder();let(manifest,_)=minecraft_manifest(&root,None).await.unwrap();let item=manifest["versions"].as_array().unwrap().iter().find(|item|item["id"]=="1.21.11").unwrap();let value=minecraft_version(&root,item).await.unwrap();assert_eq!(value["id"],"1.21.11");let(_,cached)=minecraft_manifest(&root,Some("1.21.11")).await.unwrap();assert!(cached);std::fs::remove_dir_all(root).unwrap();
    }
}
