use crate::{game, store::State};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct FavoriteServer {
    pub id: String,
    pub name: String,
    pub address: String,
    #[serde(default)]
    pub icon: String,
}
pub fn address(raw: &str) -> Result<(String, u16), String> {
    let raw = raw.trim();
    if raw.is_empty()
        || raw.len() > 255
        || raw.chars().any(|c| c.is_whitespace() || c.is_control())
        || raw.contains(['/', '\\', '\0'])
    {
        return Err("Укажите адрес сервера, например mc.example.org:25565".into());
    }
    let (host, port) = if let Some(rest) = raw.strip_prefix('[') {
        let (host, tail) = rest.split_once(']').ok_or("Некорректный IPv6")?;
        host.parse::<std::net::Ipv6Addr>()
            .map_err(|_| "Некорректный IPv6")?;
        (
            host.to_string(),
            if tail.is_empty() {
                25565
            } else {
                tail.strip_prefix(':')
                    .ok_or("Некорректный порт")?
                    .parse::<u16>()
                    .map_err(|_| "Порт от 1 до 65535")?
            },
        )
    } else if raw.matches(':').count() > 1 {
        raw.parse::<std::net::Ipv6Addr>()
            .map_err(|_| "Некорректный IPv6")?;
        (raw.to_string(), 25565)
    } else if let Some((host, port)) = raw.rsplit_once(':') {
        (
            host.to_string(),
            port.parse::<u16>().map_err(|_| "Порт от 1 до 65535")?,
        )
    } else {
        (raw.to_string(), 25565)
    };
    if host.is_empty() || port == 0 || host.contains(['[', ']', '@', '#', '?']) {
        return Err("Некорректный адрес сервера".into());
    }
    Ok((host, port))
}
fn varint(mut n: u32) -> Vec<u8> {
    let mut bytes = vec![];
    loop {
        let mut b = (n & 0x7f) as u8;
        n >>= 7;
        if n != 0 {
            b |= 0x80;
        }
        bytes.push(b);
        if n == 0 {
            break;
        }
    }
    bytes
}
fn decode(bytes: &[u8], offset: &mut usize) -> Result<u32, String> {
    let mut n = 0;
    for shift in (0..35).step_by(7) {
        let b = *bytes.get(*offset).ok_or("Короткий пакет сервера")?;
        *offset += 1;
        n |= ((b & 127) as u32) << shift;
        if b & 128 == 0 {
            return Ok(n);
        }
    }
    Err("Слишком длинный VarInt".into())
}
async fn frame(stream: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut len = vec![];
    for _ in 0..5 {
        let b = stream.read_u8().await.map_err(|e| e.to_string())?;
        len.push(b);
        if b & 128 == 0 {
            break;
        }
    }
    let n = decode(&len, &mut 0)? as usize;
    if n > 1024 * 1024 {
        return Err("Слишком большой ответ сервера".into());
    }
    let mut bytes = vec![0; n];
    stream
        .read_exact(&mut bytes)
        .await
        .map_err(|e| e.to_string())?;
    Ok(bytes)
}
async fn send(stream: &mut TcpStream, body: &[u8]) -> Result<(), String> {
    let mut packet = varint(body.len() as u32);
    packet.extend_from_slice(body);
    stream.write_all(&packet).await.map_err(|e| e.to_string())
}
fn plain(v: &Value) -> String {
    if let Some(s) = v.as_str() {
        return s.into();
    }
    if let Some(a) = v.as_array() {
        return a.iter().map(plain).collect();
    }
    let mut s = v["text"].as_str().unwrap_or("").to_string();
    for child in v["extra"].as_array().into_iter().flatten() {
        s.push_str(&plain(child));
    }
    s.chars().take(500).collect()
}
async fn status(host: &str, port: u16) -> Result<Value, String> {
    let start = Instant::now();
    let endpoint = srv_endpoint(host, port).await;
    let mut stream = TcpStream::connect((endpoint.0.as_str(), endpoint.1))
        .await
        .map_err(|e| e.to_string())?;
    let mut hello = vec![0];
    hello.extend(varint(u32::MAX));
    hello.extend(varint(host.len() as u32));
    hello.extend(host.as_bytes());
    hello.extend(port.to_be_bytes());
    hello.push(1);
    send(&mut stream, &hello).await?;
    send(&mut stream, &[0]).await?;
    // Send ping without waiting: some vanilla servers defer status until ping arrives.
    let stamp = chrono::Utc::now().timestamp_millis();
    let mut ping = vec![1];
    ping.extend(stamp.to_be_bytes());
    send(&mut stream, &ping).await?;
    let packet = frame(&mut stream).await?;
    let mut offset = 0;
    if decode(&packet, &mut offset)? != 0 {
        return Err("Ожидался ответ статуса".into());
    }
    let len = decode(&packet, &mut offset)? as usize;
    let payload = packet
        .get(offset..offset + len)
        .ok_or("Неверная длина статуса")?;
    let data: Value = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    let latency = start.elapsed().as_millis();
    let mut skip = false;
    let description: String = plain(&data["description"])
        .chars()
        .filter(|c| {
            if skip {
                skip = false;
                return false;
            }
            if *c == '§' {
                skip = true;
                return false;
            }
            true
        })
        .collect();
    Ok(
        json!({"online":true,"icon":data["favicon"].as_str().map(|s| s.chars().filter(|c|!c.is_whitespace()).collect::<String>()).filter(|s| s.len() < 512*1024 && s.starts_with("data:image/png;base64,")),"latency":latency,"players":data["players"]["online"].as_u64(),"max":data["players"]["max"].as_u64(),"version":data["version"]["name"].as_str().unwrap_or(""),"description":description}),
    )
}

