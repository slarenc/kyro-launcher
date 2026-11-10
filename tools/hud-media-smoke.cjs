// Real Windows media-session fixture. Only a disposable browser profile and localhost are used.
const fs=require('node:fs');
const http=require('node:http');
const readline=require('node:readline');
const {spawn}=require('node:child_process');
const assert=require('node:assert/strict');
const {chromium}=require(process.env.PLAYWRIGHT_MODULE||'C:/Users/User/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const delay=ms=>new Promise(resolve=>setTimeout(resolve,ms));
(async()=>{
 const rate=8000,seconds=240,wav=Buffer.alloc(44+rate*seconds);
 wav.write('RIFF');wav.writeUInt32LE(wav.length-8,4);wav.write('WAVEfmt ',8);wav.writeUInt32LE(16,16);wav.writeUInt16LE(1,20);wav.writeUInt16LE(1,22);wav.writeUInt32LE(rate,24);wav.writeUInt32LE(rate,28);wav.writeUInt16LE(1,32);wav.writeUInt16LE(8,34);wav.write('data',36);wav.writeUInt32LE(rate*seconds,40);wav.fill(128,44);
 const art=fs.readFileSync('ui/kyro-logo.png');
 const html=`<!doctype html><title>KYRO media bridge QA</title><body style="background:#17141d;color:#cda2e9;font:18px sans-serif"><h1>KYRO / Media bridge QA</h1><p>Local silent test session. This window closes after verification.</p><audio id="audio" controls loop src="/silence.wav"></audio><script>
 const audio=document.querySelector('audio');let index=1;
 function meta(){navigator.mediaSession.metadata=new MediaMetadata({title:'KYRO bridge QA '+index,artist:'KYRO QA',artwork:[{src:location.origin+'/cover.png',sizes:'256x256',type:'image/png'}]});}
 function position(){if(Number.isFinite(audio.duration))navigator.mediaSession.setPositionState({duration:audio.duration,position:audio.currentTime,playbackRate:1});}
 navigator.mediaSession.setActionHandler('play',()=>audio.play());navigator.mediaSession.setActionHandler('pause',()=>audio.pause());
 navigator.mediaSession.setActionHandler('previoustrack',()=>{index--;meta()});navigator.mediaSession.setActionHandler('nexttrack',()=>{index++;meta()});
 navigator.mediaSession.setActionHandler('seekto',e=>{audio.currentTime=e.seekTime;position()});
 audio.onplay=()=>{navigator.mediaSession.playbackState='playing';position()};audio.onpause=()=>{navigator.mediaSession.playbackState='paused';position()};audio.ontimeupdate=position;
 meta();audio.play();</script>`;
 const server=http.createServer((req,res)=>{if(req.url==='/silence.wav'){
  const range=/bytes=(\d+)-(\d*)/.exec(req.headers.range||'');const start=range?Number(range[1]):0,end=range&&range[2]?Math.min(wav.length-1,Number(range[2])):wav.length-1;
  res.writeHead(range?206:200,{'Content-Type':'audio/wav','Accept-Ranges':'bytes','Content-Length':end-start+1,...(range?{'Content-Range':`bytes ${start}-${end}/${wav.length}`}:{})});res.end(wav.subarray(start,end+1));
 }else if(req.url==='/cover.png'){res.writeHead(200,{'Content-Type':'image/png'});res.end(art);}else{res.writeHead(200,{'Content-Type':'text/html'});res.end(html);}});
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 let browser,helper;
 try{
  browser=await chromium.launch({channel:'msedge',headless:false,args:['--autoplay-policy=no-user-gesture-required','--no-default-browser-check']});
  const page=await browser.newPage();await page.goto(`http://127.0.0.1:${server.address().port}/`);
  await page.locator('audio').evaluate(a=>a.play());
  helper=spawn('components/media/target/release/kyro-media.exe',[],{windowsHide:true});
  const lines=readline.createInterface({input:helper.stdout});const iterator=lines[Symbol.asyncIterator]();
  const request=async action=>{helper.stdin.write(JSON.stringify(action)+'\n');const response=await Promise.race([iterator.next(),delay(6000).then(()=>{throw Error('Bridge response timeout')})]);return JSON.parse(response.value);};
  let state;
  for(let i=0;i<25;i++){state=await request({action:'poll'});if(state.title?.startsWith('KYRO bridge QA'))break;await delay(300);}
  assert.ok(state.title?.startsWith('KYRO bridge QA'),'Browser fixture is visible to Windows media sessions');
  assert.equal(state.protocol,2);assert.ok(state.artwork,'Artwork returned');assert.ok(state.canSeek,'Seek supported');
  const source=state.source;
  await request({action:'toggle',session:source});await delay(500);state=await request({action:'poll',session:source});assert.equal(state.playing,false,'Pause');
  await request({action:'toggle',session:source});await delay(500);state=await request({action:'poll',session:source});assert.equal(state.playing,true,'Resume');
  const seekResult=await request({action:'seek',session:source,value:12000});await delay(800);state=await request({action:'poll',session:source});console.log(JSON.stringify({seekAccepted:seekResult.accepted,position:state.position,browserPosition:await page.locator('audio').evaluate(a=>a.currentTime)}));assert.ok(Math.abs(state.position-12000)<2500,'Seek position');
  await request({action:'next',session:source});await delay(400);state=await request({action:'poll',session:source});assert.equal(state.title,'KYRO bridge QA 2','Next track');
  await request({action:'previous',session:source});await delay(400);state=await request({action:'poll',session:source});assert.equal(state.title,'KYRO bridge QA 1','Previous track');
  const revision=state.artRevision;state=await request({action:'poll',session:source,artRevision:revision});assert.equal(state.artwork,undefined,'Unchanged artwork is not retransmitted');
  if(state.volume>=0){const volume=state.volume;state=await request({action:'volume',session:source,value:.37});assert.ok(Math.abs(state.volume-.37)<.02,'Per-app volume');await request({action:'volume',session:source,value:volume});}
  state=await request({action:'seek',session:'KYRO-missing-session',value:40000});assert.equal(state.available,false,'Stale commands cannot target another app');
  console.log('KYRO Windows bridge: artwork, pause/resume, seek, next/previous, cache and stale-session checks passed.');
  if(process.argv.includes('--hold')){
   const stopFile='output/hud-media-stop';fs.mkdirSync('output',{recursive:true});fs.rmSync(stopFile,{force:true});
   console.log('QA session is held for in-game inspection (up to 180 seconds).');
   await new Promise(resolve=>{const timer=setInterval(()=>{if(fs.existsSync(stopFile)){clearInterval(timer);clearTimeout(timeout);resolve();}},500);const timeout=setTimeout(()=>{clearInterval(timer);resolve();},180000);});
  }
 }finally{helper?.kill();await browser?.close();await new Promise(resolve=>server.close(resolve));}
})().catch(error=>{console.error(error);process.exitCode=1;});
