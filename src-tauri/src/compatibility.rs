use crate::store::{Settings, State};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs,
    io::{Cursor, Read, Seek},
    path::Path,
};

#[derive(Clone, Debug)]
struct Requirement {
    id: String,
    range: Value,
    conflict: bool,
}
#[derive(Clone, Debug)]
struct Mod {
    embedded: bool,
    id: String,
    version: String,
    file: String,
    formats: Vec<String>,
    requirements: Vec<Requirement>,
}
fn version(raw: &str) -> Option<semver::Version> {
    let raw = raw.trim().trim_start_matches('v');
    let end = raw.find(['-', '+']).unwrap_or(raw.len());
    let base = &raw[..end];
    let suffix = if &raw[end..] == "-" {
        "-0"
    } else {
        &raw[end..]
    };
    let parts = base.split('.').count();
    let v = match parts {
        1 => format!("{base}.0.0{suffix}"),
        2 => format!("{base}.0{suffix}"),
        _ => format!("{base}{suffix}"),
    };
    let mut parsed = semver::Version::parse(&v).ok()?;
    parsed.build = semver::BuildMetadata::EMPTY;
    Some(parsed)
}
fn matches(raw: &str, range: &Value) -> Option<bool> {
    if let Some(ranges) = range.as_array() {
        let values: Vec<_> = ranges.iter().map(|r| matches(raw, r)).collect();
        return if values.contains(&Some(true)) {
            Some(true)
        } else if values.contains(&None) {
            None
        } else {
            Some(false)
        };
    }
    let range = range.as_str()?.trim();
    if range == "*" || range.is_empty() {
        return Some(true);
    }
    let v = version(raw)?;
    if range.contains("||") {
        let values: Vec<_> = range
            .split("||")
            .map(|s| matches(raw, &json!(s.trim())))
            .collect();
        return if values.contains(&Some(true)) {
            Some(true)
        } else if values.contains(&None) {
            None
        } else {
            Some(false)
        };
    }
    if (range.starts_with('[') || range.starts_with('('))
        && (range.ends_with(']') || range.ends_with(')'))
    {
        let inner = &range[1..range.len() - 1];
        if let Some((low, high)) = inner.split_once(',') {
            let lo = if low.is_empty() {
                true
            } else {
                let bound = version(low)?;
                if range.starts_with('[') {
                    v >= bound
                } else {
                    v > bound
                }
            };
            let hi = if high.is_empty() {
                true
            } else {
                let bound = version(high)?;
                if range.ends_with(']') {
                    v <= bound
                } else {
                    v < bound
                }
            };
            return Some(lo && hi);
        }
        return Some(v == version(inner)?);
    }
    if let Some((low, high)) = range.split_once(" - ") {
        return Some(v >= version(low)? && v <= version(high)?);
    }
    // Fabric uses whitespace intersections; semver expects commas. Bare versions are exact.
    if range.chars().next().is_some_and(|c| c.is_ascii_digit())
        && !range.contains(['*', 'x', 'X', ',', ' '])
    {
        return Some(v == version(range)?);
    }
    let mut unknown = false;
    for token in range
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
    {
        let op = [">=", "<=", ">", "<", "=", "~", "^"]
            .into_iter()
            .find(|p| token.starts_with(p))
            .unwrap_or("");
        let operand = &token[op.len()..];
        let result = if operand.contains(['*', 'x', 'X']) {
            if !op.is_empty() {
                None
            } else {
                let prefix = operand
                    .split('.')
                    .take_while(|s| !["*", "x", "X"].contains(s))
                    .map(str::parse::<u64>)
                    .collect::<Result<Vec<_>, _>>()
                    .ok();
                prefix.map(|p| {
                    p.iter()
                        .zip([v.major, v.minor, v.patch])
                        .all(|(a, b)| *a == b)
                })
            }
        } else if let Some(bound) = version(operand) {
            match op {
                ">=" => Some(v >= bound),
                "<=" => Some(v <= bound),
                ">" => Some(v > bound),
                "<" => Some(v < bound),
                "" | "=" => Some(v == bound),
                "~" | "^" => {
                    let count = operand
                        .split(['-', '+'])
                        .next()
                        .unwrap_or("")
                        .split('.')
                        .count();
                    let mut upper = if op == "~" {
                        if count == 1 {
                            semver::Version::new(bound.major + 1, 0, 0)
                        } else {
                            semver::Version::new(bound.major, bound.minor + 1, 0)
                        }
                    } else if bound.major > 0 || count == 1 {
                        semver::Version::new(bound.major + 1, 0, 0)
                    } else if bound.minor > 0 || count == 2 {
                        semver::Version::new(0, bound.minor + 1, 0)
                    } else {
                        semver::Version::new(0, 0, bound.patch + 1)
                    };
                    upper.pre = semver::Prerelease::new("0").ok()?;
                    Some(v >= bound && v < upper)
                }
                _ => None,
            }
        } else {
            None
        };
        match result {
            Some(false) => return Some(false),
            None => unknown = true,
            _ => {}
        }
    }
    if unknown {
        None
    } else {
        Some(true)
    }
}
fn json_metadata(bytes: &[u8]) -> Option<Value> {
    if let Ok(v) = serde_json::from_slice(bytes) {
        return Some(v);
    }
    let text = std::str::from_utf8(bytes)
        .ok()?
        .trim_start_matches('\u{feff}');
    // Gson, used by Fabric, accepts literal control characters in JSON strings.
    let mut clean = String::new();
    let (mut quoted, mut escape) = (false, false);
    for c in text.chars() {
        if quoted {
            if escape {
                clean.push(c);
                escape = false;
                continue;
            }
            match c {
                '\\' => {
                    clean.push(c);
                    escape = true;
                }
                '"' => {
                    clean.push(c);
                    quoted = false;
                }
                '\n' => clean.push_str("\\n"),
                '\r' => clean.push_str("\\r"),
                '\t' => clean.push_str("\\t"),
                c if c.is_control() => clean.push_str(&format!("\\u{:04x}", c as u32)),
                _ => clean.push(c),
            }
        } else {
            clean.push(c);
            if c == '"' {
                quoted = true;
            }
        }
    }
    serde_json::from_str(&clean).ok()
}
fn read_entry<R: Read + Seek>(
    zip: &mut zip::ZipArchive<R>,
    name: &str,
    limit: u64,
) -> Option<Vec<u8>> {
    let mut f = zip.by_name(name).ok()?;
    if f.size() > limit {
        return None;
    }
    let mut bytes = vec![];
    f.read_to_end(&mut bytes).ok()?;
    Some(bytes)
}
fn metadata<R: Read + Seek>(
    zip: &mut zip::ZipArchive<R>,
    file: &str,
    depth: usize,
    budget: &mut u64,
    out: &mut Vec<Mod>,
) -> Result<(), String> {
    let fabric = read_entry(zip, "fabric.mod.json", 1024 * 1024).and_then(|b| json_metadata(&b));
    let quilt = read_entry(zip, "quilt.mod.json", 1024 * 1024).and_then(|b| json_metadata(&b));
    let forge = read_entry(zip, "META-INF/mods.toml", 1024 * 1024)
        .and_then(|b| String::from_utf8(b).ok())
        .and_then(|s| s.parse::<toml::Value>().ok());
    let neo = read_entry(zip, "META-INF/neoforge.mods.toml", 1024 * 1024)
        .and_then(|b| String::from_utf8(b).ok())
        .and_then(|s| s.parse::<toml::Value>().ok());
    let mut formats = vec![];
    if fabric.is_some() {
        formats.push("Fabric".into());
    }
    if quilt.is_some() {
        formats.push("Quilt".into());
    }
    if forge.is_some() {
        formats.push("Forge".into());
    }
    if neo.is_some() {
        formats.push("NeoForge".into());
    }
    if let Some(meta) = &fabric {
        if let Some(id) = meta["id"].as_str() {
            let mut requirements = vec![];
            for (key, conflict) in [("depends", false), ("breaks", true)] {
                for (id, range) in meta[key].as_object().into_iter().flatten() {
                    requirements.push(Requirement {
                        id: id.clone(),
                        range: range.clone(),
                        conflict,
                    });
                }
            }
            out.push(Mod {
                embedded: depth > 0,
                id: id.into(),
                version: meta["version"].as_str().unwrap_or("").into(),
                file: file.into(),
                formats: formats.clone(),
                requirements,
            });
            for id in meta["provides"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                out.push(Mod {
                    embedded: true,
                    id: id.into(),
                    version: meta["version"].as_str().unwrap_or("").into(),
                    file: file.into(),
                    formats: formats.clone(),
                    requirements: vec![],
                });
            }
        }
    }
    if let Some(meta) = &quilt {
        let q = &meta["quilt_loader"];
        if let Some(id) = q["id"].as_str() {
            if !out.iter().any(|m| m.id == id && m.file == file) {
                let requirements = q["depends"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|d| d["optional"] != true)
                    .filter_map(|d| {
                        Some(Requirement {
                            id: d["id"].as_str()?.into(),
                            range: d.get("versions").cloned().unwrap_or(json!("*")),
                            conflict: false,
                        })
                    })
                    .collect();
                out.push(Mod {
                    embedded: depth > 0,
                    id: id.into(),
                    version: q["version"].as_str().unwrap_or("").into(),
                    file: file.into(),
                    formats: formats.clone(),
                    requirements,
                });
            }
        }
    }
    let manifest = read_entry(zip, "META-INF/MANIFEST.MF", 1024 * 1024)
        .and_then(|b| String::from_utf8(b).ok())
        .unwrap_or_default();
    let implementation = manifest
        .lines()
        .find_map(|l| l.strip_prefix("Implementation-Version: "))
        .unwrap_or("")
        .trim();
    for meta in [forge.as_ref(), neo.as_ref()].into_iter().flatten() {
        for m in meta
            .get("mods")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(id) = m.get("modId").and_then(toml::Value::as_str) else {
                continue;
            };
            if out.iter().any(|m| m.id == id && m.file == file) {
                continue;
            }
            let v = m.get("version").and_then(toml::Value::as_str).unwrap_or("");
            let requirements = meta
                .get("dependencies")
                .and_then(|d| d.get(id))
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
                .filter(|d| {
                    d.get("mandatory").and_then(toml::Value::as_bool) == Some(true)
                        || d.get("type").and_then(toml::Value::as_str) == Some("required")
                })
                .filter_map(|d| {
                    if d.get("side").and_then(toml::Value::as_str) == Some("SERVER") {
                        return None;
                    }
                    Some(Requirement {
                        id: d.get("modId")?.as_str()?.into(),
                        range: json!(d
                            .get("versionRange")
                            .and_then(toml::Value::as_str)
                            .unwrap_or("*")),
                        conflict: false,
                    })
                })
                .collect();
            out.push(Mod {
                embedded: depth > 0,
                id: id.into(),
                version: if v.contains("${") {
                    implementation.into()
                } else {
                    v.into()
                },
                file: file.into(),
                formats: formats.clone(),
                requirements,
            });
        }
    }
    if depth < 3 {
        let mut nested: Vec<String> = fabric
            .as_ref()
            .map(|m| {
                m["jars"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|j| j["file"].as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        if let Some(meta) = read_entry(zip, "META-INF/jarjar/metadata.json", 1024 * 1024)
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        {
            nested.extend(
                meta["jars"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|j| j["path"].as_str().map(str::to_string)),
            );
        }
        nested.truncate(300);
        for name in nested {
            if *budget > 64 * 1024 * 1024 {
                return Err("Слишком много вложенных JAR".into());
            }
            if let Some(bytes) = read_entry(zip, &name, 16 * 1024 * 1024) {
                *budget += bytes.len() as u64;
                if let Ok(mut child) = zip::ZipArchive::new(Cursor::new(bytes)) {
                    metadata(&mut child, file, depth + 1, budget, out)?;
                }
            }
        }
    }
    Ok(())
}
fn issue(issues: &mut Vec<Value>, severity: &str, file: &str, title: &str, detail: String) {
    issues.push(json!({"severity":severity,"file":file,"title":title,"detail":detail}));
}
fn analyze(mods: &[Mod], s: &Settings, unknown: &[String]) -> Value {
    let mut issues = vec![];
    let mut available: HashMap<String, (String, String, bool)> = HashMap::new();
    available.insert(
        "minecraft".into(),
        (s.version.clone(), String::new(), false),
    );
    let loader_id = match s.loader.as_str() {
        "Fabric" => "fabricloader",
        "Quilt" => "quilt_loader",
        "Forge" => "forge",
        "NeoForge" => "neoforge",
        _ => "",
    };
    if !loader_id.is_empty() {
        available.insert(
            loader_id.into(),
            (s.loader_version.clone(), String::new(), false),
        );
    }
    if s.loader == "Quilt" {
        available.insert("fabricloader".into(), (String::new(), String::new(), false));
    }
    for m in mods {
        if let Some((old_version, file, embedded)) = available.get(&m.id) {
            if !file.is_empty() && file != &m.file && !embedded && !m.embedded {
                issue(
                    &mut issues,
                    "error",
                    &m.file,
                    "Повторяющийся мод",
                    format!("ID {} уже есть в {}", m.id, file),
                );
            }
            if !file.is_empty()
                && *embedded
                && (!m.embedded || version(&m.version) > version(old_version))
            {
                available.insert(
                    m.id.clone(),
                    (m.version.clone(), m.file.clone(), m.embedded),
                );
            }
        } else {
            available.insert(
                m.id.clone(),
                (m.version.clone(), m.file.clone(), m.embedded),
            );
        }
    }
    let mut seen = std::collections::HashSet::new();
    for m in mods {
        let accepted = m
            .formats
            .iter()
            .any(|l| l == &s.loader || (s.loader == "Quilt" && l == "Fabric"));
        if !accepted && m.embedded {
            continue;
        }
        if !accepted && seen.insert(format!("loader/{}", m.file)) {
            issue(
                &mut issues,
                "error",
                &m.file,
                "Другое ядро",
                format!(
                    "Мод рассчитан на {}, выбрано {}.",
                    m.formats.join(" / "),
                    s.loader
                ),
            );
        }
        for r in &m.requirements {
            if r.id == "java" {
                continue;
            }
            if let Some((installed, _, _)) = available.get(&r.id) {
                match matches(installed,&r.range){Some(true) if r.conflict=>issue(&mut issues,"error",&m.file,"Несовместимые моды",format!("{} несовместим с {} {} ({})",m.id,r.id,installed,r.range)),Some(false) if !r.conflict=>issue(&mut issues,"error",&m.file,"Не подходит версия зависимости",format!("{} требует {} {}, установлено {}",m.id,r.id,r.range,installed)),None if !r.conflict&&seen.insert(format!("range/{}/{:?}",r.id,r.range))=>issue(&mut issues,"warning",&m.file,"Версия требует проверки",format!("Для {} требуется {}. Версия неизвестна или формат ограничения не распознан.",r.id,r.range)),_=>{}}
            } else if !r.conflict {
                issue(
                    &mut issues,
                    "error",
                    &m.file,
                    "Не хватает зависимости",
                    format!(
                        "{} требует {} {}. Зависимость отсутствует или выключена.",
                        m.id, r.id, r.range
                    ),
                );
            }
        }
    }
    for file in unknown {
        issue(
            &mut issues,
            "warning",
            file,
            "Манифест не распознан",
            "Проверить совместимость этого JAR автоматически не удалось.".into(),
        );
    }
    let errors = issues.iter().filter(|i| i["severity"] == "error").count();
    let warnings = issues.len() - errors;
    json!({"checked":mods.iter().map(|m|&m.file).collect::<std::collections::HashSet<_>>().len(),"errors":errors,"warnings":warnings,"issues":issues,"scope":"Проверка локальных манифестов и вложенных модов. Игровые конфликты могут проявиться при запуске."})
}
pub fn check_dir(dir: &Path, s: &Settings) -> Result<Value, String> {
    let mut mods = vec![];
    let mut unknown = vec![];
    let mut budget = 0;
    if dir.exists() {
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".jar") || !entry.file_type().map_err(|e| e.to_string())?.is_file() {
                continue;
            }
            let before = mods.len();
            let result = (|| {
                let file = fs::File::open(entry.path()).map_err(|e| e.to_string())?;
                let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
                metadata(&mut zip, &name, 0, &mut budget, &mut mods)
            })();
            if result.is_err() || mods.len() == before {
                unknown.push(name);
            }
        }
    }
    Ok(analyze(&mods, s, &unknown))
}