// Minecraft domains may point at an SRV endpoint instead of an A record.
async fn srv_endpoint(host: &str, port: u16) -> (String, u16) {
    #[cfg(windows)]
    if port == 25565 && host.parse::<std::net::IpAddr>().is_err() {
        let mut command = tokio::process::Command::new("powershell.exe");
        command.args(["-NoProfile", "-NonInteractive", "-Command", "$ErrorActionPreference='Stop'; Resolve-DnsName -Name $env:KYRO_SRV_QUERY -Type SRV -DnsOnly | Where-Object Type -eq SRV | Sort-Object Priority | Select-Object -First 1 NameTarget,Port | ConvertTo-Json -Compress"])
            .env("KYRO_SRV_QUERY",format!("_minecraft._tcp.{host}")).creation_flags(0x08000000).kill_on_drop(true);
        if let Ok(Ok(output)) = tokio::time::timeout(Duration::from_secs(2),command.output()).await {
            if let Ok(value) = serde_json::from_slice::<Value>(&output.stdout) {
                if let (Some(target),Some(port))=(value["NameTarget"].as_str(),value["Port"].as_u64().filter(|n|*n>0&&*n<=65535)) {
                    return (target.trim_end_matches('.').into(),port as u16);
                }
            }
        }
    }
    (host.into(),port)
}

pub async fn server_status(state: &State, id: String) -> Result<Value, String> {
    let server = state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .servers
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or("Сервер не найден")?;
    let (host, port) = address(&server.address)?;
    Ok(
        match tokio::time::timeout(Duration::from_secs(6), status(&host, port)).await {
            Ok(Ok(v)) => v,
            Ok(Err(e)) => json!({"online":false,"error":e}),
            Err(_) => json!({"online":false,"error":"Сервер не ответил за 6 секунд"}),
        },
    )
}

pub fn save_server(state: &State, name: String, address: String) -> Result<(), String> {
    let (host, port) = self::address(&address)?;
    let normalized = if host.contains(':') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    };
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err("Название: от 1 до 100 символов".into());
    }
    {
        let mut data = state.data.lock().map_err(|e| e.to_string())?;
        if data
            .servers
            .iter()
            .any(|s| s.address.eq_ignore_ascii_case(&normalized))
        {
            return Err("Сервер уже добавлен".into());
        }
        if data.servers.len() >= 3 {
            return Err("Можно сохранить до 3 серверов".into());
        }
        data.servers.push(FavoriteServer {
            icon: String::new(),
            id: chrono::Utc::now().timestamp_micros().to_string(),
            name: name.into(),
            address: normalized,
        });
    }
    state.save()
}

pub fn set_icon(state: &State, id: String, image: String) -> Result<(), String> {
    crate::customization::validate_image(&image)?;
    state.data.lock().map_err(|e|e.to_string())?.servers.iter_mut().find(|s|s.id==id).ok_or("Сервер не найден")?.icon=image;
    state.save()
}

