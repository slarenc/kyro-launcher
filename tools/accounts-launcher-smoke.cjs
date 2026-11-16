const fs=require('fs'),path=require('path'),assert=require('assert/strict'),{spawn}=require('child_process');
const sleep=ms=>new Promise(r=>setTimeout(r,ms));
(async()=>{
 const root=path.resolve('target/account-launcher-qa-'+Date.now()),cache=path.join(process.env.LOCALAPPDATA,'KyroLauncher');fs.mkdirSync(root,{recursive:true});
 for(const name of ['assets','libraries','versions'])fs.symlinkSync(path.join(cache,name),path.join(root,name),'junction');
 fs.cpSync(path.join(cache,'metadata'),path.join(root,'metadata'),{recursive:true});
 const settings={auto_java:false,version:'1.21.11',loader:'Vanilla',memory_gb:2,java_path:'C:/Program Files/Eclipse Adoptium/jdk-21.0.13.9-hotspot/bin/java.exe',launch_behavior:'none',discord_enabled:false};
 fs.writeFileSync(path.join(root,'state.json'),JSON.stringify({settings}));
 const app=spawn(path.resolve('dist/KyroLauncher-RightShift-Test.exe'),[],{windowsHide:true,stdio:'ignore',env:{...process.env,KYRO_DATA_DIR:root,WEBVIEW2_USER_DATA_FOLDER:path.join(root,'webview')}});let bridge;
 async function until(fn,timeout=180000){const deadline=Date.now()+timeout;while(Date.now()<deadline){try{const value=fn();if(value)return value;}catch{}await sleep(400);}throw Error('Timeout');}
 function read(p){return JSON.parse(fs.readFileSync(p,'utf8'));}
 const sessions=[];
 try{
  bridge=await until(()=>{const parent=path.join(root,'runtime/accounts');return fs.readdirSync(parent).map(n=>path.join(parent,n)).find(p=>fs.existsSync(path.join(p,'catalog.json')));});
  async function request(r){const name=require('crypto').randomUUID()+'.json';fs.writeFileSync(path.join(bridge,'requests',name),JSON.stringify(r));const result=await until(()=>fs.existsSync(path.join(bridge,name))&&read(path.join(bridge,name)));assert.equal(result.message,'Готово',result.message);}
  for(const name of ['ShiftQAOne','ShiftQATwo']){await request({action:'add',name});const account=await until(()=>read(path.join(bridge,'catalog.json')).accounts.find(a=>a.name===name));await request({action:'launch',key:account.key,server:''});const session=await until(()=>fs.readdirSync(bridge).filter(n=>fs.existsSync(path.join(bridge,n,'state.json'))).find(n=>read(path.join(bridge,n,'state.json')).name===name));sessions.push(session);console.log('PASS: HUD request launched '+name);}
  assert.equal(sessions.length,2);assert.notEqual(sessions[0],sessions[1]);
  for(const session of sessions){fs.writeFileSync(path.join(bridge,session,'command.json'),JSON.stringify({action:'disconnect'}));await until(()=>read(path.join(bridge,session,'ack.json')).action==='disconnect');}
  await until(()=>read(path.join(root,'state.json')).sessions?.length===2);const state=read(path.join(root,'state.json'));assert(state.sessions.every(s=>s.exit_code===0));
  fs.mkdirSync('output/hud-accounts-qa',{recursive:true});fs.writeFileSync('output/hud-accounts-qa/launcher-result.json',JSON.stringify({root,sessions,history:state.sessions},null,2));console.log('PASS: two actual Minecraft clients launched through HUD IPC and closed cleanly');
 }finally{if(bridge)for(const session of sessions)try{fs.writeFileSync(path.join(bridge,session,'command.json'),JSON.stringify({action:'disconnect'}));}catch{}await sleep(1000);app.kill();}
})().catch(e=>{console.error(e);process.exitCode=1;});
