use crate::{playtools::tail, store::State};
use serde_json::{json, Value};
use std::{fs, path::Path};

pub fn start(app: &crate::app_context::AppContext) -> Option<(std::path::PathBuf, i64)> {
    let state = app.state();
    let s = state.data.lock().ok()?.settings.clone();
    let dir = state.instance_dir(&s).ok()?.join("kyro-diagnostics");
    fs::create_dir_all(&dir).ok()?;
    let time = chrono::Utc::now().timestamp_millis();
    let _ = fs::write(
        dir.join("last-launch.json"),
        serde_json::to_vec(
            &json!({"started":time,"version":s.version,"loader":s.loader,"ok":null}),
        )
        .ok()?,
    );
    Some((dir, time))
}
pub fn finish(
    app: &crate::app_context::AppContext,
    record: Option<(std::path::PathBuf, i64)>,
    result: &Result<(), String>,
) {
    let Some((dir, time)) = record else {
        return;
    };
    let state = app.state();
    let path = dir.join("last-launch.json");
    let mut meta: Value = fs::read(&path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or(json!({"started":time}));
    let cancelled = result
        .as_ref()
        .err()
        .is_some_and(|e| e == crate::launch_control::CANCELLED);
    meta["cancelled"] = json!(cancelled);
    meta["ok"] = if cancelled {
        Value::Null
    } else {
        json!(result.is_ok())
    };
    meta["error"] = if cancelled {
        Value::Null
    } else {
        json!(result.as_ref().err())
    };
    let _ = fs::write(path, serde_json::to_vec(&meta).unwrap_or_default());
    let game = state.root.join("game.log");
    let current = game
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .is_some_and(|d| d.as_millis() as i64 >= time);
    if current {
        let _ = fs::write(dir.join("game-tail.log"), tail(&game, 1024 * 1024));
    }
}
fn finding(title: &str, detail: &str, action: &str) -> Value {
    json!({"title":title,"detail":detail,"action":action})
}
fn mentioned_mods(text: &str) -> Vec<String> {
    let mut names = vec![];
    for line in text.lines() {
        if let Some(at) = line.find("from mod ") {
            let name: String = line[at + 9..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || "_.-".contains(*c))
                .take(80)
                .collect();
            if !name.is_empty() && !names.contains(&name) {
                names.push(name);
            }
        }
        if let Some(at) = line.find("Mod File:") {
            let rest = line[at + 9..].trim();
            if let Some(end) = rest.find(".jar") {
                let name = rest[..end + 4]
                    .rsplit(['/', '\\'])
                    .next()
                    .unwrap_or("")
                    .to_string();
                if !name.is_empty() && !names.contains(&name) {
                    names.push(name);
                }
            }
        }
        if names.len() >= 5 {
            break;
        }
    }
    names
}
pub fn analyze(text: &str) -> Vec<Value> {
    let lower = text.to_lowercase();
    let mut findings = vec![];
    if lower.contains("outofmemoryerror")
        || lower.contains("java heap space")
        || lower.contains("could not reserve enough space")
    {
        findings.push(finding("Не хватило памяти","Java сообщила об ошибке выделения памяти. Проверь объём RAM для игры; если резервирование не удалось, уменьши его и закрой другие приложения.","java"));
    }
    if lower.contains("unsupportedclassversionerror")
        || lower.contains("class file version")
        || lower.contains("java не найдена")
        || lower.contains("нужна java")
        || lower.contains("нужен java")
    {
        findings.push(finding("Неподходящая Java","Клиенту или моду нужна другая версия Java. Включи автоматический подбор и повтори запуск.","java"));
    }
    if lower.contains("incompatible mods found")
        || lower.contains("mod resolution encountered")
        || lower.contains("missing mandatory dependencies")
        || lower.contains("requires version")
        || lower.contains("missing or unsupported mandatory dependencies")
    {
        findings.push(finding("Не совпали моды или зависимости","В сообщении загрузчика есть несовместимые версии или отсутствующие зависимости. Проверь состав модов текущего профиля.","compatibility"));
    }
    if lower.contains("mixinapplyerror")
        || lower.contains("mixintransformererror")
        || lower.contains("mixin apply failed")
        || lower.contains("invalidmixinexception")
    {
        let mods = mentioned_mods(text);
        let detail = if mods.is_empty() {
            "Один из модов не смог изменить код игры. Это возможный конфликт или неподходящая версия мода; точный участник обычно указан рядом с ошибкой в логе.".to_string()
        } else {
            format!("В логе упомянуты: {}. Проверь версии этих модов и их зависимости. Это возможные участники конфликта, а не гарантированная причина.",mods.join(", "))
        };
        findings.push(finding("Ошибка применения Mixin", &detail, "mods"));
    }
    if lower.contains("glfw error")
        || lower.contains("failed to create window")
        || lower.contains("pixel format not accelerated")
    {
        findings.push(finding("Ошибка графической системы","Minecraft не смог создать графическое окно. Проверь драйвер видеокарты и поддержку требуемой версии OpenGL.","log"));
    }
    if lower.contains("could not find or load main class")
        || lower.contains("не найдена локальная библиотека")
        || lower.contains("контрольная сумма")
    {
        findings.push(finding("Не удалось загрузить файлы клиента","Есть признак отсутствующего или повреждённого файла. Запусти проверку Minecraft; если ошибка относится к моду, переустанови его.","verify"));
    }
    findings
}
fn last_crash(dir: &Path, started: i64) -> Option<std::path::PathBuf> {
    fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|e| {
            let time = e
                .metadata()
                .ok()?
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_millis() as i64;
            if time < started.saturating_sub(2000) {
                return None;
            }
            Some((time, e.path()))
        })
        .max_by_key(|(t, _)| *t)
        .map(|(_, p)| p)
}

