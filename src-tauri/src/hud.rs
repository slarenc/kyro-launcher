//! Bundled, optional client-side panels; installed only into a compatible active profile.
use crate::store::State;
use serde_json::{json, Value};
use std::{fs, io::Read, path::Path};

const HUD: &[u8] = include_bytes!("../resources/kyro-hud-1.0.0.jar");
const API: &[u8] = include_bytes!("../resources/fabric-api-0.141.6+1.21.11.jar");
const MEDIA: &[u8] = include_bytes!("../resources/kyro-media.exe");
const NAME: &str = "kyro-hud-1.0.0.jar";

fn mod_id(path: &Path) -> Option<String> {
    let mut zip = zip::ZipArchive::new(fs::File::open(path).ok()?).ok()?;
    let file = zip.by_name("fabric.mod.json").ok()?;
    let mut body = String::new();
    file.take(256 * 1024).read_to_string(&mut body).ok()?;
    serde_json::from_str::<Value>(&body).ok()?["id"].as_str().map(str::to_owned)
}
fn supported(version: &str, loader: &str) -> bool { version == "1.21.11" && loader == "Fabric" }
pub fn status(state: &State) -> Result<Value, String> {
    let s = state.data.lock().map_err(|e| e.to_string())?.settings.clone();
    let dir = state.instance_dir(&s)?.join("mods");
    let installed = fs::read_dir(dir).ok().into_iter().flatten().flatten().any(|entry| {
        entry.path().extension().is_some_and(|e| e == "jar") && mod_id(&entry.path()).as_deref() == Some("kyro_hud")
    });
    Ok(json!({"supported":supported(&s.version,&s.loader),"installed":installed,"version":s.version,"loader":s.loader}))
}
pub fn install(state: &State) -> Result<(), String> {
    let mut busy = state.busy.lock().map_err(|e| e.to_string())?;
    if *busy { return Err("Сначала закрой Minecraft и дождись завершения загрузок".into()); }
    *busy = true;
    drop(busy);
    let result = (|| {
        let s = state.data.lock().map_err(|e| e.to_string())?.settings.clone();
        if !supported(&s.version, &s.loader) { return Err("Панели KYRO поддерживают Fabric 1.21.11. Выбери этот профиль; другие версии не изменены.".into()); }
        let profile = state.instance_dir(&s)?;
        let dir = profile.join("mods");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        if !fs::canonicalize(&dir).map_err(|e| e.to_string())?.starts_with(fs::canonicalize(&profile).map_err(|e| e.to_string())?) {
            return Err("Папка модов ведёт за пределы профиля".into());
        }
        let mut api_exists = false;
        for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if !entry.file_type().map_err(|e| e.to_string())?.is_file() { continue; }
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".jar") && !name.ends_with(".jar.disabled") { continue; }
            match mod_id(&path).as_deref() {
                Some("fabric-api") if name.ends_with(".disabled") => return Err("Fabric API отключён. Включи его в менеджере модов.".into()),
                Some("fabric-api") => api_exists = true,
                Some("kyro_hud") => return Err("Панели KYRO уже есть в профиле. Включить или удалить их можно в менеджере модов.".into()),
                _ => {}
            }
        }
        let config = profile.join("config");
        fs::create_dir_all(&config).map_err(|e| e.to_string())?;
        if !fs::canonicalize(&config).map_err(|e| e.to_string())?.starts_with(fs::canonicalize(&profile).map_err(|e| e.to_string())?) {
            return Err("Папка настроек ведёт за пределы профиля".into());
        }
        let bridge = config.join("kyro-media.exe");
        let bridge_exists = bridge.symlink_metadata().is_ok();
        if bridge_exists && (!bridge.symlink_metadata().map_err(|e|e.to_string())?.file_type().is_file() || fs::read(&bridge).map_err(|e|e.to_string())? != MEDIA) {
            return Err("В config уже есть другой файл kyro-media.exe. Он не изменён.".into());
        }
        // create_new avoids overwriting user-owned files or symlinks with matching names.
        use std::io::Write;
        let mut created = Vec::new();
        let files: Vec<(&str,&[u8])> = if api_exists { vec![(NAME,HUD)] } else { vec![("fabric-api-0.141.6+1.21.11.jar",API),(NAME,HUD)] };
        for (name,bytes) in files {
            let path = dir.join(name);
            let result = fs::OpenOptions::new().write(true).create_new(true).open(&path).and_then(|mut f| {
                created.push(path.clone());
                f.write_all(bytes)
            });
            if let Err(e) = result { for path in created { let _ = fs::remove_file(path); } return Err(e.to_string()); }
        }
        if !bridge_exists {
        let helper_result = fs::OpenOptions::new().write(true).create_new(true).open(&bridge).and_then(|mut f| { created.push(bridge.clone()); f.write_all(MEDIA) });
        if let Err(e) = helper_result { for path in created { let _=fs::remove_file(path); } return Err(e.to_string()); }
        }
        Ok(())
    })();
    *state.busy.lock().map_err(|e| e.to_string())? = false;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installs_only_into_compatible_profile_and_can_reinstall() {
        let root=std::env::temp_dir().join(format!("kyro-hud-test-{}",chrono::Utc::now().timestamp_micros()));
        let mut data=crate::store::Data::default();
        data.settings.version="1.21.11".into();data.settings.loader="Vanilla".into();
        let state=State{root:root.clone(),data:std::sync::Mutex::new(data),busy:std::sync::Mutex::new(false)};
        assert!(install(&state).is_err());assert!(!root.exists());
        state.data.lock().unwrap().settings.loader="Fabric".into();
        *state.busy.lock().unwrap()=true;assert!(install(&state).is_err());*state.busy.lock().unwrap()=false;
        install(&state).unwrap();assert_eq!(status(&state).unwrap()["installed"],true);
        let settings=state.data.lock().unwrap().settings.clone();let profile=state.instance_dir(&settings).unwrap();
        assert_eq!(fs::read(profile.join("config/kyro-media.exe")).unwrap(),MEDIA);
        assert!(install(&state).is_err());
        fs::remove_file(profile.join("mods").join(NAME)).unwrap();
        install(&state).unwrap();
        assert_eq!(fs::read_dir(profile.join("mods")).unwrap().count(),2);
        fs::remove_file(profile.join("mods").join(NAME)).unwrap();
        fs::write(profile.join("mods").join(NAME),b"user file").unwrap();
        assert!(install(&state).is_err());assert_eq!(fs::read(profile.join("mods").join(NAME)).unwrap(),b"user file");
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn bundled_mods_have_correct_metadata() {
        for (bytes,id) in [(HUD,"kyro_hud"),(API,"fabric-api")] {
            let mut zip=zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
            let metadata: Value=serde_json::from_reader(zip.by_name("fabric.mod.json").unwrap()).unwrap();
            assert_eq!(metadata["id"],id);
        }
        assert!(supported("1.21.11","Fabric"));
        assert!(!supported("1.21.1","Fabric"));
        assert!(!supported("1.21.11","Vanilla"));
    }
}
