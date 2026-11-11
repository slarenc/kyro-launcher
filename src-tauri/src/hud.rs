//! Built-in HUD is attached to every launched game, independently of its version or mod loader.
use crate::{store::{Settings, State},net};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{fs,io::{Read,Write},path::Path};

const AGENT:&[u8]=include_bytes!("../resources/kyro-hud-agent.jar");
const RUNTIME:&[u8]=include_bytes!("../resources/kyro-hud-runtime.jar");
const MEDIA:&[u8]=include_bytes!("../resources/kyro-media.exe");
const LEGACY_HASHES:&[&str]=&["c07263e0b0b79352533fc4b2bba0e338d22e85cb0fdad110b8062814443c326f","f9972f6623ea80edc3b15f467abe28fd1d9dc47eebb93bbd3dfbf3950194794c","dc9c5d17cd99bf865e852df8f8699d289997d9e28a82b99807a81cc7c1c353bc"];

fn owned_folder(folder:&Path,boundary:&Path)->Result<(),String>{
    fs::create_dir_all(folder).map_err(|e|e.to_string())?;
    if !fs::canonicalize(folder).map_err(|e|e.to_string())?.starts_with(fs::canonicalize(boundary).map_err(|e|e.to_string())?){return Err("Папка встроенного HUD ведёт за пределы профиля".into());}Ok(())
}
fn ensure(path:&Path,bytes:&[u8])->Result<(),String>{
    if path.symlink_metadata().is_ok(){
        if !path.symlink_metadata().map_err(|e|e.to_string())?.file_type().is_file()||fs::read(path).map_err(|e|e.to_string())?!=bytes{return Err(format!("Файл встроенного HUD изменён: {}",path.display()));}return Ok(());
    }
    let temporary=path.with_extension(format!("tmp-{}",chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()));
    let result=(||{let mut f=fs::OpenOptions::new().write(true).create_new(true).open(&temporary).map_err(|e|e.to_string())?;f.write_all(bytes).and_then(|_|f.sync_all()).map_err(|e|e.to_string())?;fs::rename(&temporary,path).map_err(|e|e.to_string())})();
    if result.is_err(){let _=fs::remove_file(temporary);}result
}
fn mod_id(path:&Path)->Option<String>{
    let mut zip=zip::ZipArchive::new(fs::File::open(path).ok()?).ok()?;let file=zip.by_name("fabric.mod.json").ok()?;let mut text=String::new();file.take(256*1024).read_to_string(&mut text).ok()?;
    serde_json::from_str::<Value>(&text).ok()?["id"].as_str().map(str::to_owned)
}
fn migrate_legacy(profile:&Path)->Result<(),String>{
    let dir=profile.join("mods");if !dir.exists(){return Ok(());}
    for entry in fs::read_dir(&dir).map_err(|e|e.to_string())?{
        let entry=entry.map_err(|e|e.to_string())?;let path=entry.path();if path.extension().is_none_or(|s|s!="jar")||mod_id(&path).as_deref()!=Some("kyro_hud"){continue;}
        let bytes=fs::read(&path).map_err(|e|e.to_string())?;
        if !entry.file_type().map_err(|e|e.to_string())?.is_file()||!LEGACY_HASHES.contains(&format!("{:x}",Sha256::digest(&bytes)).as_str()){return Err("Найден изменённый старый мод KYRO HUD. Удали его через менеджер модов, чтобы избежать двух HUD одновременно.".into());}
        owned_folder(&dir,profile)?;
        let backup=path.with_file_name(format!("{}.kyro-backup-{}",entry.file_name().to_string_lossy(),chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()));
        fs::rename(path,backup).map_err(|e|e.to_string())?;
    }Ok(())
}
pub fn sync_appearance(state:&State,settings:&Settings)->Result<(),String>{
    let folder=state.root.join("runtime/kyro-hud");owned_folder(&folder,&state.root)?;
    let bytes=serde_json::to_vec(&json!({"theme":settings.theme,"accent":settings.accent})).map_err(|e|e.to_string())?;
    let temporary=folder.join(format!("appearance-{}.tmp",chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()));
    fs::write(&temporary,bytes).and_then(|_|fs::rename(&temporary,folder.join("appearance.json"))).map_err(|e|e.to_string())
}
fn replace_owned(path:&Path,bytes:&[u8])->Result<(),String>{
    let temporary=path.with_extension("kyro-tmp");fs::write(&temporary,bytes).and_then(|_|fs::rename(temporary,path)).map_err(|e|e.to_string())
}
pub fn prepare_badge(state:&State,settings:&Settings)->Result<(),String>{
    let profile=state.instance_dir(settings)?;let pack=profile.join("resourcepacks/kyro-badge");owned_folder(&pack,&profile)?;
    let mut format=4;
    let jar=state.root.join("versions").join(&settings.version).join(format!("{}.jar",settings.version));
    if let Ok(file)=fs::File::open(jar){if let Ok(mut jar)=zip::ZipArchive::new(file){if let Ok(mut version)=jar.by_name("version.json"){
        let mut text=String::new();if version.read_to_string(&mut text).is_ok(){if let Ok(value)=serde_json::from_str::<Value>(&text){format=value["pack_version"]["resource"].as_u64().or_else(||value["pack_version"]["resource_major"].as_u64()).or_else(||value["pack_version"]["resource"]["major"].as_u64()).unwrap_or(4);}}
    }}}
    let mut metadata=json!({"pack":{"pack_format":format,"description":"KYRO · значок перед ником"}});
    // Since resource format 65, supported_formats makes the whole pack invalid.
    // Older clients use pack_format; modern clients additionally require min/max.
    if format>=65{metadata["pack"]["min_format"]=json!([format,0]);metadata["pack"]["max_format"]=json!([format,999]);}
    replace_owned(&pack.join("pack.mcmeta"),&serde_json::to_vec(&metadata).map_err(|e|e.to_string())?)?;
    for folder in ["assets/kyro/font","assets/kyro/textures/font"]{owned_folder(&pack.join(folder),&profile)?;}
    replace_owned(&pack.join("assets/kyro/textures/font/badge.png"),include_bytes!("../../ui/kyro-logo.png"))?;
    replace_owned(&pack.join("assets/kyro/font/badge.json"),&serde_json::to_vec(&json!({"providers":[{"type":"bitmap","file":"kyro:font/badge.png","ascent":7,"height":8,"chars":["\u{e100}"]},{"type":"space","advances":{" ":4}}]})).map_err(|e|e.to_string())?)?;
    // Remove only the legacy file shipped by KYRO. Server/user fonts are untouched.
    let legacy=pack.join("assets/minecraft/font/default.json");if legacy.exists(){fs::remove_file(legacy).map_err(|e|e.to_string())?;}
    let options=profile.join("options.txt");let old=fs::read_to_string(&options).unwrap_or_default();let mut lines=Vec::new();let mut found=false;
    for line in old.lines(){if let Some(value)=line.strip_prefix("resourcePacks:"){
        let mut packs:Vec<String>=serde_json::from_str(value).map_err(|e|format!("Не удалось прочитать список ресурспаков: {e}"))?;
        if !packs.iter().any(|name|name=="file/kyro-badge"){packs.push("file/kyro-badge".into());}
        lines.push(format!("resourcePacks:{}",serde_json::to_string(&packs).map_err(|e|e.to_string())?));found=true;
    }else{lines.push(line.to_owned());}}
    if !found{lines.push("resourcePacks:[\"file/kyro-badge\"]".into());}
    let updated=format!("{}\n",lines.join("\n"));if updated!=old{
        let backup=profile.join("config/options-before-kyro-badge.txt");if options.exists()&&!backup.exists(){fs::copy(&options,&backup).map_err(|e|e.to_string())?;}
        replace_owned(&options,updated.as_bytes())?;
    }Ok(())
}
pub fn prepare(state:&State,settings:&Settings)->Result<Vec<String>,String>{
    sync_appearance(state,settings)?;
    let profile=state.instance_dir(settings)?;fs::create_dir_all(&profile).map_err(|e|e.to_string())?;
    let config=profile.join("config");owned_folder(&config,&profile)?;
    prepare_badge(state,settings)?;
    let mut hash=Sha256::new();hash.update(AGENT);hash.update(RUNTIME);hash.update(MEDIA);let version=format!("{:x}",hash.finalize());
    let runtime=state.root.join("runtime/kyro-hud").join(&version[..20]);owned_folder(&runtime,&state.root)?;
    let agent=runtime.join("kyro-hud-agent.jar");ensure(&agent,AGENT)?;ensure(&runtime.join("kyro-hud-runtime.jar"),RUNTIME)?;let media=runtime.join("kyro-media.exe");ensure(&media,MEDIA)?;
    migrate_legacy(&profile)?;
    Ok(vec![format!("-javaagent:{}",agent.display()),format!("-Dkyro.hud.config={}",config.display()),format!("-Dkyro.hud.media={}",media.display()),format!("-Dkyro.hud.appearance={}",state.root.join("runtime/kyro-hud/appearance.json").display()),"-Dkyro.hud.agent.active=true".into()])
}

