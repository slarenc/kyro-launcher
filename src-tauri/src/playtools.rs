use crate::store::State;
use base64::Engine;
use serde_json::{json, Value};
use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

pub fn preset_values(id: &str) -> Result<Vec<(&'static str, &'static str)>, String> {
    let values = match id {
        "fps" => vec![
            ("renderDistance", "8"),
            ("simulationDistance", "5"),
            ("graphicsMode", "0"),
            ("fancyGraphics", "false"),
            ("particles", "2"),
            ("mipmapLevels", "2"),
            ("entityDistanceScaling", "0.75"),
            ("enableVsync", "false"),
            ("maxFps", "260"),
            ("renderClouds", "\"false\""),
        ],
        "beautiful" => vec![
            ("renderDistance", "16"),
            ("simulationDistance", "10"),
            ("graphicsMode", "1"),
            ("fancyGraphics", "true"),
            ("particles", "0"),
            ("mipmapLevels", "4"),
            ("entityDistanceScaling", "1.0"),
            ("enableVsync", "true"),
            ("maxFps", "120"),
            ("renderClouds", "\"true\""),
        ],
        "low" => vec![
            ("renderDistance", "4"),
            ("simulationDistance", "5"),
            ("graphicsMode", "0"),
            ("fancyGraphics", "false"),
            ("particles", "2"),
            ("mipmapLevels", "0"),
            ("entityDistanceScaling", "0.5"),
            ("enableVsync", "false"),
            ("maxFps", "60"),
            ("renderClouds", "\"false\""),
        ],
        _ => return Err("Неизвестный профиль настроек".into()),
    };
    Ok(values)
}
fn merge_options(original: &str, values: &[(&str, &str)]) -> String {
    let mut lines: Vec<String> = original.lines().map(str::to_string).collect();
    for (key, value) in values {
        let prefix = format!("{key}:");
        let mut found = false;
        lines.retain_mut(|line| {
            if line.starts_with(&prefix) {
                if found {
                    return false;
                }
                *line = format!("{key}:{value}");
                found = true;
            }
            true
        });
        if !found {
            lines.push(format!("{key}:{value}"));
        }
    }
    format!("{}\n", lines.join("\n"))
}
pub fn apply_to(dir: &Path, id: &str) -> Result<Value, String> {
    let values = preset_values(id)?;
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let options = dir.join("options.txt");
    let had = options.exists();
    if had
        && fs::symlink_metadata(&options)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
    {
        return Err("options.txt не должен быть ссылкой".into());
    }
    let old = if had {
        fs::read_to_string(&options).map_err(|e| e.to_string())?
    } else {
        String::new()
    };
    if old.len() > 2 * 1024 * 1024 {
        return Err("options.txt слишком большой".into());
    }
    let backup_dir = dir.join("kyro-backups");
    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    let backup = format!("options-{}.txt", chrono::Utc::now().timestamp_micros());
    fs::write(backup_dir.join(&backup), old.as_bytes()).map_err(|e| e.to_string())?;
    let new = merge_options(&old, &values);
    fs::write(dir.join("options.kyro-tmp"), new).map_err(|e| e.to_string())?;
    fs::rename(dir.join("options.kyro-tmp"), &options).map_err(|e| e.to_string())?;
    let meta = json!({"id":id,"backup":backup,"had_options":had});
    fs::write(
        dir.join("kyro-preset.json"),
        serde_json::to_vec(&meta).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(meta)
}

pub fn apply_graphics_preset(state: &State, id: String) -> Result<Value, String> {
    let busy = state.busy.lock().map_err(|e| e.to_string())?;
    if *busy {
        return Err("Закройте Minecraft перед сменой графики".into());
    }
    let settings = state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .settings
        .clone();
    apply_to(&state.instance_dir(&settings)?, &id)
}

pub fn restore_graphics_preset(state: &State) -> Result<(), String> {
    let busy = state.busy.lock().map_err(|e| e.to_string())?;
    if *busy {
        return Err("Закройте Minecraft перед восстановлением".into());
    }
    let s = state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .settings
        .clone();
    let dir = state.instance_dir(&s)?;
    let meta: Value = serde_json::from_slice(
        &fs::read(dir.join("kyro-preset.json"))
            .map_err(|_| "Нет применённого профиля для отката")?,
    )
    .map_err(|e| e.to_string())?;
    let name = crate::store::safe_segment(meta["backup"].as_str().ok_or("Нет резервной копии")?)?;
    let backup = dir.join("kyro-backups").join(name);
    let bytes = fs::read(backup).map_err(|e| e.to_string())?;
    let options = dir.join("options.txt");
    if options.exists() {
        fs::copy(
            &options,
            dir.join("kyro-backups").join(format!(
                "before-restore-{}.txt",
                chrono::Utc::now().timestamp_micros()
            )),
        )
        .map_err(|e| e.to_string())?;
    }
    if meta["had_options"] == true {
        fs::write(&options, bytes).map_err(|e| e.to_string())?;
    } else if options.exists() {
        fs::remove_file(&options).map_err(|e| e.to_string())?;
    }
    fs::remove_file(dir.join("kyro-preset.json")).map_err(|e| e.to_string())
}
fn png_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || !name.to_ascii_lowercase().ends_with(".png")
        || name.contains(['/', '\\', ':'])
        || name.chars().any(char::is_control)
    {
        Err("Недопустимое имя скриншота".into())
    } else {
        Ok(())
    }
}

