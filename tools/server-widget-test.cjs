const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
(async()=>{
 let now=0,timer,calls=0,rendered=0,response={online:true,players:68,source:'mcsrvstat.us',icon:'data:image/png;base64,AAAA'};
 const sandbox={window:{},document:{hidden:false,querySelector:()=>({}),getElementById:()=>null,querySelectorAll:()=>[]},Date:{now:()=>now},setInterval:fn=>{timer=fn;return 1;}};
 vm.runInNewContext(fs.readFileSync('ui/servers.js','utf8'),sandbox);
 const servers=sandbox.window.KyroServers;
 servers.configure({native:true,state:()=>({data:{servers:[{id:'virtus',name:'VirtusMine',address:'virtusmine.ru'}]}}),invoke:async()=>{calls++;return response;},render:()=>rendered++});
 assert.match(servers.card(),/Проверяем/);servers.bind();await new Promise(setImmediate);
 assert.match(servers.card(),/68 онлайн/);assert.match(servers.card(),/mcsrvstat.us/);assert.match(servers.card(),/data:image/);assert.equal(calls,1);
 now=60001;response={online:false,error:'Timeout'};timer();await new Promise(setImmediate);assert.match(servers.card(),/Нет ответа/);assert.match(servers.card(),/Timeout/);
 now=75002;response={online:true,players:70};timer();await new Promise(setImmediate);assert.match(servers.card(),/70 онлайн/);assert.equal(calls,3);assert.equal(rendered,3);
 sandbox.document.hidden=true;now=180000;timer();assert.equal(calls,3);
 console.log('PASS: server icon, fallback source, online count, automatic retry and hidden-window pause');
})().catch(error=>{console.error(error);process.exitCode=1;});
