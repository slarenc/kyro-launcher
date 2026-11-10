// Launch the cached official vanilla client in a copied QA world, without any mod loader.
const fs=require('node:fs'),path=require('node:path'),{spawn}=require('node:child_process');
const root=path.resolve(__dirname,'..');
(async()=>{
 const skyQa=process.env.KYRO_SKY_QA==='1';
 const cache=process.env.KYRO_TEST_CACHE||path.join(process.env.LOCALAPPDATA,'KyroLauncher');
 const metadata=process.env.KYRO_TEST_META||path.join(process.env.USERPROFILE,'.gradle/caches/fabric-loom/1.21.11/mojang_minecraft_info.json');
 const v=JSON.parse(fs.readFileSync(metadata,'utf8'));const fixture=path.join(root,'target/vanilla-hud-qa');fs.mkdirSync(path.join(fixture,'config'),{recursive:true});
 const source=process.env.KYRO_TEST_WORLD||path.join(root,'target/legacy-hud-fixture/run/saves/New World');const world=path.join(fixture,'saves/KYRO HUD QA');if(!fs.existsSync(world))fs.cpSync(source,world,{recursive:true});
 fs.writeFileSync(path.join(fixture,'options.txt'),'fullscreen:false\npauseOnLostFocus:false\nguiScale:2\nmaxFps:30\nrenderDistance:3\nsimulationDistance:5\nsoundCategory_master:0.0\nresourcePacks:["file/kyro-badge","file/server-font-qa"]\n');
 const badge=path.join(fixture,'resourcepacks/kyro-badge'),production=path.join(root,'target/hud-pack-qa');if(!fs.existsSync(path.join(production,'pack.mcmeta')))throw Error('Run cargo test badge_pack_uses_modern_metadata_and_exports_real_pack_for_game_qa first');
 fs.cpSync(production,badge,{recursive:true});
 const legacy=path.join(badge,'assets/minecraft/font/default.json');if(fs.existsSync(legacy))fs.unlinkSync(legacy);
 // A higher-priority pack deliberately assigns the same code point a huge glyph.
 const server=path.join(fixture,'resourcepacks/server-font-qa');for(const dir of ['assets/minecraft/font','assets/server/textures/font'])fs.mkdirSync(path.join(server,dir),{recursive:true});
 fs.copyFileSync(path.join(root,'ui/kyro-logo.png'),path.join(server,'assets/server/textures/font/menu.png'));fs.writeFileSync(path.join(server,'pack.mcmeta'),JSON.stringify({pack:{pack_format:75,min_format:[75,0],max_format:[75,999],description:'Adversarial server font QA'}}));
 fs.writeFileSync(path.join(server,'assets/minecraft/font/default.json'),JSON.stringify({providers:[{type:'bitmap',file:'server:font/menu.png',height:64,ascent:56,chars:['\uE100']}]}));
 fs.writeFileSync(path.join(fixture,'config/kyro-hud-mapping.json'),JSON.stringify({textClass:'bhg',styleFontMethod:'a',styleColorMethod:'a',badgeScopes:['gjf','gkg','hwp','igk'],...(skyQa?{skyProbeClass:'ced',skyValueMethod:'a',skyAttributesClass:'ceg',skyAttributeField:'i',skyFogAttributeField:'a',skyCloudAttributeField:'k',skySunsetAttributeField:'j'}:{})}));
 fs.writeFileSync(path.join(fixture,'config/kyro-panels.json'),JSON.stringify({musicX:.7,musicY:.7,infoScale:.8,musicScale:.65,...(skyQa?{customSky:true,skyUseThemeColor:false,skyColor:0x8849cd,skyMix:.9,skyCloudColor:0xc4a0ed,skyCloudMix:.75}:{} )}));
 // A private datapack in the copied QA world prints a sample nickname in real chat.
 const data=path.join(world,'datapacks/kyro-badge-qa');for(const dir of ['data/minecraft/tags/function','data/kyro/function'])fs.mkdirSync(path.join(data,dir),{recursive:true});
 fs.writeFileSync(path.join(data,'pack.mcmeta'),JSON.stringify({pack:{pack_format:94,min_format:[94,1],max_format:[94,999],description:'Private KYRO badge check'}}));
 fs.writeFileSync(path.join(data,'data/minecraft/tags/function/tick.json'),JSON.stringify({values:['kyro:tick']}));const tag='kyro_qa_'+Date.now();
 const objective='kq'+Date.now().toString(36),once=`execute as @a[tag=!${tag}] run `;
 const skyCommands=skyQa?`${once}time set 6000\n${once}weather clear\n${once}gamemode spectator @s\n${once}tp @s 8 100 8 0 -20\n`:'';
 fs.writeFileSync(path.join(data,'data/kyro/function/tick.mcfunction'),`${once}scoreboard objectives add ${objective} dummy\n${once}scoreboard players set KyroBuiltinQA ${objective} 1\n${once}scoreboard objectives setdisplay sidebar ${objective}\n${once}tellraw @s {"text":"<KyroBuiltinQA> KYRO name badge check","color":"gold","bold":true}\n${skyCommands}tag @a add ${tag}\n`);
 function allowed(item){let yes=true;if(item.rules){yes=false;for(const rule of item.rules){if(rule.features)continue;const os=rule.os;if(os?.name&&os.name!=='windows'||os?.arch&&os.arch!=='x86_64')continue;yes=rule.action==='allow';}}return yes;}
 const cp=[];for(const lib of v.libraries.filter(allowed)){const artifact=lib.downloads?.artifact;if(!artifact)continue;const file=path.join(cache,'libraries',artifact.path);if(!fs.existsSync(file))throw Error('Missing cached library '+file);cp.push(file);}
 const jar=path.join(cache,'versions',v.id,v.id+'.jar');if(!fs.existsSync(jar))throw Error('Missing cached vanilla client');cp.push(jar);
 const natives=path.join(fixture,'natives');fs.mkdirSync(natives,{recursive:true});
 const vars={natives_directory:natives,launcher_name:'KYRO',launcher_version:'QA',classpath:cp.join(path.delimiter),library_directory:path.join(cache,'libraries'),classpath_separator:path.delimiter};
 const expand=s=>s.replace(/\$\{([^}]+)\}/g,(_,name)=>{if(vars[name]===undefined)throw Error('Unknown JVM token '+name);return vars[name];});
 const args=['-Xmx1G','-Xms256M'];for(const item of v.arguments.jvm){if(typeof item==='string')args.push(expand(item));else if(allowed(item))for(const value of Array.isArray(item.value)?item.value:[item.value])args.push(expand(value));}
 const reports=path.join(root,skyQa?'output/hud-agent-sky':'output/hud-agent-vanilla');fs.mkdirSync(reports,{recursive:true});
 args.push('-javaagent:'+path.join(root,'src-tauri/resources/kyro-hud-agent.jar'),'-Dkyro.hud.player=KyroBuiltinQA','-Dkyro.hud.config='+path.join(fixture,'config'),'-Dkyro.hud.media='+path.join(root,'src-tauri/resources/kyro-media.exe'),'-Dkyro.hud.agentPreview=true','-Dkyro.hud.previewHidden=true','-Dkyro.hud.requireWorld=true','-Dkyro.hud.capture='+reports);
 if(skyQa)args.push('-Dkyro.hud.skyTrace=true');
 args.push(v.mainClass,'--username','KyroBuiltinQA','--version',v.id,'--gameDir',fixture,'--assetsDir',path.join(cache,'assets'),'--assetIndex',v.assetIndex.id,'--uuid','24c15bb2a67a3fe6ad4843b13d57eb10','--accessToken','0','--userType','legacy','--versionType','release','--width','1280','--height','720','--quickPlaySingleplayer','KYRO HUD QA');
 const log=fs.openSync(path.join(reports,'game.log'),'w');const java=process.env.KYRO_TEST_JAVA||'C:/Program Files/Eclipse Adoptium/jdk-21.0.13.9-hotspot/bin/java.exe';
 const child=spawn(java,args,{cwd:fixture,windowsHide:true,stdio:['ignore',log,log]});fs.closeSync(log);
 const timer=setTimeout(()=>child.kill(),300000);let code;try{code=await new Promise((resolve,reject)=>{child.on('error',reject);child.on('exit',resolve);});}finally{clearTimeout(timer);}
 const text=fs.readFileSync(path.join(reports,'game.log'),'utf8');if(code!==0||!text.includes('[KYRO HUD] Game client:')||!text.includes('Captured preview 2 (world=true)')||!fs.existsSync(path.join(reports,'agent-2.png')))throw Error('Vanilla HUD verification failed; see '+path.join(reports,'game.log'));
 if(text.includes('Overlay disabled after a rendering error')||text.includes('Input dispatch failed'))throw Error('HUD runtime error in vanilla');
 if(!text.includes('Reloading ResourceManager: vanilla, file/kyro-badge, file/server-font-qa')||text.includes("Couldn't load file/kyro-badge pack metadata")||text.includes('Removed resource pack file/kyro-badge')||text.includes('Unknown font: kyro:badge'))throw Error('The production-generated badge pack was rejected by Minecraft');
 if(skyQa&&(!text.includes('Custom Sky applied SKY')||!text.includes('Custom Sky applied FOG')||text.includes('Custom Sky hook failed')))throw Error('Custom Sky did not change real game environment colours');
 console.log('PASS: official vanilla '+v.id+', copied world, no loader or HUD mod, built-in renderer and editor, screenshots.');
})().catch(error=>{console.error(error);process.exitCode=1;});
