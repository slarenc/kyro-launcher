const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const path = require('path');
const fs = require('fs');
(async () => {
 const browser = await chromium.launch({headless:true,channel:'msedge'});
 const page = await browser.newPage({viewport:{width:1320,height:850},deviceScaleFactor:1});
 const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(() => {
   const settings={username:'Nova',version:'1.21.1',loader:'Fabric',memory_gb:6,java_path:'java',jvm_args:'-XX:+UseG1GC',download_threads:8,launch_behavior:'hide',language:'ru',theme:'black',accent:'#a185cf',microsoft_client_id:''};
   const data={settings,sessions:[],notices:[],installed:[],offline_accounts:[],selected_account:null};
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
    if(name==='install_mod'){data.installed.push({project_id:args.project,name:args.name,filename:'sodium.jar',kind:'mod',game_version:settings.version,loader:settings.loader});return;}
    if(name==='notices'){if(args.action==='clear')data.notices=[];return;}
    throw new Error('Unmocked command: '+name);
   }},event:{listen:async()=>{}},window:{getCurrentWindow:()=>({})}};
 });
 await page.goto('file:///'+path.resolve('ui/index.html').replaceAll('\\','/'));
 fs.mkdirSync('output/app-preview',{recursive:true});
 await page.getByRole('button',{name:'PLAY',exact:true}).waitFor();
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
 await page.waitForFunction(()=>document.querySelectorAll('#version-dialog .version-row').length===4);
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
 if(cores.join(',')!=='Vanilla,Fabric,Quilt,NeoForge')throw new Error('Core dropdown incomplete');
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
 await page.setViewportSize({width:1000,height:700});
 await page.screenshot({path:'output/app-preview/07-compact.png'});
 await browser.close();
 if(errors.length)throw new Error(errors.join('\n'));
 console.log('PASS: navigation, upright Fabric, catalog, install state, settings, 371-day grid, no JS errors.');
})().catch(e=>{console.error(e);process.exit(1);});