fn mapping_names(text:&str)->Value{
    let mut result=json!({"mappingRevision":8,"moduleClasses":{},"moduleMembers":{},"moduleSignatures":{},"moduleFields":{}});let mut section="";let mut module_owner=String::new();
    for line in text.lines(){
        if line.trim_start().starts_with('#')||line.trim().is_empty(){continue;}
        if !line.starts_with(' '){
            section="";module_owner.clear();
            if let Some((name,obfuscated))=line.trim_end_matches(':').split_once(" -> "){
                result["moduleClasses"][name]=json!(obfuscated);
                if ["net.minecraft.client.Minecraft","net.minecraft.client.Options","net.minecraft.client.MouseHandler","net.minecraft.client.Camera","net.minecraft.client.renderer.GameRenderer","net.minecraft.client.renderer.LightTexture","net.minecraft.client.renderer.ScreenEffectRenderer","net.minecraft.client.renderer.ItemInHandRenderer","net.minecraft.client.gui.components.ChatComponent","net.minecraft.client.GuiMessage","net.minecraft.client.multiplayer.MultiPlayerGameMode","net.minecraft.client.renderer.entity.EntityRenderDispatcher","net.minecraft.client.renderer.entity.EntityRenderer","net.minecraft.client.renderer.entity.player.PlayerRenderer","net.minecraft.client.gui.screens.PauseScreen","net.minecraft.client.gui.screens.ConfirmScreen","net.minecraft.world.entity.Entity","net.minecraft.world.entity.LivingEntity","net.minecraft.world.entity.player.Player","net.minecraft.world.entity.player.Inventory","net.minecraft.world.item.ItemStack","net.minecraft.world.item.Item","net.minecraft.world.effect.MobEffectInstance","net.minecraft.world.effect.MobEffect","net.minecraft.world.phys.Vec3","net.minecraft.world.phys.AABB","net.minecraft.client.multiplayer.ClientLevel","net.minecraft.world.level.Level","net.minecraft.network.chat.Component","net.minecraft.network.chat.MutableComponent","net.minecraft.network.chat.TextComponent","net.minecraft.nbt.CompoundTag","net.minecraft.nbt.ListTag","net.minecraft.nbt.Tag","net.minecraft.core.component.DataComponents","net.minecraft.core.component.DataComponentHolder","net.minecraft.core.component.DataComponentGetter","net.minecraft.core.Holder","net.minecraft.core.Holder$Reference","net.minecraft.core.Holder$Direct","net.minecraft.world.item.component.ItemContainerContents","net.minecraft.world.item.component.BundleContents","com.mojang.blaze3d.vertex.PoseStack","net.minecraft.client.renderer.state.CameraRenderState","net.minecraft.client.renderer.entity.state.EntityRenderState","net.minecraft.client.renderer.entity.state.LivingEntityRenderState"].contains(&name){module_owner=name.to_owned();}
                if name=="net.minecraft.client.Minecraft"{section="client";result["clientClass"]=json!(obfuscated);}
                if name=="net.minecraft.client.multiplayer.PlayerInfo"||name=="net.minecraft.client.network.NetworkPlayerInfo"{section="entry";}
                if name=="net.minecraft.client.Options"{section="options";}
                if name=="net.minecraft.util.StringDecomposer"{result["textClass"]=json!(obfuscated);}
                if name=="net.minecraft.network.chat.Style"{section="style";}
                if name=="net.minecraft.client.multiplayer.ClientPacketListener"{section="connection";}
                if name=="net.minecraft.world.entity.player.Player"{section="player";}
                if name=="net.minecraft.world.entity.LivingEntity"{section="living";}
                if name=="net.minecraft.world.item.ItemStack"{section="stack";}
                if name=="net.minecraft.core.component.DataComponents"{section="components";result["componentsClass"]=json!(obfuscated);}
                if name=="net.minecraft.network.chat.Component"{result["componentClass"]=json!(obfuscated);}
                if name=="net.minecraft.client.gui.Gui"{result["guiClass"]=json!(obfuscated);}
                if name=="net.minecraft.core.component.DataComponentGetter"||name=="net.minecraft.core.component.DataComponentHolder"{section="componentGetter";}
                if name=="net.minecraft.world.item.component.ItemLore"{section="lore";}
                if name=="net.minecraft.world.InteractionResult"{section="interaction";}
                if name=="net.minecraft.client.multiplayer.MultiPlayerGameMode"{section="gameMode";result["gameModeClass"]=json!(obfuscated);}
                if name=="net.minecraft.world.attribute.EnvironmentAttributeProbe"{section="skyProbe";result["skyProbeClass"]=json!(obfuscated);}
                if name=="net.minecraft.world.attribute.EnvironmentAttributes"{section="skyAttributes";result["skyAttributesClass"]=json!(obfuscated);}
                if ["net.minecraft.client.gui.components.ChatComponent","net.minecraft.client.gui.components.PlayerTabOverlay","net.minecraft.client.renderer.entity.EntityRenderer","net.minecraft.client.renderer.feature.NameTagFeatureRenderer"].contains(&name){if result.get("badgeScopes").is_none(){result["badgeScopes"]=json!([]);}result["badgeScopes"].as_array_mut().unwrap().push(json!(obfuscated));}
            }
        }else if let Some((field,obfuscated))=line.trim().split_once(" -> "){
            if !module_owner.is_empty(){
                let token=field.split_whitespace().last().unwrap_or("");
                let member=if let Some((before,_))=field.split_once('('){before.split_whitespace().last().unwrap_or("").rsplit(':').next().unwrap_or("")}else{token};
                if !member.is_empty(){let key=format!("{}#{}",module_owner,member);
                    if !field.contains('('){result["moduleFields"][&key]=json!({"name":obfuscated,"type":field.split_whitespace().next().unwrap_or("")});}
                    if let Some((before,after))=field.split_once('('){let returns=before.split_whitespace().next().unwrap_or("").rsplit(':').next().unwrap_or("");let params=after.split(')').next().unwrap_or("");let args:Vec<&str>=if params.is_empty(){vec![]}else{params.split(',').collect()};if result["moduleSignatures"].get(&key).is_none(){result["moduleSignatures"][&key]=json!([]);}result["moduleSignatures"][&key].as_array_mut().unwrap().push(json!({"name":obfuscated,"returns":returns,"args":args}));}
if result["moduleMembers"].get(&key).is_none(){result["moduleMembers"][&key]=json!([]);}let list=result["moduleMembers"][&key].as_array_mut().unwrap();if !list.iter().any(|x|x==obfuscated){list.push(json!(obfuscated));}}
            }
            if section=="gameMode"&&field.contains(" useItemOn("){result["useItemOnMethod"]=json!(obfuscated);}
            if section=="interaction"&&field.ends_with(" FAIL"){result["interactionFailField"]=json!(obfuscated);}
            if section=="components"&&field.ends_with(" ITEM_MODEL"){result["itemModelField"]=json!(obfuscated);}
            if section=="skyProbe"&&field.contains(" getValue("){result["skyValueMethod"]=json!(obfuscated);}
            for (name,key) in [("SKY_COLOR","skyAttributeField"),("FOG_COLOR","skyFogAttributeField"),("CLOUD_COLOR","skyCloudAttributeField"),("SUNRISE_SUNSET_COLOR","skySunsetAttributeField")]{if section=="skyAttributes"&&field.ends_with(&format!(" {name}")){result[key]=json!(obfuscated);}}
            for (owner,pattern,key) in [("player"," getCooldowns()","playerCooldownMethod"),("living"," getMainHandItem()","mainHandMethod"),("living"," getOffhandItem()","offHandMethod"),("living"," getItemInHand(","itemInHandMethod"),("stack"," getItem()","stackItemMethod"),("stack"," getHoverName()","stackNameMethod"),("stack"," getCount()","stackCountMethod"),("components"," LORE","loreField"),("components"," CUSTOM_MODEL_DATA","customModelField"),("componentGetter"," get(","componentGetMethod"),("lore"," lines()","loreLinesMethod"),("interaction"," consumesAction()","consumesActionMethod"),("gameMode"," useItem(","useItemMethod")]{if section==owner&&field.contains(pattern){result[key]=json!(obfuscated);}}
            if section=="client"&&field.ends_with(" player"){result["playerField"]=json!(obfuscated);}
            if section=="client"&&field.ends_with(" options"){result["optionsField"]=json!(obfuscated);}
            if section=="client"&&field.contains(" getConnection()"){result["clientConnectionMethod"]=json!(obfuscated);}
            if section=="connection"&&field.contains(" getOnlinePlayers()"){result["connectionPlayersMethod"]=json!(obfuscated);}
            if section=="entry"&&field=="int latency"{result["latencyField"]=json!(obfuscated);}
            if section=="options"&&field=="boolean hideGui"{result["hideField"]=json!(obfuscated);}
            if section=="style"&&field.contains(" withFont("){result["styleFontMethod"]=json!(obfuscated);}
            if section=="style"&&field.contains(" withColor(int)"){result["styleColorMethod"]=json!(obfuscated);}
        }
    }result
}
pub async fn prepare_mappings(state:&State,settings:&Settings,version:&Value){
    // Render/input hooks never require mappings. These optional aliases improve vanilla ping lookup.
    let Some(url)=version["downloads"]["client_mappings"]["url"].as_str()else{return;};
    let Some(sha)=version["downloads"]["client_mappings"]["sha1"].as_str()else{return;};
    let Ok(profile)=state.instance_dir(settings)else{return;};let names=profile.join("config/kyro-hud-mapping.json");if fs::read(&names).ok().and_then(|bytes|serde_json::from_slice::<Value>(&bytes).ok()).is_some_and(|value|value["mappingRevision"]==8&&value["sourceVersion"]==settings.version&&value["mappingSha"]==sha&&value["textClass"].is_string()&&value["styleFontMethod"].is_string()&&value["badgeScopes"].is_array()&&value["connectionPlayersMethod"].is_string()&&value["playerCooldownMethod"].is_string()){return;}
    let task=async{
        let cache=state.root.join("runtime/kyro-hud/mappings");owned_folder(&cache,&state.root)?;let file=cache.join(format!("{sha}.txt"));net::download(url,&file,Some(sha)).await?;
        let text=fs::read_to_string(&file).map_err(|e|e.to_string())?;let mut map=mapping_names(&text);map["sourceVersion"]=json!(settings.version);map["mappingSha"]=json!(sha);let bytes=serde_json::to_vec(&map).map_err(|e|e.to_string())?;replace_owned(&names,&bytes)
    };
    let _=tokio::time::timeout(std::time::Duration::from_secs(15),task).await;
}