pub fn diagnose_launch(state: &State) -> Result<Value, String> {
    let s = state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .settings
        .clone();
    let profile = state.instance_dir(&s)?;
    let dir = profile.join("kyro-diagnostics");
    let meta: Value = fs::read(dir.join("last-launch.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or(Value::Null);
    let started = meta["started"].as_i64().unwrap_or(0);
    if meta["cancelled"] == true {
        return Ok(
            json!({"title":"Запуск отменён пользователем","findings":[],"evidence":[],"running":false,"ok":false}),
        );
    }
    let crash = last_crash(&profile.join("crash-reports"), started);
    let mut text = meta["error"].as_str().unwrap_or("").to_string();
    if !meta.is_null()
        && dir
            .join("game-tail.log")
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .is_some_and(|d| d.as_millis() as i64 >= started)
    {
        text.push_str(&tail(&dir.join("game-tail.log"), 1024 * 1024));
    }
    if let Some(p) = &crash {
        text.push_str(&tail(p, 512 * 1024));
    }
    let running =
        !meta.is_null() && meta["ok"].is_null() && *state.busy.lock().map_err(|e| e.to_string())?;
    let ok = meta["ok"] == true && crash.is_none();
    let findings = if ok || running {
        vec![]
    } else {
        analyze(&text)
    };
    let title = if running {
        "Игра или подготовка ещё выполняется"
    } else if ok {
        "Последний запуск завершился штатно"
    } else if meta.is_null() && crash.is_none() {
        "Пока нет данных о запуске"
    } else if !findings.is_empty() {
        "Найдены признаки причины вылета"
    } else {
        "Причина автоматически не определена"
    };
    // Only short matched lines are exposed. Full local logs stay in the profile.
    let evidence: Vec<_> = text
        .lines()
        .filter(|l| {
            let lower = l.to_lowercase();
            lower.contains("caused by:")
                || lower.contains("exception:")
                || lower.contains("requires version")
                || lower.contains("incompatible mods")
        })
        .filter(|l| {
            !l.to_lowercase().contains("accesstoken")
                && !l.to_lowercase().contains("access_token")
                && !l.to_lowercase().contains("authorization")
        })
        .take(8)
        .map(|l| l.chars().take(300).collect::<String>())
        .collect();
    Ok(
        json!({"title":title,"ok":ok,"record":meta,"findings":findings,"evidence":evidence,"crash_file":crash.and_then(|p|p.file_name().map(|s|s.to_string_lossy().into_owned())),"note":"Выводы основаны на сигнатурах локального лога. Если явной причины нет, открой полный лог."}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identifies_memory_java_and_mod_failures() {
        assert!(
            analyze("MixinApplyError: failed from mod controlify: injection failed")[0]["detail"]
                .as_str()
                .unwrap()
                .contains("controlify")
        );
        assert_eq!(
            analyze("java.lang.OutOfMemoryError: Java heap space")[0]["action"],
            "java"
        );
        assert_eq!(
            analyze("UnsupportedClassVersionError: class file version 65")[0]["title"],
            "Неподходящая Java"
        );
        assert_eq!(
            analyze("Incompatible mods found! Mod test requires version >=1.21")[0]["action"],
            "compatibility"
        );
        assert!(analyze("Stopping normally").is_empty());
    }
}
