//! Desktop Discord IPC. All blocking pipe operations stay off the UI/game threads.
use crate::store::{Settings, State};
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use serde_json::json;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    sync::{mpsc, Arc, Mutex, Weak},
    time::{Duration, Instant},
};

pub const APPLICATION_ID: &str = "1568628419514863656";
const ICON: &str = "https://cdn.discordapp.com/app-icons/1568628419514863656/72ec4899b6e52ca72b7c13e42c23b676.png?size=1024";

fn application_id(settings: &Settings) -> &str {
    let id = settings.discord_application_id.trim();
    if id.is_empty() {
        APPLICATION_ID
    } else {
        id
    }
}
enum Command {
    Game(Settings, Option<String>, PathBuf),
    Idle,
    Stop,
}
pub struct Presence {
    sender: mpsc::Sender<Command>,
    status: Arc<Mutex<String>>,
}
impl Presence {
    pub fn new(state: Weak<State>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let status = Arc::new(Mutex::new("Discord: ожидание настройки".into()));
        let report = status.clone();
        std::thread::spawn(move || worker(state, receiver, report));
        Self { sender, status }
    }
    pub fn status(&self) -> String {
        self.status.lock().map(|s| s.clone()).unwrap_or_default()
    }
    pub fn game_started(&self, settings: &Settings, server: Option<&str>, log: PathBuf) {
        let _ = self.sender.send(Command::Game(
            settings.clone(),
            server.map(str::to_owned),
            log,
        ));
    }
    pub fn game_finished(&self) {
        let _ = self.sender.send(Command::Idle);
    }
    pub fn shutdown(&self) {
        let _ = self.sender.send(Command::Stop);
    }
}
impl Drop for Presence {
    fn drop(&mut self) {
        self.shutdown();
    }
}

