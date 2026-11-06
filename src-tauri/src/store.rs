use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Mutex};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub username: String,
    pub version: String,
    pub loader: String,
    pub memory_gb: u32,
    pub java_path: String,
    pub jvm_args: String,
    pub download_threads: usize,
    pub launch_behavior: String,
    pub language: String,
    pub theme: String,
    pub accent: String,
    pub mod_notifications: bool,
    pub system_notifications: bool,
    pub microsoft_client_id: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            username: "Nova".into(),
            version: "1.21.1".into(),
            loader: "Fabric".into(),
            memory_gb: 6,
            java_path: detect_java(),
            jvm_args: "-XX:+UseG1GC -XX:MaxGCPauseMillis=50".into(),
            download_threads: 8,
            launch_behavior: "hide".into(),
            language: "ru".into(),
            theme: "black".into(),
            accent: "#a185cf".into(),
            mod_notifications: true,
            system_notifications: true,
            microsoft_client_id: String::new(),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub started: String,
    pub seconds: u64,
    pub version: String,
    pub loader: String,
    pub exit_code: Option<i32>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Notice {
    pub id: String,
    pub title: String,
    pub body: String,
    pub time: String,
    pub unread: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Installed {
    pub project_id: String,
    pub version_id: String,
    pub name: String,
    pub filename: String,
    pub kind: String,
    pub game_version: String,
    pub loader: String,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Data {
    pub settings: Settings,
    pub sessions: Vec<Session>,
    pub notices: Vec<Notice>,
    pub installed: Vec<Installed>,
    pub offline_accounts: Vec<OfflineAccount>,
    pub selected_account: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct OfflineAccount {
    pub name: String,
    pub uuid: String,
}
impl Data {
    pub fn add_offline(&mut self, name: &str) -> Result<(), String> {
        let name = name.trim();
        if !(3..=16).contains(&name.len())
            || !name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
        {
            return Err("Ник: от 3 до 16 латинских букв, цифр или символов _".into());
        }
        if self
            .offline_accounts
            .iter()
            .any(|a| a.name.eq_ignore_ascii_case(name))
        {
            return Err("Аккаунт с таким ником уже добавлен".into());
        }
        use md5::{Digest, Md5};
        let mut bytes: [u8; 16] = Md5::digest(format!("OfflinePlayer:{name}").as_bytes()).into();
        bytes[6] = (bytes[6] & 0x0f) | 0x30;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        let uuid = uuid::Uuid::from_bytes(bytes).simple().to_string();
        self.selected_account = Some(format!("offline:{uuid}"));
        self.offline_accounts.push(OfflineAccount {
            name: name.into(),
            uuid,
        });
        Ok(())
    }
    pub fn select_offline(&mut self, id: &str) -> Result<(), String> {
        if !self
            .offline_accounts
            .iter()
            .any(|a| format!("offline:{}", a.uuid) == id)
        {
            return Err("Аккаунт не найден".into());
        }
        self.selected_account = Some(id.into());
        Ok(())
    }
    pub fn remove_offline(&mut self, id: &str) -> Result<(), String> {
        if !self
            .offline_accounts
            .iter()
            .any(|a| format!("offline:{}", a.uuid) == id)
        {
            return Err("Аккаунт не найден".into());
        }
        self.offline_accounts
            .retain(|a| format!("offline:{}", a.uuid) != id);
        if self.selected_account.as_deref() == Some(id) {
            self.selected_account = self
                .offline_accounts
                .first()
                .map(|a| format!("offline:{}", a.uuid));
        }
        Ok(())
    }
}
pub struct State {
    pub root: PathBuf,
    pub data: Mutex<Data>,
    pub busy: Mutex<bool>,
}
impl State {
    pub fn new() -> Self {
        let root = std::env::var_os("KYRO_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let portable = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|p| p.join("data")));
                if let Some(path) = portable.filter(|p| p.join("state.json").exists()) {
                    return path;
                }
                dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("KyroLauncher")
            });
        let data = fs::read(root.join("state.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        Self {
            root,
            data: Mutex::new(data),
            busy: Mutex::new(false),
        }
    }
    pub fn save(&self) -> Result<(), String> {
        fs::create_dir_all(&self.root).map_err(|e| e.to_string())?;
        let data = self.data.lock().map_err(|e| e.to_string())?;
        let bytes = serde_json::to_vec_pretty(&*data).map_err(|e| e.to_string())?;
        let temporary = self.root.join("state.json.tmp");
        fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
        fs::rename(temporary, self.root.join("state.json")).map_err(|e| e.to_string())
    }
    pub fn notify(&self, title: &str, body: &str) {
        if let Ok(mut d) = self.data.lock() {
            d.notices.insert(
                0,
                Notice {
                    id: Utc::now().timestamp_micros().to_string(),
                    title: title.into(),
                    body: body.into(),
                    time: Utc::now().to_rfc3339(),
                    unread: true,
                },
            );
            d.notices.truncate(200);
        }
        let _ = self.save();
    }
    pub fn game_dir(&self, version: &str, loader: &str) -> PathBuf {
        self.root
            .join("instances")
            .join(format!("{version}-{loader}"))
    }
    pub fn target_path(&self, target: &str, settings: &Settings) -> Result<PathBuf, String> {
        match target {
            "launcher" => std::env::current_exe()
                .map_err(|e| e.to_string())?
                .parent()
                .map(PathBuf::from)
                .ok_or("Не найдена папка EXE".into()),
            "root" => Ok(self.root.clone()),
            "instance" => {
                safe_segment(&settings.version)?;
                safe_segment(&settings.loader)?;
                Ok(self.game_dir(&settings.version, &settings.loader))
            }
            "log" => Ok(self.root.join("game.log")),
            "core-log" => Ok(self.root.join("core-install.log")),
            _ => Err("Неизвестный путь".into()),
        }
    }
}
fn detect_java() -> String {
    for root in [
        "C:/Program Files/Eclipse Adoptium",
        "C:/Program Files/Java",
        "C:/Program Files/Microsoft",
    ] {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                let java = entry.path().join("bin/java.exe");
                if (name.starts_with("jdk-21") || name.starts_with("jdk21")) && java.exists() {
                    return java.to_string_lossy().into_owned();
                }
            }
        }
    }
    "java".into()
}
pub fn safe_segment(s: &str) -> Result<&str, String> {
    if s.is_empty()
        || !s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-+".contains(c))
        || s == "."
        || s == ".."
    {
        Err("Недопустимое имя файла или версии".into())
    } else {
        Ok(s)
    }
}

#[cfg(test)]
mod account_tests {
    use super::*;
    #[test]
    fn folder_targets_are_distinct() {
        let state = State {
            root: PathBuf::from("C:/Kyro/data"),
            data: Mutex::new(Data::default()),
            busy: Mutex::new(false),
        };
        let s = Settings::default();
        let profile = state.target_path("instance", &s).unwrap();
        assert!(profile.starts_with(&state.root));
        assert_ne!(profile, state.target_path("root", &s).unwrap());
        assert_ne!(profile, state.target_path("launcher", &s).unwrap());
        assert_eq!(profile.file_name().unwrap(), "1.21.1-Fabric");
        let mut quilt = s;
        quilt.loader = "Quilt".into();
        assert_ne!(profile, state.target_path("instance", &quilt).unwrap());
    }
    #[test]
    fn offline_identity_is_java_compatible() {
        let mut d = Data::default();
        d.add_offline("Notch").unwrap();
        assert_eq!(
            d.offline_accounts[0].uuid,
            "b50ad385829d3141a2167e7d7539ba7f"
        );
        assert!(d.add_offline("notch").is_err());
        for invalid in [
            "ab",
            "has space",
            "../oops",
            "кириллица",
            "12345678901234567",
        ] {
            assert!(d.add_offline(invalid).is_err());
        }
    }
    #[test]
    fn selection_removal_and_persistence() {
        let mut d = Data::default();
        d.add_offline("Alice").unwrap();
        let first = d.selected_account.clone().unwrap();
        d.add_offline("Bob").unwrap();
        let second = d.selected_account.clone().unwrap();
        d.select_offline(&first).unwrap();
        assert_eq!(d.selected_account.as_deref(), Some(first.as_str()));
        d.remove_offline(&first).unwrap();
        assert_eq!(d.selected_account.as_deref(), Some(second.as_str()));
        let mut saved: Data = serde_json::from_str(&serde_json::to_string(&d).unwrap()).unwrap();
        assert_eq!(saved.offline_accounts[0].name, "Bob");
        saved.remove_offline(&second).unwrap();
        assert!(saved.selected_account.is_none());
        let legacy: Data = serde_json::from_str("{\"settings\":{\"version\":\"1.21.1\"}}").unwrap();
        assert!(legacy.offline_accounts.is_empty());
    }
}