pub fn remove_server(state: &State, id: String) -> Result<(), String> {
    {
        let mut data = state.data.lock().map_err(|e| e.to_string())?;
        data.servers.retain(|s| s.id != id);
    }
    state.save()
}

pub async fn launch_server(app: crate::app_context::AppContext, id: String) -> Result<(), String> {
    let address = {
        let state = app.state();
        let data = state.data.lock().map_err(|e| e.to_string())?;
        data.servers
            .iter()
            .find(|s| s.id == id)
            .ok_or("Сервер не найден")?
            .address
            .clone()
    };
    game::progress(&app, "Подключение к избранному серверу", 0.01);
    crate::run_launch(app, Some(address)).await
}
pub fn connect_args(version: &str, raw: &str) -> Result<Vec<String>, String> {
    let (host, port) = address(raw)?;
    let modern = version.split('.').next().is_some_and(|s| s != "1")
        || version
            .split('.')
            .nth(1)
            .and_then(|s| s.parse::<u32>().ok())
            .is_some_and(|v| v >= 20);
    if modern {
        Ok(vec![
            "--quickPlayMultiplayer".into(),
            if host.contains(':') {
                format!("[{host}]:{port}")
            } else {
                format!("{host}:{port}")
            },
        ])
    } else {
        Ok(vec![
            "--server".into(),
            host,
            "--port".into(),
            port.to_string(),
        ])
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    #[ignore = "requires live Minecraft servers"]
    async fn live_server_icons() {
        let mut values = serde_json::Map::new();
        for host in ["mc.aresmine.ru", "jux.virtusmine.fun"] {
            let value = tokio::time::timeout(Duration::from_secs(10),status(host,25565)).await;
            match value { Ok(Ok(v)) => { println!("{host}: online={}, icon={}",v["online"],v["icon"].as_str().is_some()); values.insert(host.into(),v); }, _=>println!("{host}: no response") }
        }
        std::fs::create_dir_all("../output/ui-refinement").unwrap();
        std::fs::write("../output/ui-refinement/live-servers.json",serde_json::to_vec(&values).unwrap()).unwrap();
    }
    #[test]
    fn limits_favorites_to_three_and_frees_removed_slot() {
        let root = std::env::temp_dir().join(format!("kyro-servers-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap()));
        std::fs::create_dir_all(&root).unwrap();
        let state = State { root: root.clone(), data: std::sync::Mutex::new(crate::store::Data::default()), busy: std::sync::Mutex::new(false) };
        for i in 0..3 { save_server(&state, format!("Server {i}"), format!("server{i}.example.org")).unwrap(); }
        assert!(save_server(&state, "Fourth".into(), "fourth.example.org".into()).unwrap_err().contains("3"));
        assert!(save_server(&state, "Duplicate".into(), "SERVER0.example.org:25565".into()).is_err());
        let id = state.data.lock().unwrap().servers[0].id.clone();
        remove_server(&state, id).unwrap();
        save_server(&state, "Replacement".into(), "replacement.example.org".into()).unwrap();
        assert_eq!(state.data.lock().unwrap().servers.len(), 3);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn addresses_and_launch_arguments() {
        assert_eq!(address("[::1]:25566").unwrap(), ("::1".into(), 25566));
        assert!(address("https://example.org").is_err());
        assert!(address("x:0").is_err());
        assert_eq!(
            connect_args("1.21.1", "example.org").unwrap()[0],
            "--quickPlayMultiplayer"
        );
        assert_eq!(
            connect_args("1.19.4", "example.org").unwrap()[0],
            "--server"
        );
    }
    #[tokio::test]
    async fn reads_real_status_packet() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            let (mut s, _) = listener.accept().await.unwrap();
            let handshake = frame(&mut s).await.unwrap();
            assert_eq!(handshake[0], 0);
            assert_eq!(frame(&mut s).await.unwrap(), vec![0]);
            let _ping = frame(&mut s).await.unwrap();
            let data=serde_json::to_vec(&json!({"players":{"online":3,"max":20},"version":{"name":"1.21.1"},"description":{"text":"Local test"}})).unwrap();
            let mut packet = vec![0];
            packet.extend(varint(data.len() as u32));
            packet.extend(data);
            send(&mut s, &packet).await.unwrap();
        });
        let result = status("127.0.0.1", port).await.unwrap();
        assert_eq!(result["players"], 3);
        assert_eq!(result["description"], "Local test");
    }
}
