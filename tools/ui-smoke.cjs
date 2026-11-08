const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const path = require('path');
const fs = require('fs');
(async () => {
 const browser = await chromium.launch({headless:true,channel:'msedge'});
 const page = await browser.newPage({viewport:{width:1060,height:600},deviceScaleFactor:1});
 const errors=[];page.on('pageerror',e=>{errors.push(e.message);console.error('UI error:',e.message);});
 await page.addInitScript(() => {
   const settings={auto_java:true,home_layout:'orbit',home_background:'',username:'Nova',version:'1.21.1',loader:'Fabric',memory_gb:6,java_path:'java',jvm_args:'-XX:+UseG1GC',download_threads:8,launch_behavior:'hide',language:'ru',theme:'black',accent:'#a185cf',microsoft_client_id:''};
   const mods=[{filename:'sodium.jar',name:'Sodium',enabled:true,size:2000000,version_number:'1.0',dependencies:[{id:'api',name:'Fabric API'}],update:{id:'v2',version:'1.1'}}];
   const data={settings,sessions:[],notices:[],installed:[],modpacks:[],servers:[],offline_accounts:[],selected_account:null};
   const prepared=[{version:'1.21.1',loader:'Fabric',prepared_at:'2026-11-01T12:00:00Z'}];
   window.__TAURI__={core:{invoke:async(name,args)=>{
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
    if(name==='check_compatibility')return {checked:2,errors:window.__compatProblems?1:0,warnings:0,issues:window.__compatProblems?[{severity:'error',file:'wrong-core.jar',title:'Другое ядро',detail:'Мод рассчитан на Forge, выбран Fabric.'}]:[],scope:'Проверка локальных манифестов'};
    if(name==='save_server'){data.servers.push({id:'server1',name:args.name,address:args.address});return;}
    if(name==='remove_server'){data.servers=data.servers.filter(s=>s.id!==args.id);return;}
    if(name==='server_status')return {online:true,players:3,max:20,version:'1.21.1',latency:18,description:'Тестовый сервер'};
    if(name==='launch_server'){window.__connectedServer=args.id;return;}
    if(name==='list_screenshots')return {images:[{name:'2026-11-07_test.png',modified:Date.now(),size:1000}]};
    if(name==='screenshot_image')return 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jVVkAAAAASUVORK5CYII=';
    if(name==='diagnose_launch')return {title:'Найдены признаки причины вылета',ok:false,record:{error:'OutOfMemoryError'},findings:[{title:'Не хватило памяти',detail:'Java сообщила об ошибке выделения памяти.',action:'java'}],evidence:['java.lang.OutOfMemoryError: Java heap space'],note:'Локальная диагностика'};
    if(name==='apply_graphics_preset'){window.__appliedPreset=args.id;return {id:args.id};}
    if(name==='restore_graphics_preset'){window.__appliedPreset=null;return;}
    if(name==='scan_mods')return {files:structuredClone(mods),warning:''};
    if(name==='toggle_mod'){const m=mods.find(m=>m.filename===args.filename);m.enabled=args.enabled;m.filename=m.enabled?'sodium.jar':'sodium.jar.disabled';return;}
    if(name==='update_mod'){mods[0].update=null;mods[0].version_number='1.1';return;}
    if(name==='install_java'){settings.java_path='C:/Kyro/data/runtimes/java-21/bin/java.exe';return settings.java_path;}
    if(name==='set_pack_cover'){data.modpacks.find(p=>p.id===args.id).cover_image=args.image;return;}
    if(name==='check_launcher_update')return {current:'0.6.0',version:window.__mockUpdate?'0.7.0':'0.6.0',available:!!window.__mockUpdate,installable:!!window.__mockUpdate,notes:'Тестовое обновление'};
    if(name==='install_launcher_update'){window.__updateRequested=true;return;}
    if(name==='notices'){if(args.action==='clear')data.notices=[];return;}
    if(name==='fps_plan')return {label:'1.21.1 · Fabric',rows:[{name:'Sodium',description:'Оптимизация отрисовки',status:'Будет установлен'}],items:[{project:'sodium'}]};
    if(name==='fps_install'){window.__fpsInstalled=true;return {installed:1};}
    if(name==='cancel_launch'){window.__launchCancelled=true;return true;}
    throw new Error('Unmocked command: '+name);
   }},event:{listen:async(name,callback)=>{(window.__listeners??={})[name]=callback;}},window:{getCurrentWindow:()=>({})}};
 });
 await page.goto('file:///'+path.resolve('ui/index.html').replaceAll('\\','/'));
 fs.mkdirSync('output/app-preview',{recursive:true});
 await page.locator('#play').waitFor();
 await page.locator('#mod-updates-card strong').getByText('Доступно 1 обновление',{exact:true}).waitFor();
 if(await page.locator('#profile-button').evaluate(e=>parseFloat(getComputedStyle(e).borderTopLeftRadius))<10)throw new Error('Account card must have rounded corners');
 await page.locator('#mod-updates-card').click();
 await page.getByRole('heading',{name:'Мои моды',exact:true}).waitFor();
 await page.getByRole('button',{name:'Обновить · 1.1',exact:true}).waitFor();
 await page.locator('[data-kind="mod"]').click();
 await page.locator('[data-page="play"]').click();
 if(await page.locator('#game-loader-trigger').evaluate(e=>getComputedStyle(e).fontStyle)!=='normal')throw new Error('Fabric must be upright');
 await page.screenshot({path:'output/app-preview/01-home.png'});
 if(await page.locator('#page').evaluate(e=>e.scrollWidth>e.clientWidth))throw new Error('Home must not scroll horizontally');
 if(await page.locator('[data-page="friends"]').count())throw new Error('Friends must be removed');
 if(await page.getByText('Локальный профиль',{exact:true}).count())throw new Error('Local profile label must be removed');
 await page.evaluate(()=>document.fonts.load('600 14px Inter'));
 if(!await page.evaluate(()=>[...document.fonts].some(f=>f.family==='Inter'&&f.status==='loaded')))throw new Error('Bundled Inter font did not load');
 if(await page.locator('.loader-tabs button:disabled').count())throw new Error('Unavailable loader placeholders must be removed');
 await page.locator('#game-loader-trigger').click();
 await page.getByRole('listbox',{name:'Ядро Minecraft'}).waitFor();
 await page.screenshot({path:'output/app-preview/11-core-menu.png'});
 await page.getByRole('option',{name:'Quilt',exact:true}).click();
 await page.waitForFunction(()=>document.querySelector('#footer-version').textContent.includes('QUILT'));
 await page.locator('#game-version-trigger').click();
 await page.locator('#version-search').fill('26.3');
 await page.waitForFunction(()=>document.querySelectorAll('#version-dialog .version-row').length===5);
 await page.screenshot({path:'output/app-preview/12-versions.png'});
 await page.getByRole('button',{name:'Minecraft 26.3 NeoForge',exact:true}).click();
 await page.waitForFunction(()=>document.querySelector('#footer-version').textContent.includes('26.3 · NEOFORGE'));
 await page.locator('#game-version-trigger').click();await page.locator('#version-search').fill('1.21.1');
 await page.getByRole('button',{name:'Minecraft 1.21.1 Quilt',exact:true}).click();
 await page.locator('#all-versions').click();await page.locator('#downloaded-filter').click();
 await page.waitForFunction(()=>document.querySelectorAll('#version-dialog .version-row').length===1);
 await page.locator('#prepare-version').click();
 await page.waitForFunction(()=>document.querySelectorAll('#version-dialog .version-row').length===2);
 await page.screenshot({path:'output/app-preview/13-downloaded.png'});
 await page.getByRole('button',{name:'Minecraft 1.21.1 Fabric',exact:true}).click();
 if(await page.locator('select:visible').count())throw new Error('Native Windows selectors remain visible');
 const cores=await page.locator('#game-loader option').allTextContents();
 if(cores.join(',')!=='Vanilla,Fabric,Quilt,Forge,NeoForge')throw new Error('Core dropdown incomplete');
 await page.locator('#game-loader').selectOption('Quilt',{force:true});
 await page.waitForFunction(()=>document.querySelector('#footer-version').textContent.includes('QUILT'));
 await page.locator('#game-loader').selectOption('NeoForge',{force:true});
 await page.waitForFunction(()=>document.querySelector('#footer-version').textContent.includes('NEOFORGE'));
 await page.locator('#game-loader').selectOption('Fabric',{force:true});
 await page.locator('#profile-button').click();
 await page.locator('#offline-name').fill('KyroTestA');
 await page.getByRole('button',{name:'Добавить офлайн-аккаунт',exact:true}).click();
 await page.waitForFunction(()=>document.querySelector('#account-name').textContent==='KyroTestA');
 await page.locator('#profile-button').click();await page.locator('#offline-name').fill('KyroTestB');await page.getByRole('button',{name:'Добавить офлайн-аккаунт',exact:true}).click();
 await page.waitForFunction(()=>document.querySelector('#account-name').textContent==='KyroTestB');
 await page.locator('#profile-button').click();await page.locator('[data-select-account="offline:KyroTestA"]').click();
 await page.waitForFunction(()=>document.querySelector('#account-name').textContent==='KyroTestA');
 await page.locator('#profile-button').click();
 await page.screenshot({path:'output/app-preview/08-accounts.png'});
 await page.locator('[data-remove-account="offline:KyroTestA"]').click();
 await page.waitForFunction(()=>document.querySelector('#account-name').textContent==='KyroTestB');
 await page.locator('[data-remove-account="offline:KyroTestB"]').click();
 await page.waitForFunction(()=>document.querySelector('#account-name').textContent==='Аккаунты');await page.locator('#dialog-close').click();
 for(const [key,file]of [['activity','02-activity'],['catalog','03-catalog'],['notifications','04-notifications'],['settings','05-settings']]){
   await page.locator(`[data-page="${key}"]`).click();
   if(key==='catalog')await page.getByText('Sodium',{exact:true}).waitFor();
   await page.screenshot({path:`output/app-preview/${file}.png`});
   const overflow=await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth);
   if(overflow)throw new Error('Horizontal overflow in '+key);
 }
 if(await page.locator('input[type="color"]').count())throw new Error('Native accent picker must be removed');
 if(await page.locator('.palette button').count()!==9)throw new Error('Palette missing');
 await page.locator('[data-setting="accent"][data-value="#6b9fff"]').click();
 await page.waitForTimeout(450);
 if(await page.evaluate(()=>getComputedStyle(document.documentElement).getPropertyValue('--accent').trim())!=='#6b9fff')throw new Error('Palette did not apply');
 await page.locator('[data-page="play"]').click();
 if(await page.locator('#play').evaluate(e=>getComputedStyle(e).backgroundColor)!=='rgb(107, 159, 255)')throw new Error('Play button must follow palette');
 if(await page.locator('.home-note').evaluate(e=>parseFloat(getComputedStyle(e).fontSize))<13)throw new Error('Home note too small');
 if(await page.locator('.sidebar').evaluate(e=>e.getBoundingClientRect().width)>205)throw new Error('Sidebar too wide');
 if(await page.locator('#navigation').evaluate(e=>parseFloat(getComputedStyle(e).gap))>8)throw new Error('Menu items must have compact gaps');
 if(await page.evaluate(()=>innerHeight-document.querySelector('.profile').getBoundingClientRect().bottom)>30)throw new Error('Account must stay near window bottom');
 await page.screenshot({path:'output/app-preview/21-blue-home.png'});
 await page.locator('[data-page="settings"]').click();
 await page.locator('[data-setting="accent"][data-value="#a185cf"]').click();await page.waitForTimeout(450);
 await page.locator('[data-settings-tab="downloads"]').click();
 const launcher=await page.locator('[data-folder="launcher"] small').textContent();const profile=await page.locator('[data-folder="instance"] small').textContent();
 if(launcher===profile||!profile.includes('/instances/'))throw new Error('Folder paths must be distinct');
 await page.screenshot({path:'output/app-preview/09-folders.png'});
 await page.locator('[data-settings-tab="general"]').click();await page.screenshot({path:'output/app-preview/10-palette.png'});
 await page.locator('[data-settings-tab="java"]').click();
 await page.locator('[data-setting="memory_gb"][data-value="8"]').click();
 await page.waitForTimeout(500);
 if(!await page.locator('[data-setting="memory_gb"][data-value="8"]').evaluate(e=>e.classList.contains('selected')))throw new Error('RAM selector failed');
 await page.screenshot({path:'output/app-preview/06-java.png'});
 await page.locator('[data-page="catalog"]').click();
 await page.locator('#catalog-version').selectOption('1.21',{force:true});
 await page.waitForFunction(()=>window.__kyroLastSearch.version==='1.21');
 await page.locator('#catalog-loader').selectOption('Vanilla',{force:true});
 await page.waitForFunction(()=>window.__kyroLastSearch.loader==='Vanilla');
 await page.locator('#catalog-loader').selectOption('Fabric',{force:true});
 await page.waitForFunction(()=>window.__kyroLastSearch.loader==='Fabric');
 await page.locator('#catalog-version').selectOption('1.21.1',{force:true});
 await page.waitForFunction(()=>window.__kyroLastSearch.version==='1.21.1');
 await page.getByRole('button',{name:'Установить',exact:true}).click();
 await page.locator('[data-kind="installed"]').click();
 await page.getByText('sodium.jar',{exact:true}).waitFor();
 await page.locator('[data-page="activity"]').click();
 if(await page.locator('.heatmap .cell').count()!==371)throw new Error('Calendar must have 53 x 7 cells');
 await page.setViewportSize({width:960,height:600});
 await page.screenshot({path:'output/app-preview/07-compact.png'});
 await page.locator('[data-page="catalog"]').click();
 await page.locator('[data-kind="modpack"]').click();
 await page.getByRole('button',{name:'Установить',exact:true}).waitFor();
 await page.evaluate(()=>window.__failPackInstall=true);
 await page.getByRole('button',{name:'Установить',exact:true}).click();
 await page.waitForFunction(()=>document.querySelector('#toast').textContent.includes('Контрольная сумма'));
 if(await page.locator('#active-modpack-trigger').count())throw new Error('Failed install must not select a pack');
 await page.evaluate(()=>window.__failPackInstall=false);
 await page.getByRole('button',{name:'Установить',exact:true}).click();
 await page.locator('#active-modpack-trigger').waitFor();
 await page.locator('#play').click();
 await page.waitForFunction(()=>window.__launchedPack==='pack-v1');
 await page.locator('[data-page="catalog"]').click();
 await page.getByRole('button',{name:'Запустить',exact:true}).waitFor();
 await page.locator('[data-kind="installed"]').click();
 await page.getByRole('button',{name:'Запустить',exact:true}).waitFor();
 if(await page.getByText('sodium.jar',{exact:true}).count())throw new Error('Ordinary profile mods leaked into pack');
 await page.getByRole('button',{name:'Запустить',exact:true}).click();
 await page.locator('#active-modpack-trigger').waitFor();
 await page.screenshot({path:'output/app-preview/13-modpack-home.png'});
 await page.locator('#active-modpack').selectOption('',{force:true});
 await page.waitForFunction(()=>!document.querySelector('#game-loader-trigger').disabled);
 await page.locator('[data-page="catalog"]').click();
 await page.locator('[data-kind="manage"]').click();
 await page.getByRole('checkbox',{name:'Включить Sodium'}).waitFor();
 await page.getByRole('checkbox',{name:'Включить Sodium'}).uncheck();
 await page.getByText('sodium.jar.disabled',{exact:true}).waitFor();
 await page.getByRole('checkbox',{name:'Включить Sodium'}).check();
 await page.locator('#refresh-mods').click();
 await page.getByRole('button',{name:'Обновить · 1.1',exact:true}).click();
 await page.waitForFunction(()=>!document.querySelector('[data-update-mod]'));
 await page.getByText('Обновлений нет',{exact:true}).waitFor();
 await page.screenshot({path:'output/app-preview/14-mod-manager.png'});
 await page.locator('[data-page="settings"]').click();
 await page.locator('[data-settings-tab="general"]').click();
 await page.locator('[data-home-layout="card"]').click();
 const png=Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jVVkAAAAASUVORK5CYII=','base64');
 await page.locator('#background-file').setInputFiles({name:'background.png',mimeType:'image/png',buffer:png});
 await page.locator('#clear-background:enabled').waitFor();
 await page.locator('[data-page="play"]').click();
 if(!await page.locator('.home.home-card.has-image').count())throw new Error('Home customization not applied');
 await page.screenshot({path:'output/app-preview/15-custom-home.png'});
 await page.locator('[data-page="settings"]').click();
 await page.locator('[data-settings-tab="java"]').click();
 await page.locator('#auto-java').uncheck();
 await page.locator('#install-java').click();
 await page.waitForFunction(()=>document.querySelector('#java-path').value.includes('/runtimes/'));
 await page.locator('[data-settings-tab="general"]').click();
 await page.evaluate(()=>window.__mockUpdate=true);
 await page.locator('#check-kyro-update').click();
 await page.locator('#install-kyro-update').click();
 await page.waitForFunction(()=>window.__updateRequested===true);
 await page.locator('[data-settings-tab="game"]').click();
 await page.locator('[data-preset="fps"]').click();
 await page.waitForFunction(()=>window.__appliedPreset==='fps');
 await page.screenshot({path:'output/app-preview/16-graphics-presets.png'});
 await page.locator('#restore-preset').click();
 await page.waitForFunction(()=>window.__appliedPreset===null);
 if(await page.locator('[data-page="servers"]').count())throw new Error('Servers tab must be removed');
 await page.locator('[data-page="screenshots"]').click();
 await page.locator('[data-view-shot="0"]').click();
 await page.locator('#screenshot-view[open] img').waitFor();
 await page.keyboard.press('Escape');
 await page.screenshot({path:'output/app-preview/18-screenshots.png'});
 await page.locator('[data-page="diagnostics"]').click();
 await page.getByRole('heading',{name:'Не хватило памяти',exact:true}).waitFor();
 await page.screenshot({path:'output/app-preview/19-diagnostics.png'});
 await page.locator('[data-diagnostic-action="java"]').click();
 await page.locator('#java-path').waitFor();
 await page.locator('[data-page="play"]').click();
 const launches=await page.evaluate(()=>window.__gameLaunchCount||0);
 await page.evaluate(()=>window.__compatProblems=true);
 await page.locator('#play').click();
 await page.locator('#compatibility-warning[open]').waitFor();
 await page.locator('#cancel-incompatible').click();
 await page.getByText('Есть явные несовместимости',{exact:true}).waitFor();
 if(await page.evaluate(()=>window.__gameLaunchCount||0)!==launches)throw new Error('Cancelled compatibility warning still launched game');
 await page.screenshot({path:'output/app-preview/20-compatibility.png'});
 await page.locator('[data-page="play"]').click();
 await page.locator('#play').click();
 await page.locator('#launch-anyway').click();
 await page.waitForFunction(n=>(window.__gameLaunchCount||0)===n+1,launches);
 await page.evaluate(()=>window.__compatProblems=false);
 await page.locator('[data-page="play"]').click();
 await page.locator('#fps-boost').click();
 await page.locator('#fps-install:not([disabled])').waitFor();
 await page.screenshot({path:'output/app-preview/22-fps-boost.png'});
 await page.locator('#fps-install').click();
 await page.waitForFunction(()=>window.__fpsInstalled===true);
 await page.waitForFunction(()=>document.querySelector('#operation').hidden);
 await page.evaluate(()=>{document.querySelector('#operation').hidden=false;window.__listeners['launch-state']({payload:{can_cancel:true,running:false}});});
 if(await page.locator('#play').innerText()!=='Отменить')throw new Error('Main launch button did not expose cancellation');
 await page.locator('#play').click();
 await page.waitForFunction(()=>window.__launchCancelled===true);
 await page.evaluate(()=>window.__listeners["launch-state"]({payload:{can_cancel:false,running:false}}));
 await page.locator('[data-page="settings"]').click();
 await page.locator('[data-settings-tab="general"]').click();
 await page.locator('#discord-enabled').uncheck();
 await page.locator('[data-page="play"]').click();
 await page.locator('[data-page="settings"]').click();
 if(await page.locator('#discord-enabled').isChecked())throw new Error('Discord setting did not persist');
 await browser.close();
 if(errors.length)throw new Error(errors.join('\n'));
 console.log('PASS: navigation, upright Fabric, catalog, install state, settings, 371-day grid, FPS boost, launch cancellation, Discord persistence, no JS errors.');
})().catch(e=>{console.error(e);process.exit(1);});
