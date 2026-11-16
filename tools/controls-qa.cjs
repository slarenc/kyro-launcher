const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const path = require('path');
const fs = require('fs');
(async () => {
 const browser = await chromium.launch({headless:true,channel:'msedge'});
 const page = await browser.newPage({viewport:{width:1060,height:600},deviceScaleFactor:1});
 const errors=[];page.on('pageerror',e=>{errors.push(e.message);console.error('UI error:',e.message);});
 await page.addInitScript(() => {
   const settings={auto_java:true,home_layout:'orbit',home_background:'',username:'Nova',version:'1.21.1',loader:'Fabric',memory_gb:6,java_path:'java',jvm_args:'-XX:+UseG1GC',download_threads:8,launch_behavior:'hide',language:'ru',theme:'black',accent:'#a185cf',microsoft_client_id:''};
   const mods=[{icon_url:'https://cdn.modrinth.com/data/P7dR8mSH/icon.png',filename:'sodium.jar',name:'Sodium',enabled:true,size:2000000,version_number:'1.0',dependencies:[{id:'api',name:'Fabric API'}],update:{id:'v2',version:'1.1'}}];
   const data={settings,sessions:[],notices:[],installed:[],modpacks:[],servers:[{id:'ares',name:'AresMine',address:'mc.aresmine.ru:25565'},{id:'virtus',name:'VirtusMine',address:'jux.virtusmine.fun:25565'}],offline_accounts:[],selected_account:null};
   const prepared=[{version:'1.21.1',loader:'Fabric',prepared_at:'2026-11-01T12:00:00Z'}];
   window.__TAURI__={core:{invoke:async(name,args)=>{
    if(name==='save_server'){if(data.servers.length>=3)throw Error('Лимит 3');data.servers.push({id:String(Date.now()),...args});return;}
    if(name==='remove_server'){data.servers=data.servers.filter(s=>s.id!==args.id);return;}
    if(name==='launch_server'){window.__serverLaunch=args.id;return;}
    if(name==='search_curseforge'){window.__cfSearch=args;return {hits:[{project_id:'cf-42',title:'CurseForge Test',description:'Test',author:'Author',downloads:42}],total_hits:1};}
    if(name==='server_status')return {online:true,players:22,icon:window.__qaIcon};
    if(name==='snapshot'){const a=data.offline_accounts.find(a=>'offline:'+a.uuid===data.selected_account);return {data:structuredClone(data),account:a?{...a,kind:'offline'}:null,paths:{launcher:'C:/Kyro',data:'C:/Kyro/data',profile:`C:/Kyro/data/instances/${settings.version}-${settings.loader}`},busy:false};}
    if(name==='offline_account'){
      if(args.action==='add'){const uuid=args.value;data.offline_accounts.push({name:args.value,uuid});data.selected_account='offline:'+uuid;}
      if(args.action==='select')data.selected_account=args.value;
      if(args.action==='remove'){data.offline_accounts=data.offline_accounts.filter(a=>'offline:'+a.uuid!==args.value);if(data.selected_account===args.value)data.selected_account=data.offline_accounts.length?'offline:'+data.offline_accounts[0].uuid:null;}
      return;
    }
    if(name==='versions')return ['1.21.1','1.21','1.20.6'];
    if(name==='version_library')return {releases:[{id:'26.3',date:'2026-09-15T12:00:00Z'},{id:'1.21.1',date:'2024-08-08T12:00:00Z'},{id:'1.21',date:'2024-06-13T12:00:00Z'}],prepared:structuredClone(prepared)};
    if(name==='verify_files'){if(!prepared.some(p=>p.version===settings.version&&p.loader===settings.loader))prepared.push({version:settings.version,loader:settings.loader,prepared_at:new Date().toISOString()});return;}
    if(name==='save_settings'){Object.assign(settings,args.settings);return;}
    if(name==='search_modrinth'){window.__kyroLastSearch=args;return {hits:[{project_id:'sodium',title:'Sodium',description:'Оптимизация рендеринга и повышение FPS',author:'CaffeineMC',downloads:1000000}],total_hits:1};}
    if(name==='install_mod'){
      if(args.kind==='modpack'){
        if(window.__failPackInstall)throw new Error('Контрольная сумма не совпала');
        const pack={id:'pack-v1',project_id:args.project,version_id:'v1',name:args.name,version_name:'1.0',game_version:settings.version,loader:settings.loader,loader_version:'0.16.9'};
        data.modpacks.push(pack);Object.assign(settings,{instance_id:pack.id,loader_version:pack.loader_version});return;
      }
      data.installed.push({project_id:args.project,name:args.name,filename:'sodium.jar',kind:'mod',game_version:settings.version,loader:settings.loader});return;
    }
    if(name==='select_modpack'){const p=data.modpacks.find(p=>p.id===args.id);Object.assign(settings,p?{instance_id:p.id,loader_version:p.loader_version,version:p.game_version,loader:p.loader}:{instance_id:'',loader_version:''});return;}
    if(name==='launch_game'){window.__launchedPack=settings.instance_id;window.__gameLaunchCount=(window.__gameLaunchCount||0)+1;return;}
    if(name==='check_compatibility')return {fingerprint:String(window.__compatProblems),checked:2,errors:window.__compatProblems?1:0,warnings:0,issues:window.__compatProblems?[{severity:'error',file:'wrong-core.jar',title:'Другое ядро',detail:'Мод рассчитан на Forge, выбран Fabric.'}]:[],scope:'Проверка локальных манифестов'};
    if(name==='save_server'){data.servers.push({id:'server1',name:args.name,address:args.address});return;}
    if(name==='remove_server'){data.servers=data.servers.filter(s=>s.id!==args.id);return;}
    if(name==='server_status')return {online:true,players:3,max:20,version:'1.21.1',latency:18,description:'Тестовый сервер'};
    if(name==='launch_server'){window.__connectedServer=args.id;return;}
    if(name==='list_screenshots')return {images:[{name:'2026-11-07_test.png',modified:Date.now(),size:1000}]};
    if(name==='screenshot_image')return 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jVVkAAAAASUVORK5CYII=';
    if(name==='diagnose_launch')return {title:'Найдены признаки причины вылета',ok:false,record:{error:'OutOfMemoryError'},findings:[{title:'Не хватило памяти',detail:'Java сообщила об ошибке выделения памяти.',action:'java'}],evidence:['java.lang.OutOfMemoryError: Java heap space'],note:'Локальная диагностика'};
    if(name==='apply_graphics_preset'){window.__appliedPreset=args.id;return {id:args.id};}
    if(name==='restore_graphics_preset'){window.__appliedPreset=null;return;}
    if(name==='scan_mods')return {files:[{...mods[0],icon_url:window.__qaIcons.sodium},{filename:'fabric-api-0.141.6+1.21.11.jar',name:'Fabric API',icon_url:window.__qaIcons['fabric-api'],enabled:true,size:2400000,version_number:'0.141.6',dependencies:[]}],warning:''};
    if(name==='toggle_mod'){const m=mods.find(m=>m.filename===args.filename);m.enabled=args.enabled;m.filename=m.enabled?'sodium.jar':'sodium.jar.disabled';return;}
    if(name==='update_mod'){mods[0].update=null;mods[0].version_number='1.1';return;}
    if(name==='install_java'){settings.java_path='C:/Kyro/data/runtimes/java-21/bin/java.exe';return settings.java_path;}
    if(name==='set_pack_cover'){data.modpacks.find(p=>p.id===args.id).cover_image=args.image;return;}
    if(name==='check_launcher_update')return {current:'0.6.0',version:window.__mockUpdate?'0.7.0':'0.6.0',available:!!window.__mockUpdate,installable:!!window.__mockUpdate,notes:'Тестовое обновление'};
    if(name==='install_launcher_update'){window.__updateRequested=true;return;}
    if(name==='notices'){if(args.action==='clear')data.notices=[];return;}
    if(name==='fps_plan'){
     if(window.__fpsHold)await new Promise(resolve=>window.__fpsRelease=resolve);
     if(window.__fpsFail)throw new Error('Не удалось загрузить список. Проверь подключение.');
     return {label:'1.21.1 · Fabric',rows:['Sodium','Embeddium','Lithium','FerriteCore','ImmediatelyFast','Entity Culling','ModernFix','More Culling'].map((name,i)=>({name,description:'Оптимизация отрисовки и использования памяти',status:i===1?'Уже выбран другой мод отрисовки':'Будет установлен'})),items:Array.from({length:6},(_,i)=>({project:'optimization-'+i}))};
    }
    if(name==='fps_install'){window.__fpsInstalled=true;return {installed:1};}
    if(name==='cancel_launch'){window.__launchCancelled=true;return true;}
    throw new Error('Unmocked command: '+name);
   }},event:{listen:async(name,callback)=>{(window.__listeners??={})[name]=callback;}},window:{getCurrentWindow:()=>({})}};
 });

 await page.addInitScript(icons=>{window.__qaIcon=icons.sodium;window.__qaIcons=icons;},Object.fromEntries(['sodium','fabric-api'].map(name=>[name,'data:image/png;base64,'+fs.readFileSync('output/ui-refinement/'+name+'.png').toString('base64')])));

 await page.goto('file:///'+path.resolve('ui/index.html').replaceAll('\\','/'));const assert=require('assert/strict');fs.mkdirSync('output/controls-qa',{recursive:true});
 await page.locator('[data-page="catalog"]').click();await page.locator('#catalog-loader-trigger').click();await page.waitForTimeout(180);await page.screenshot({path:'output/controls-qa/selectors.png'});await page.keyboard.press('Escape');
 await page.locator('[data-page="activity"]').click();const cell=page.locator('.heatmap [data-activity-date]').nth(150);await cell.hover();await page.locator('#activity-tooltip').waitFor();assert.equal(await cell.getAttribute('title'),null);await page.waitForTimeout(180);await page.screenshot({path:'output/controls-qa/activity.png'});
 await page.keyboard.press('Escape');assert.equal(await page.locator('#activity-tooltip').count(),0);await cell.focus();await page.locator('#activity-tooltip').waitFor();await page.locator('[data-page="play"]').click();assert.equal(await page.locator('#activity-tooltip').count(),0);
 for(const size of [{width:1060,height:600},{width:960,height:600}]){await page.setViewportSize(size);await page.evaluate(()=>{document.getElementById('operation').hidden=false;document.getElementById('operation-speed').hidden=false;document.getElementById('operation-speed').textContent='· 12 МБ/с';});assert(await page.evaluate(()=>document.documentElement.scrollHeight<=innerHeight&&document.documentElement.scrollWidth<=innerWidth));const account=await page.locator('#profile-button').boundingBox();assert(account.y+account.height<=size.height);}
 await browser.close();if(errors.length)throw Error(errors.join('\n'));console.log('PASS: tooltip pointer and keyboard, dismissal, update viewport overflow at 1060/960x600');
})().catch(e=>{console.error(e);process.exit(1);});