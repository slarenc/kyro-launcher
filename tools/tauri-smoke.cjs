const {chromium}=require(process.env.PLAYWRIGHT_MODULE||'playwright');
const assert=require('node:assert/strict');
(async()=>{
 let browser;
 for(let i=0;i<120;i++){try{browser=await chromium.connectOverCDP('http://127.0.0.1:9228');break;}catch{await new Promise(r=>setTimeout(r,250));}}
 assert(browser,'WebView2 debugging endpoint unavailable');
 try{
 const context=browser.contexts()[0];let page;
 for(let i=0;i<120;i++){page=context.pages().find(p=>/tauri|localhost/.test(p.url()));if(page)break;await new Promise(r=>setTimeout(r,250));}
 assert(page,'Tauri page unavailable');
 const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.locator('#play').waitFor();
 const result=await page.evaluate(async()=>{
   const invoke=window.__TAURI__.core.invoke;
   let state=await invoke('snapshot');
   if(!state.root.includes('tauri-ui-smoke'))throw Error('Expected isolated data');
   await invoke('offline_account',{action:'add',value:'TauriPreview'});
   state=await invoke('snapshot');
   state.data.settings.accent='#67b7f7';
   await invoke('save_settings',{settings:state.data.settings});
   const saved=await invoke('snapshot');
   if(saved.account.name!=='TauriPreview'||saved.data.settings.accent!=='#67b7f7')throw Error('IPC persistence failed');
   const checks=await Promise.all(['check_compatibility','list_screenshots','diagnose_launch','cancel_launch'].map(name=>invoke(name)));
   return {account:saved.account.name,checks:checks.length};
 });
 await page.reload();await page.locator('#play').waitFor();
 assert.equal(await page.locator('#account-name').textContent(),'TauriPreview');
 assert.equal(await page.locator('#kyro-hud,#kyro-panels-dialog').count(),0);

 for(const name of ['catalog','screenshots','diagnostics','activity','notices','settings','play']){const link=page.locator('[data-page="'+name+'"]');if(await link.count())await link.click();}
 await page.screenshot({path:'output/app-preview/23-tauri-native.png'});
 assert.deepEqual(errors,[]);
 console.log('PASS: real Tauri IPC, isolated state, accounts, settings, local services, automatic HUD interface, navigation.',result);
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
