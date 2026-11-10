const fs=require('node:fs'),path=require('node:path'),{spawnSync}=require('node:child_process');
const root=path.resolve(__dirname,'..'),cache=path.join(process.env.LOCALAPPDATA,'KyroLauncher');
const tiny=fs.readFileSync(path.join(process.env.USERPROFILE,'.gradle/caches/fabric-loom/1.21.11/loom.mappings.1_21_11.layered+hash.2198-v2/mappings.tiny'),'utf8');
const classes=new Map();let current;
for(const line of tiny.split(/\r?\n/)){const p=line.split('\t');if(p[0]==='c'){current={obf:p[1],inter:p[2],members:[]};classes.set(p[3],current);}else if((p[1]==='m'||p[1]==='f')&&current)current.members.push({kind:p[1],desc:p[2],obf:p[3],inter:p[4],named:p[5]});}
const spec={sharedClass:['net/minecraft/SharedConstants',{detectMethod:['m','tryDetectVersion']}],bootstrapClass:['net/minecraft/server/Bootstrap',{bootstrapMethod:['m','bootStrap']}],itemsClass:['net/minecraft/world/item/Items',{pearlField:['f','ENDER_PEARL']}],stackClass:['net/minecraft/world/item/ItemStack',{stackItemMethod:['m','getItem'],stackNameMethod:['m','getHoverName'],stackCountMethod:['m','getCount'],stackSetMethod:['m','set']}],componentsClass:['net/minecraft/core/component/DataComponents',{loreField:['f','LORE'],customModelField:['f','CUSTOM_MODEL_DATA']}],componentGetter:['net/minecraft/core/component/DataComponentGetter',{componentGetMethod:['m','get']}],componentClass:['net/minecraft/network/chat/Component',{literalMethod:['m','literal']}],loreClass:['net/minecraft/world/item/component/ItemLore',{loreLinesMethod:['m','lines']}],cooldownClass:['net/minecraft/world/item/ItemCooldowns',{addMethod:['m','addCooldown'],tickMethod:['m','tick']}],gameModeClass:['net/minecraft/client/multiplayer/MultiPlayerGameMode',{useItemMethod:['m','useItem']}]};
spec.skyProbeClass=['net/minecraft/world/attribute/EnvironmentAttributeProbe',{skyValueMethod:['m','getValue']}];
spec.skyAttributesClass=['net/minecraft/world/attribute/EnvironmentAttributes',{skyAttributeField:['f','SKY_COLOR'],skyFogAttributeField:['f','FOG_COLOR'],skyCloudAttributeField:['f','CLOUD_COLOR'],skySunsetAttributeField:['f','SUNRISE_SUNSET_COLOR']}];
const metadata=JSON.parse(fs.readFileSync(path.join(process.env.USERPROFILE,'.gradle/caches/fabric-loom/1.21.11/mojang_minecraft_info.json'),'utf8'));
const libraries=metadata.libraries.flatMap(lib=>lib.downloads?.artifact?[path.join(cache,'libraries',lib.downloads.artifact.path)]:[]).filter(fs.existsSync);
for(const kind of ['obf','inter']){
 const names={};
 for(const [key,[clazz,members]]of Object.entries(spec)){
  const c=classes.get(clazz);if(!c)throw Error(clazz);names[key]=c[kind].replaceAll('/','.');
  for(const [alias,[type,named]]of Object.entries(members)){
   const m=c.members.find(m=>m.kind===type&&m.named===named
    &&(alias!=='stackSetMethod'||m.desc.includes(';Ljava/lang/Object;)'))
    &&(alias!=='addMethod'||m.desc.startsWith('(L'+classes.get('net/minecraft/world/item/ItemStack').obf+';')));
   if(!m)throw Error(clazz+' '+named);names[alias]=m[kind];
  }
 }
 const config=path.join(root,'target/cooldowns-qa',kind);fs.mkdirSync(config,{recursive:true});fs.writeFileSync(path.join(config,'names.properties'),Object.entries(names).map(([k,v])=>k+'='+v).join('\n'));fs.writeFileSync(path.join(config,'kyro-hud-mapping.json'),JSON.stringify(names));
 const game=kind==='obf'?path.join(cache,'versions/1.21.11/1.21.11.jar'):path.join(process.env.USERPROFILE,'.gradle/caches/fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-intermediary/1.21.11-net.fabricmc.yarn.1_21_11.1.21.11+build.6-v2/minecraft-merged-intermediary-1.21.11-net.fabricmc.yarn.1_21_11.1.21.11+build.6-v2.jar');
 const testDir=path.join(config,'classes/ru/kyro/hudtest');fs.mkdirSync(testDir,{recursive:true});const compiled=path.join(root,'components/hud/build/classes/java/test/ru/kyro/hudtest');for(const file of fs.readdirSync(compiled).filter(f=>f.startsWith('ActualCooldownHarness')))fs.copyFileSync(path.join(compiled,file),path.join(testDir,file));
 const args=['-javaagent:'+path.join(root,'src-tauri/resources/kyro-hud-agent.jar'),'-Dkyro.hud.config='+config,'-cp',[path.join(config,'classes'),game,...libraries].join(path.delimiter),'ru.kyro.hudtest.ActualCooldownHarness',path.join(config,'names.properties')];
 const result=spawnSync('C:/Program Files/Eclipse Adoptium/jdk-21.0.13.9-hotspot/bin/java.exe',args,{cwd:root,stdio:'inherit',windowsHide:true,timeout:120000});if(result.error)throw result.error;if(result.status!==0)process.exit(result.status||1);
}
