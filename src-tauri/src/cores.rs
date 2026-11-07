use crate::{
    net,
    store::{safe_segment, Settings},
};
use serde_json::{json, Value};
use std::{fs, path::Path, process::Stdio};

pub const LOADERS: [&str; 5] = ["Vanilla", "Fabric", "Quilt", "Forge", "NeoForge"];
fn version_key(s: &str) -> Vec<u32> {
    s.split(|c: char| !c.is_ascii_digit())
        .filter_map(|x| x.parse().ok())
        .collect()
}
pub fn newest(mut versions: Vec<String>) -> Result<String, String> {
    if versions.iter().any(|v| !v.contains('-')) {
        versions.retain(|v| !v.contains('-'));
    }
    versions.sort_by_key(|v| version_key(v));
    versions
        .pop()
        .ok_or("Для этой версии Minecraft нет совместимого ядра".into())
}
pub async fn light_profile(loader: &str, game: &str) -> Result<Value, String> {
    light_profile_version(loader, game, "").await
}
pub async fn light_profile_version(
    loader: &str,
    game: &str,
    pinned: &str,
) -> Result<Value, String> {
    safe_segment(game)?;
    let base = match loader {
        "Fabric" => "https://meta.fabricmc.net/v2",
        "Quilt" => "https://meta.quiltmc.org/v3",
        _ => return Err("Неизвестное ядро".into()),
    };
    if !pinned.is_empty() {
        safe_segment(pinned)?;
        return net::json(&format!(
            "{base}/versions/loader/{game}/{pinned}/profile/json"
        ))
        .await;
    }
    let list = net::json(&format!("{base}/versions/loader/{game}")).await?;
    let versions = list
        .as_array()
        .ok_or("Некорректный список ядер")?
        .iter()
        .filter_map(|v| v["loader"]["version"].as_str().map(str::to_owned))
        .collect();
    let version = newest(versions)?;
    safe_segment(&version)?;
    net::json(&format!(
        "{base}/versions/loader/{game}/{version}/profile/json"
    ))
    .await
}
pub fn neo_prefix(game: &str) -> Result<String, String> {
    let parts: Vec<_> = game.split('.').collect();
    if parts.len() < 2 || parts[0] != "1" || parts[1].parse::<u32>().unwrap_or(0) < 20 {
        return Err("NeoForge доступен для современных версий Minecraft 1.20.2 и новее".into());
    }
    if game == "1.20" || game == "1.20.1" {
        return Err(
            "Для этой версии используйте Fabric или Quilt; NeoForge поддерживается с 1.20.2".into(),
        );
    }
    Ok(format!("{}.{}.", parts[1], parts.get(2).unwrap_or(&"0")))
}
pub async fn neo_version(game: &str) -> Result<String, String> {
    let prefix = neo_prefix(game)?;
    let xml = net::client()?
        .get("https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    let versions = xml
        .split("<version>")
        .skip(1)
        .filter_map(|s| s.split("</version>").next())
        .filter(|s| s.starts_with(&prefix))
        .map(str::to_owned)
        .collect();
    newest(versions)
}
pub fn merge_profile(base: &mut Value, extra: &Value) -> Result<(), String> {
    let additions = extra["libraries"]
        .as_array()
        .ok_or("В профиле ядра нет библиотек")?;
    let key = |v: &Value| {
        let n = v["name"].as_str().unwrap_or("");
        let p: Vec<_> = n.split(':').collect();
        format!(
            "{}:{}:{}",
            p.first().unwrap_or(&""),
            p.get(1).unwrap_or(&""),
            p.get(3).unwrap_or(&"")
        )
    };
    let replaced: std::collections::HashSet<_> = additions.iter().map(&key).collect();
    let libs = base["libraries"]
        .as_array_mut()
        .ok_or("В Minecraft нет списка библиотек")?;
    libs.retain(|v| !replaced.contains(&key(v)));
    libs.extend(additions.clone());
    if !extra["mainClass"].is_string() {
        return Err("В профиле ядра нет mainClass".into());
    }
    base["mainClass"] = extra["mainClass"].clone();
    for kind in ["jvm", "game"] {
        if let Some(args) = extra["arguments"][kind].as_array() {
            if base["arguments"][kind].is_null() {
                base["arguments"][kind] = json!([]);
            }
            base["arguments"][kind]
                .as_array_mut()
                .ok_or("Некорректные аргументы Minecraft")?
                .extend(args.clone());
        }
    }
    Ok(())
}
pub async fn neoforge_profile(
    root: &Path,
    settings: &Settings,
    base: &Value,
) -> Result<Value, String> {
    installer_profile(root, settings, base, false).await
}
pub async fn forge_profile(
    root: &Path,
    settings: &Settings,
    base: &Value,
) -> Result<Value, String> {
    installer_profile(root, settings, base, true).await
}
async fn installer_profile(
    root: &Path,
    settings: &Settings,
    base: &Value,
    forge: bool,
) -> Result<Value, String> {
    let version = if settings.loader_version.is_empty() {
        if forge {
            let promos = net::json(
                "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json",
            )
            .await?;
            promos["promos"][format!("{}-recommended", settings.version)]
                .as_str()
                .or_else(|| promos["promos"][format!("{}-latest", settings.version)].as_str())
                .ok_or("Нет Forge для выбранной версии Minecraft")?
                .to_string()
        } else {
            neo_version(&settings.version).await?
        }
    } else {
        settings.loader_version.clone()
    };
    safe_segment(&version)?;
    let core = if forge { "Forge" } else { "NeoForge" };
    let id = if forge {
        format!("{}-forge-{version}", settings.version)
    } else {
        format!("neoforge-{version}")
    };
    let profile_path = root.join("versions").join(&id).join(format!("{id}.json"));
    let marker = root.join("metadata").join(format!("{id}.installed"));
    if marker.exists() && profile_path.exists() {
        return serde_json::from_slice(&fs::read(profile_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string());
    }
    let url = if forge {
        format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{}-{version}/forge-{}-{version}-installer.jar",settings.version,settings.version)
    } else {
        format!("https://maven.neoforged.net/releases/net/neoforged/neoforge/{version}/neoforge-{version}-installer.jar")
    };
    let sha = net::client()?
        .get(format!("{url}.sha1"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    let sha = sha
        .split_whitespace()
        .next()
        .ok_or("Не получен SHA-1 установщика NeoForge")?;
    if sha.len() != 40 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Неверный SHA-1 установщика NeoForge".into());
    }
    let installer = root.join("installers").join(format!("{id}.jar"));
    net::download(&url, &installer, Some(sha)).await?;
    let vanilla = root.join("versions").join(&settings.version);
    fs::create_dir_all(&vanilla).map_err(|e| e.to_string())?;
    fs::write(
        vanilla.join(format!("{}.json", settings.version)),
        serde_json::to_vec(base).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let profiles = root.join("launcher_profiles.json");
    if !profiles.exists() {
        fs::write(profiles, b"{\"profiles\":{},\"settings\":{},\"version\":3}")
            .map_err(|e| e.to_string())?;
    }
    let log = fs::File::create(root.join("core-install.log")).map_err(|e| e.to_string())?;
    let mut command = tokio::process::Command::new(&settings.java_path);
    command
        .kill_on_drop(true)
        .current_dir(root)
        .arg("-Djava.awt.headless=true")
        .arg("-jar")
        .arg(&installer)
        .arg(if forge {
            "--installClient"
        } else {
            "--install-client"
        })
        .arg(root)
        .stdout(Stdio::from(log.try_clone().map_err(|e| e.to_string())?))
        .stderr(Stdio::from(log));
    #[cfg(windows)]
    {
        command.creation_flags(0x08000000);
    }
    let mut child = command
        .spawn()
        .map_err(|e| format!("Не удалось запустить установщик {core}: {e}"))?;
    let status = child.wait().await.map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("Установка {core} завершилась с кодом {:?}. Подробности: core-install.log в папке данных лаунчера",status.code()));
    }
    let profile: Value = serde_json::from_slice(
        &fs::read(&profile_path)
            .map_err(|e| format!("Установщик не создал профиль NeoForge: {e}"))?,
    )
    .map_err(|e| e.to_string())?;
    if profile["inheritsFrom"] != settings.version {
        return Err("Установщик создал профиль для другой версии Minecraft".into());
    }
    fs::create_dir_all(root.join("metadata")).map_err(|e| e.to_string())?;
    fs::write(marker, version).map_err(|e| e.to_string())?;
    Ok(profile)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matching_versions() {
        assert_eq!(neo_prefix("1.21.1").unwrap(), "21.1.");
        assert_eq!(neo_prefix("1.21").unwrap(), "21.0.");
        assert!(neo_prefix("1.20.1").is_err());
        assert_eq!(
            newest(vec![
                "0.20.0-beta.9".into(),
                "0.27.0".into(),
                "0.29.2".into(),
                "0.29.10".into()
            ])
            .unwrap(),
            "0.29.10"
        );
    }
    #[test]
    fn replaces_conflicting_libraries() {
        let mut base = json!({"mainClass":"vanilla","libraries":[{"name":"org.ow2.asm:asm:9.5"},{"name":"org.lwjgl:lwjgl:3.3.3:natives-windows"}],"arguments":{"jvm":["-cp","${classpath}"],"game":["--username","${auth_player_name}"]}});
        merge_profile(&mut base,&json!({"mainClass":"core","libraries":[{"name":"org.ow2.asm:asm:9.10.1"}],"arguments":{"jvm":["--add-modules","ALL-MODULE-PATH"]}})).unwrap();
        assert_eq!(base["libraries"].as_array().unwrap().len(), 2);
        assert_eq!(base["mainClass"], "core");
        assert_eq!(base["arguments"]["jvm"].as_array().unwrap().len(), 4);
    }
    #[tokio::test]
    #[ignore = "requires official Quilt and NeoForge endpoints"]
    async fn official_core_metadata() {
        let q = light_profile("Quilt", "1.21.1").await.unwrap();
        assert!(q["mainClass"].as_str().unwrap().contains("quiltmc"));
        assert!(neo_version("1.21.1").await.unwrap().starts_with("21.1."));
    }
    #[tokio::test]
    #[ignore = "downloads and runs official NeoForge client installer"]
    async fn official_neoforge_client_install() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../target/neoforge-integration");
        fs::create_dir_all(root.join("versions/1.21.1")).unwrap();
        let manifest = net::json("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
            .await
            .unwrap();
        let item = manifest["versions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["id"] == "1.21.1")
            .unwrap();
        let base = net::json(item["url"].as_str().unwrap()).await.unwrap();
        let jar = root.join("versions/1.21.1/1.21.1.jar");
        let cached = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../dist/data/versions/1.21.1/1.21.1.jar");
        if !jar.exists() && cached.exists() {
            fs::copy(cached, &jar).unwrap();
        }
        let client = &base["downloads"]["client"];
        net::download(
            client["url"].as_str().unwrap(),
            &jar,
            client["sha1"].as_str(),
        )
        .await
        .unwrap();
        let mut settings = Settings::default();
        settings.loader = "NeoForge".into();
        let p = neoforge_profile(&root, &settings, &base).await.unwrap();
        assert_eq!(p["inheritsFrom"], "1.21.1");
        assert!(p["mainClass"]
            .as_str()
            .unwrap()
            .contains("BootstrapLauncher"));
        let mut merged = base;
        merge_profile(&mut merged, &p).unwrap();
        assert!(merged["arguments"]["jvm"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a == "-p"));
    }
}
