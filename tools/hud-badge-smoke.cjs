const fs=require('node:fs'),path=require('node:path'),{spawnSync}=require('node:child_process');
const root=path.resolve(__dirname,'..'),cache=path.join(process.env.LOCALAPPDATA,'KyroLauncher');
const metadata=JSON.parse(fs.readFileSync(path.join(process.env.USERPROFILE,'.gradle/caches/fabric-loom/1.21.11/mojang_minecraft_info.json'),'utf8'));
const libraries=metadata.libraries.flatMap(lib=>lib.downloads?.artifact?[path.join(cache,'libraries',lib.downloads.artifact.path)]:[]).filter(file=>fs.existsSync(file));
const java='C:/Program Files/Eclipse Adoptium/jdk-21.0.13.9-hotspot/bin/java.exe';
const variants=[['vanilla','bhg',path.join(cache,'versions/1.21.11/1.21.11.jar')],['intermediary','net.minecraft.class_5223',path.join(process.env.USERPROFILE,'.gradle/caches/fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-intermediary/1.21.11-net.fabricmc.yarn.1_21_11.1.21.11+build.6-v2/minecraft-merged-intermediary-1.21.11-net.fabricmc.yarn.1_21_11.1.21.11+build.6-v2.jar')]];
for(const [kind,clazz,game] of variants){
 const config=path.join(root,'target/badge-qa',kind);fs.mkdirSync(config,{recursive:true});fs.writeFileSync(path.join(config,'kyro-hud-mapping.json'),JSON.stringify({textClass:clazz,styleFontMethod:'a',styleColorMethod:'a'}));
 const args=['-javaagent:'+path.join(root,'src-tauri/resources/kyro-hud-agent.jar'),'-Dkyro.hud.player=WannaForget','-Dkyro.hud.config='+config,'-Dkyro.hud.media='+path.join(root,'src-tauri/resources/kyro-media.exe'),'-cp',[path.join(root,'components/hud/build/classes/java/test'),game,...libraries].join(path.delimiter),'ru.kyro.hudtest.ActualTextHarness',clazz];
 const result=spawnSync(java,args,{cwd:root,stdio:'inherit',windowsHide:true,timeout:180000});if(result.error)throw Error(kind+' badge check: '+result.error.code);if(result.status!==0)process.exit(result.status||1);
}