#[cfg(test)]
mod tests{
    use super::*;
    fn fixture()->State{let root=std::env::temp_dir().join(format!("kyro-built-in-hud-{}",chrono::Utc::now().timestamp_nanos_opt().unwrap()));State{root,data:std::sync::Mutex::new(crate::store::Data::default()),busy:std::sync::Mutex::new(false)}}
    #[test]
    fn appearance_follows_launcher_changes(){
        let state=fixture();let mut settings=Settings::default();let args=prepare(&state,&settings).unwrap();
        assert!(args.iter().any(|arg|arg.starts_with("-Dkyro.hud.appearance=")));
        settings.theme="light".into();settings.accent="#52c7c3".into();sync_appearance(&state,&settings).unwrap();
        let file=state.root.join("runtime/kyro-hud/appearance.json");let value:Value=serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();assert_eq!(value["theme"],"light");assert_eq!(value["accent"],"#52c7c3");
        settings.theme="black".into();sync_appearance(&state,&settings).unwrap();let value:Value=serde_json::from_slice(&fs::read(file).unwrap()).unwrap();assert_eq!(value["theme"],"black");fs::remove_dir_all(state.root).unwrap();
    }
    #[test]
    fn badge_pack_preserves_user_packs_and_options(){
        let state=fixture();let settings=Settings::default();let profile=state.instance_dir(&settings).unwrap();fs::create_dir_all(&profile).unwrap();
        let legacy=profile.join("resourcepacks/kyro-badge/assets/minecraft/font");fs::create_dir_all(&legacy).unwrap();fs::write(legacy.join("default.json"),b"old KYRO default font").unwrap();
        let server=profile.join("resourcepacks/server-rp/assets/minecraft/font");fs::create_dir_all(&server).unwrap();fs::write(server.join("default.json"),b"server font retained").unwrap();
        let old="key_key.forward:key.keyboard.w\nresourcePacks:[\"vanilla\",\"file/UserPack.zip\"]\n";fs::write(profile.join("options.txt"),old).unwrap();
        prepare(&state,&settings).unwrap();prepare(&state,&settings).unwrap();
        let options=fs::read_to_string(profile.join("options.txt")).unwrap();assert!(options.contains("key_key.forward:key.keyboard.w"));assert!(options.contains("file/UserPack.zip"));assert_eq!(options.matches("file/kyro-badge").count(),1);
        assert_eq!(fs::read_to_string(profile.join("config/options-before-kyro-badge.txt")).unwrap(),old);
        let font:Value=serde_json::from_slice(&fs::read(profile.join("resourcepacks/kyro-badge/assets/kyro/font/badge.json")).unwrap()).unwrap();assert_eq!(font["providers"][0]["chars"][0],"\u{e100}");assert!(!profile.join("resourcepacks/kyro-badge/assets/minecraft/font/default.json").exists());
        assert!(profile.join("resourcepacks/kyro-badge/assets/kyro/textures/font/badge.png").is_file());assert_eq!(fs::read(server.join("default.json")).unwrap(),b"server font retained");fs::remove_dir_all(state.root).unwrap();
    }
    #[test]
    fn badge_pack_uses_modern_metadata_and_exports_real_pack_for_game_qa(){
        let state=fixture();let mut settings=Settings::default();settings.version="1.21.11".into();
        let version=state.root.join("versions/1.21.11");fs::create_dir_all(&version).unwrap();
        let mut jar=zip::ZipWriter::new(fs::File::create(version.join("1.21.11.jar")).unwrap());jar.start_file("version.json",zip::write::SimpleFileOptions::default()).unwrap();jar.write_all(br#"{"pack_version":{"resource_major":75,"resource_minor":0}}"#).unwrap();jar.finish().unwrap();
        prepare(&state,&settings).unwrap();let source=state.instance_dir(&settings).unwrap().join("resourcepacks/kyro-badge");
        let metadata:Value=serde_json::from_slice(&fs::read(source.join("pack.mcmeta")).unwrap()).unwrap();
        assert_eq!(metadata["pack"]["pack_format"],75);assert_eq!(metadata["pack"]["min_format"],json!([75,0]));assert!(metadata["pack"].get("supported_formats").is_none());
        // The native game check consumes these exact production-generated files,
        // rather than reconstructing metadata independently in JavaScript.
        let exported=Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("target/hud-pack-qa");
        for name in ["pack.mcmeta","assets/kyro/font/badge.json","assets/kyro/textures/font/badge.png"]{let target=exported.join(name);fs::create_dir_all(target.parent().unwrap()).unwrap();fs::copy(source.join(name),target).unwrap();}
        fs::remove_dir_all(state.root).unwrap();
    }
    #[test]
    fn every_version_and_loader_get_the_same_builtin_overlay(){
        let state=fixture();for version in ["1.13","1.16.5","1.20.1","1.21.11","26.3"]{for loader in ["Vanilla","Fabric","Quilt","Forge","NeoForge"]{
            let mut settings=Settings::default();settings.version=version.into();settings.loader=loader.into();let args=prepare(&state,&settings).unwrap();assert!(args[0].starts_with("-javaagent:"));assert!(args.iter().any(|arg|arg=="-Dkyro.hud.agent.active=true"));assert!(!state.instance_dir(&settings).unwrap().join("mods").exists());
        }}fs::remove_dir_all(state.root).unwrap();
    }
    #[test]
    fn upgrades_old_mod_without_touching_positions_or_other_mods(){
        let state=fixture();let mut settings=Settings::default();settings.version="1.21.11".into();settings.loader="Fabric".into();let profile=state.instance_dir(&settings).unwrap();fs::create_dir_all(profile.join("mods")).unwrap();fs::create_dir_all(profile.join("config")).unwrap();
        fs::write(profile.join("mods/kyro-hud.jar"),include_bytes!("../tests/fixtures/kyro-hud-v1.fixture")).unwrap();fs::write(profile.join("mods/other.jar"),b"unrelated").unwrap();fs::write(profile.join("config/kyro-panels.json"),b"{\"music\":false}").unwrap();
        prepare(&state,&settings).unwrap();prepare(&state,&settings).unwrap();assert!(!profile.join("mods/kyro-hud.jar").exists());assert_eq!(fs::read(profile.join("mods/other.jar")).unwrap(),b"unrelated");assert_eq!(fs::read(profile.join("config/kyro-panels.json")).unwrap(),b"{\"music\":false}");fs::remove_dir_all(state.root).unwrap();
    }
    #[test]
    fn official_names_are_optional_and_parse_into_small_alias_table(){let names=mapping_names("net.minecraft.client.Minecraft -> abc:\n# {fileName: Minecraft.java}\n    net.minecraft.client.player.LocalPlayer player -> p\nnet.minecraft.client.multiplayer.PlayerInfo -> xyz:\n    int latency -> l\nnet.minecraft.world.attribute.EnvironmentAttributeProbe -> probe:\n    java.lang.Object getValue(net.minecraft.world.attribute.EnvironmentAttribute,float) -> v\nnet.minecraft.world.attribute.EnvironmentAttributes -> attrs:\n    net.minecraft.world.attribute.EnvironmentAttribute SKY_COLOR -> s\n    net.minecraft.world.attribute.EnvironmentAttribute FOG_COLOR -> f\n    net.minecraft.world.attribute.EnvironmentAttribute WATER_FOG_COLOR -> w\n");assert_eq!(names["clientClass"],"abc");assert_eq!(names["playerField"],"p");assert_eq!(names["latencyField"],"l");assert_eq!(names["skyProbeClass"],"probe");assert_eq!(names["skyValueMethod"],"v");assert_eq!(names["skyAttributeField"],"s");assert_eq!(names["skyFogAttributeField"],"f");assert_eq!(names["mappingRevision"],8);}
    #[test]
    fn bootstrap_runtime_contains_java8_classes_without_game_dependencies(){
        let mut jar=zip::ZipArchive::new(std::io::Cursor::new(RUNTIME)).unwrap();let mut bytes=Vec::new();jar.by_name("ru/kyro/builtin/AgentHooks.class").unwrap().read_to_end(&mut bytes).unwrap();assert_eq!(u16::from_be_bytes([bytes[6],bytes[7]]),52);assert!(jar.by_name("fabric.mod.json").is_err());assert!(jar.by_name("ru/kyro/internal/gson/Gson.class").is_ok());
    }
    #[test]
    fn module_mappings_export_for_game_qa(){
        if let Ok(folder)=std::env::var("KYRO_MODULE_TEST_DIR"){
            for entry in fs::read_dir(folder).unwrap(){let dir=entry.unwrap().path();let input=dir.join("client.txt");if input.exists(){let text=fs::read_to_string(input).unwrap();fs::write(dir.join("mapping.json"),serde_json::to_vec(&mapping_names(&text)).unwrap()).unwrap();}}
        }
        let Ok(file)=std::env::var("KYRO_MODULE_TEST_MAP")else{return;};
        let text=fs::read_to_string(file).unwrap();let map=mapping_names(&text);
        assert!(map["moduleClasses"]["net.minecraft.client.Minecraft"].is_string());
        assert!(map["moduleMembers"]["net.minecraft.client.Minecraft#player"].is_array());
        assert!(map["moduleMembers"]["net.minecraft.client.gui.components.ChatComponent#addMessage"].is_array());
        let target=Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("target/module-mappings-qa.json");fs::write(target,serde_json::to_vec(&map).unwrap()).unwrap();
    }

}
