//! Local, per-launcher IPC. No credentials are exported to game clients.
use crate::app_context::AppContext;
use serde_json::{json, Value};
use std::{fs, path::PathBuf, sync::OnceLock, time::Duration};

pub fn folder(app: &AppContext) -> PathBuf {
    static ID: OnceLock<String> = OnceLock::new();
    app.state().root.join("runtime/accounts").join(ID.get_or_init(|| format!("{}-{}",std::process::id(),chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default())))
}
fn write(path: PathBuf, value: &Value) -> Result<(), String> {
    let temp=path.with_extension("tmp");
    fs::write(&temp,serde_json::to_vec(value).map_err(|e|e.to_string())?).and_then(|_|fs::rename(temp,path)).map_err(|e|e.to_string())
}
pub fn start(app: AppContext) {
    tauri::async_runtime::spawn(async move {
        let root=folder(&app);
        if fs::create_dir_all(root.join("requests")).is_err(){return;}
        loop {
            if let Ok(snapshot)=crate::snapshot(app.state()) {
                let mut accounts:Vec<Value>=snapshot["data"]["offline_accounts"].as_array().into_iter().flatten().map(|a|json!({"key":format!("offline:{}",a["uuid"].as_str().unwrap_or_default()),"uuid":a["uuid"],"name":a["name"]})).collect();
                if !snapshot["microsoft_account"].is_null(){let a=&snapshot["microsoft_account"];accounts.push(json!({"key":"microsoft","uuid":a["uuid"],"name":a["name"]}));}
                let _=write(root.join("catalog.json"),&json!({"time":chrono::Utc::now().timestamp_millis(),"accounts":accounts}));
            }
            if let Ok(entries)=fs::read_dir(root.join("requests")) {
                for entry in entries.flatten().take(16) {
                    let path=entry.path();
                    if path.extension().and_then(|x|x.to_str())!=Some("json"){continue;}
                    let request=fs::read(&path).ok().filter(|b|b.len()<8192).and_then(|b|serde_json::from_slice::<Value>(&b).ok());
                    let _=fs::remove_file(&path);
                    if let Some(r)=request {
                        let result=handle(&app,&r).await;
                        let message=match result {Ok(())=>"Готово".to_string(),Err(e)=>e};
                        if let Some(name)=path.file_name(){let _=write(root.join(name),&json!({"message":message}));}
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    });
}
async fn handle(app:&AppContext,r:&Value)->Result<(),String>{
    let action=r["action"].as_str().unwrap_or_default();
    if action=="add" {return crate::offline_account(app.state(),"add".into(),r["name"].as_str().unwrap_or_default().into());}
    if action!="launch"{return Err("Неизвестная команда".into());}
    let address=r["server"].as_str().unwrap_or_default().trim();
    if !address.is_empty(){crate::servers::address(address)?;}
    crate::offline_account(app.state(),"select".into(),r["key"].as_str().unwrap_or_default().into())?;
    crate::run_launch(app.clone(),if address.is_empty(){None}else{Some(address.to_string())}).await
}