pub fn list_screenshots(state: &State) -> Result<Value, String> {
    let s = state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .settings
        .clone();
    let dir = state.instance_dir(&s)?.join("screenshots");
    let mut images = vec![];
    if dir.exists() {
        for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if png_name(&name).is_err() || !entry.file_type().map_err(|e| e.to_string())?.is_file()
            {
                continue;
            }
            let meta = entry.metadata().map_err(|e| e.to_string())?;
            images.push(json!({"name":name,"size":meta.len(),"modified":meta.modified().ok().and_then(|t|t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d|d.as_millis() as u64).unwrap_or(0)}));
        }
    }
    images.sort_by_key(|v| std::cmp::Reverse(v["modified"].as_u64().unwrap_or(0)));
    images.truncate(2000);
    Ok(json!({"images":images,"folder":dir}))
}

pub fn screenshot_image(state: &State, name: String) -> Result<String, String> {
    png_name(&name)?;
    let s = state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .settings
        .clone();
    let path = state.instance_dir(&s)?.join("screenshots").join(name);
    let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 32 * 1024 * 1024 {
        return Err("Скриншот недоступен или больше 32 МБ".into());
    }
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err("Некорректный PNG".into());
    }
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}
pub fn tail(path: &Path, limit: u64) -> String {
    let Ok(mut file) = fs::File::open(path) else {
        return String::new();
    };
    let size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let _ = file.seek(SeekFrom::Start(size.saturating_sub(limit)));
    let mut bytes = Vec::new();
    let _ = file.take(limit).read_to_end(&mut bytes);
    String::from_utf8_lossy(&bytes).into_owned()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn options_keep_controls_and_server_settings() {
        let old="key_key.forward:key.keyboard.w\nrenderDistance:20\nlastServer:private.example\nrenderDistance:18\n";
        let new = merge_options(old, &preset_values("low").unwrap());
        assert!(new.contains("key_key.forward:key.keyboard.w"));
        assert!(new.contains("lastServer:private.example"));
        assert_eq!(new.matches("renderDistance:").count(), 1);
        assert!(new.contains("renderDistance:4\n"));
    }
    #[test]
    fn screenshot_names_stay_inside_profile() {
        assert!(png_name("../secret.png").is_err());
        assert!(png_name("2026-11-07_16.00.00.png").is_ok());
    }
}