struct Game {
    version: String,
    loader: String,
    server: Option<String>,
    tail: Tail,
}
#[derive(Default)]
struct Tail {
    path: PathBuf,
    offset: u64,
    pending: String,
}
impl Tail {
    fn update(&mut self, server: &mut Option<String>) {
        let Ok(mut file) = File::open(&self.path) else {
            return;
        };
        let Ok(meta) = file.metadata() else {
            return;
        };
        if meta.len() < self.offset {
            self.offset = 0;
            self.pending.clear();
        }
        if file.seek(SeekFrom::Start(self.offset)).is_err() {
            return;
        }
        let mut bytes = Vec::new();
        if file.take(256 * 1024).read_to_end(&mut bytes).is_err() {
            return;
        }
        self.offset += bytes.len() as u64;
        self.pending.push_str(&String::from_utf8_lossy(&bytes));
        while let Some(end) = self.pending.find('\n') {
            let line = self.pending[..end].to_owned();
            self.pending.drain(..=end);
            apply_log_line(&line, server);
        }
        if self.pending.len() > 8192 {
            self.pending.clear();
        }
    }
}
fn apply_log_line(line: &str, server: &mut Option<String>) {
    // Only vanilla logger messages, never chat text, become presence data.
    let Some((prefix, message)) = line.split_once("]: ") else {
        return;
    };
    if !(prefix.contains("Render thread/INFO") || prefix.contains("Client thread/INFO")) {
        return;
    }
    if let Some(address) = message.strip_prefix("Connecting to ") {
        let address = address.trim();
        let address = if let Some((host, port)) = address.rsplit_once(", ") {
            if port == "25565" {
                host.to_string()
            } else {
                format!("{host}:{port}")
            }
        } else {
            address.to_string()
        };
        if address.len() <= 110 && crate::servers::address(&address).is_ok() {
            *server = Some(address);
        }
    } else if [
        "Disconnected",
        "Connection lost",
        "Failed to connect",
        "Stopping!",
        "Starting integrated minecraft server",
        "Connecting to local server",
    ]
    .iter()
    .any(|p| message.starts_with(p))
    {
        *server = None;
    }
}
fn lines(game: Option<&Game>, show_server: bool) -> (String, String) {
    match game {
        None => ("В лаунчере".into(), "Выбирает, во что поиграть".into()),
        Some(g) => (
            format!("Minecraft {} · {}", g.version, g.loader),
            if show_server {
                g.server.as_ref().map(|s| format!("Сервер: {s}"))
            } else {
                None
            }
            .unwrap_or_else(|| "В игре".into()),
        ),
    }
}
fn valid_id(id: &str) -> bool {
    (17..=20).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_digit())
}
fn connect(id: &str) -> Result<DiscordIpcClient, String> {
    let mut client = DiscordIpcClient::new(id);
    client.connect_ipc().map_err(|e| e.to_string())?;
    client
        .send(json!({"v":1,"client_id":id}), 0)
        .map_err(|e| e.to_string())?;
    let (op, response) = client.recv().map_err(|e| e.to_string())?;
    if op != 1 || response["evt"] != "READY" {
        return Err("Discord отклонил Application ID".into());
    }
    Ok(client)
}
fn clear(client: &mut Option<DiscordIpcClient>) {
    if let Some(mut c) = client.take() {
        let _ = c.clear_activity();
        let _ = c.close();
    }
}
fn worker(state: Weak<State>, receiver: mpsc::Receiver<Command>, status: Arc<Mutex<String>>) {
    let started = chrono::Utc::now().timestamp();
    let mut game: Option<Game> = None;
    let mut client = None;
    let mut client_id = String::new();
    let mut last_lines = None;
    let mut last_sent = Instant::now() - Duration::from_secs(60);
    loop {
        match receiver.recv_timeout(Duration::from_secs(2)) {
            Ok(Command::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                clear(&mut client);
                return;
            }
            Ok(Command::Idle) => game = None,
            Ok(Command::Game(s, server, path)) => {
                game = Some(Game {
                    version: s.version,
                    loader: s.loader,
                    server,
                    tail: Tail {
                        path,
                        ..Default::default()
                    },
                })
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        let Some(state) = state.upgrade() else {
            clear(&mut client);
            return;
        };
        let Ok(settings) = state.data.lock().map(|d| d.settings.clone()) else {
            continue;
        };
        drop(state);
        let id = application_id(&settings);
        if !settings.discord_enabled || !valid_id(id) {
            clear(&mut client);
            last_lines = None;
            if let Ok(mut s) = status.lock() {
                *s = if !settings.discord_enabled {
                    "Discord Rich Presence выключен".into()
                } else if id.is_empty() {
                    "Подключение к Discord пока не настроено".into()
                } else {
                    "Не удалось подключить активность Discord".into()
                };
            }
            continue;
        }
        if let Some(g) = &mut game {
            g.tail.update(&mut g.server);
        }
        if client_id != id {
            clear(&mut client);
            client_id = id.into();
            last_lines = None;
            last_sent = Instant::now() - Duration::from_secs(60);
        }
        let current = lines(game.as_ref(), settings.discord_show_server);
        if last_sent.elapsed() < Duration::from_secs(15)
            || (last_lines.as_ref() == Some(&current)
                && last_sent.elapsed() < Duration::from_secs(60))
        {
            continue;
        }
        last_sent = Instant::now();
        if client.is_none() {
            match connect(id) {
                Ok(c) => client = Some(c),
                Err(_) => {
                    if let Ok(mut s) = status.lock() {
                        *s = "Нет подключения к Discord. Запусти настольное приложение; подключение повторится автоматически.".into();
                    }
                    continue;
                }
            }
        }
        let c = client.as_mut().unwrap();
        let payload = activity::Activity::new()
            .name("KyroLauncher")
            .buttons(vec![activity::Button::new(
                "Скачать KYRO",
                "https://kyrolauncher.pages.dev/#download",
            )])
            .details(current.0.clone())
            .state(current.1.clone())
            .timestamps(activity::Timestamps::new().start(started))
            .assets(
                activity::Assets::new()
                    .large_image(ICON)
                    .large_text("KyroLauncher"),
            );
        let sent = c
            .set_activity(payload)
            .map_err(|e| e.to_string())
            .and_then(|_| {
                let (op, reply) = c.recv().map_err(|e| e.to_string())?;
                if op != 1 || reply["evt"] == "ERROR" {
                    Err("Discord отклонил активность".into())
                } else {
                    Ok(())
                }
            });
        if let Ok(mut s) = status.lock() {
            *s = if sent.is_ok() {
                "Discord Rich Presence подключён".into()
            } else {
                "Discord отключён. Переподключаемся автоматически…".into()
            };
        }
        if sent.is_ok() {
            last_lines = Some(current);
        } else {
            client = None;
            last_lines = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn presence_fields_and_server_tracking() {
        assert_eq!(
            lines(None, true),
            ("В лаунчере".into(), "Выбирает, во что поиграть".into())
        );
        let mut game = Game {
            version: "1.21.11".into(),
            loader: "Fabric".into(),
            server: None,
            tail: Tail::default(),
        };
        apply_log_line(
            "[12:30:00] [Render thread/INFO]: Connecting to mc.aresmine.ru, 25565",
            &mut game.server,
        );
        assert_eq!(
            lines(Some(&game), true),
            (
                "Minecraft 1.21.11 · Fabric".into(),
                "Сервер: mc.aresmine.ru".into()
            )
        );
        assert_eq!(lines(Some(&game), false).1, "В игре");
        apply_log_line(
            "[12:30:00] [Render thread/INFO]: [CHAT] Connecting to fake.ru, 25565",
            &mut game.server,
        );
        assert_eq!(game.server.as_deref(), Some("mc.aresmine.ru"));
        apply_log_line(
            "[12:30:00] [Render thread/INFO]: Disconnected from server",
            &mut game.server,
        );
        assert!(game.server.is_none());
        assert!(valid_id("123456789012345678"));
        assert!(!valid_id("not-a-token"));
    }
    #[test]
    fn tail_reads_only_new_log_lines_and_handles_truncation() {
        let path = std::env::temp_dir().join(format!(
            "kyro-presence-{}.log",
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        std::fs::write(
            &path,
            "[00:00:00] [Render thread/INFO]: Connecting to test.example, 25566\n",
        )
        .unwrap();
        let mut tail = Tail {
            path: path.clone(),
            ..Default::default()
        };
        let mut server = None;
        tail.update(&mut server);
        assert_eq!(server.as_deref(), Some("test.example:25566"));
        server = None;
        tail.update(&mut server);
        assert!(server.is_none());
        std::fs::write(
            &path,
            "[00:00:00] [Render thread/INFO]: Connecting to a.ru, 25565\n",
        )
        .unwrap();
        tail.update(&mut server);
        assert_eq!(server.as_deref(), Some("a.ru"));
        let _ = std::fs::remove_file(path);
    }
}