pub fn check_compatibility(state: &State) -> Result<Value, String> {
    let s = state
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .settings
        .clone();
    let folder=state.instance_dir(&s)?.join("mods");
    let mut report=check_dir(&folder, &s)?;
    let mut files=Vec::new();
    if folder.exists(){for entry in fs::read_dir(&folder).map_err(|e|e.to_string())?{
        let entry=entry.map_err(|e|e.to_string())?;if entry.path().extension().is_none_or(|ext|ext!="jar"){continue;}
        let meta=entry.metadata().map_err(|e|e.to_string())?;
        files.push((entry.file_name().to_string_lossy().into_owned(),meta.len(),meta.modified().ok().and_then(|t|t.duration_since(std::time::UNIX_EPOCH).ok()).map(|t|t.as_nanos().to_string())));
    }}
    files.sort();
    use sha2::{Digest,Sha256};
    report["fingerprint"]=json!(format!("{:x}",Sha256::digest(serde_json::to_vec(&files).map_err(|e|e.to_string())?)));
    Ok(report)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn version_ranges() {
        assert_eq!(matches("1.2.3+fabric", &json!("1.2.3")), Some(true));
        assert_eq!(matches("0.5.0-beta.4", &json!(">=0.3.2")), Some(true));
        assert_eq!(matches("1.21.1", &json!(">=1.21- <1.21.2-")), Some(true));
        assert_eq!(
            matches("1.21.1", &json!(">=1.15-alpha.19.38.b")),
            Some(true)
        );
        assert_eq!(matches("1.21.1", &json!(">=1.21 <1.22")), Some(true));
        assert_eq!(matches("1.21.1", &json!("[1.20,1.21)")), Some(false));
        assert_eq!(matches("1.21.1", &json!(["1.20.x", "1.21.x"])), Some(true));
        assert_eq!(matches("", &json!(">=1")), None);
    }
    #[test]
    fn supports_gson_multiline_metadata() {
        assert_eq!(
            json_metadata(b"{\"description\":\"line1\nline2\",\"id\":\"test\"}").unwrap()["id"],
            "test"
        );
    }
    #[test]
    fn nested_libraries_are_not_duplicate_root_mods() {
        let s = Settings::default();
        let a = Mod {
            embedded: true,
            id: "common".into(),
            version: "1.0.0".into(),
            file: "one.jar".into(),
            formats: vec!["Fabric".into()],
            requirements: vec![],
        };
        let b = Mod {
            file: "two.jar".into(),
            version: "1.1.0".into(),
            ..a.clone()
        };
        assert_eq!(analyze(&[a, b], &s, &[])["errors"], 0);
    }
    #[test]
    fn catches_wrong_loader_missing_dependency_and_duplicates() {
        let mut s = Settings::default();
        s.loader = "Forge".into();
        let m = Mod {
            embedded: false,
            id: "test".into(),
            version: "1.0.0".into(),
            file: "a.jar".into(),
            formats: vec!["Fabric".into()],
            requirements: vec![Requirement {
                id: "missing".into(),
                range: json!("*"),
                conflict: false,
            }],
        };
        let report = analyze(
            &[
                m.clone(),
                Mod {
                    file: "b.jar".into(),
                    ..m
                },
            ],
            &s,
            &[],
        );
        assert!(report["errors"].as_u64().unwrap() >= 3);
    }
}
