import http from 'node:http';import path from 'node:path';import {spawn} from 'node:child_process';import {fileURLToPath} from 'node:url';import {readdirSync,existsSync} from 'node:fs';
import {handle} from '../worker/index.js';import {database} from './database.mjs';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..'),env={DB:database()};
const server=http.createServer(async(req,res)=>{try{const chunks=[];for await(const chunk of req)chunks.push(chunk);let url=req.url;if(url.startsWith('/slow/')){await new Promise(resolve=>setTimeout(resolve,3000));url=url.slice(5);}
 const response=await handle(new Request('http://127.0.0.1'+url,{method:req.method,headers:req.headers,body:Buffer.concat(chunks)}),env);res.writeHead(response.status,Object.fromEntries(response.headers));res.end(Buffer.from(await response.arrayBuffer()));}catch{res.writeHead(500);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
try{
 const wrapper=path.join(process.env.USERPROFILE,'.gradle/wrapper/dists/gradle-9.2.1-bin'),gradle=readdirSync(wrapper).map(name=>path.join(wrapper,name,'gradle-9.2.1/bin/gradle.bat')).find(existsSync);if(!gradle)throw Error('Gradle wrapper missing');
 const child=spawn(gradle,['-p','components/hud','presenceIntegration','-PpresenceUrl=http://127.0.0.1:'+server.address().port,'--console=plain'],{cwd:root,stdio:'inherit',windowsHide:true,shell:true});
 const code=await new Promise((resolve,reject)=>{child.once('error',reject);child.once('exit',resolve);});if(code!==0)process.exitCode=code||1;
}finally{server.closeAllConnections();await new Promise(resolve=>server.close(resolve));env.DB.raw.close();}
