import test from 'node:test';import assert from 'node:assert/strict';
import worker,{handle} from '../worker/index.js';import {database} from './database.mjs';
const A='11111111111111111111111111111111',B='22222222222222222222222222222222',C='33333333333333333333333333333333';
function setup(){let now=1700000000000;const env={DB:database()};return {env,advance(ms){now+=ms;},async post(data,path='/v1/sync'){const response=await handle(new Request('https://kyro.test'+path,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(data)}),env,()=>now);return {status:response.status,...await response.json()};}};}
test('two clients see each other, unknown players stay unmarked, leases survive separate requests',async()=>{
 const api=setup(),a=await api.post({uuid:A,name:'WannaForget',peers:[B,C]}),b=await api.post({uuid:B,name:'SecondKyro',peers:[A,C]});assert.equal(a.status,200);assert.equal(b.status,200);assert.deepEqual(a.players,[]);assert.equal(b.players[0].uuid,A);
 const updated=await api.post({uuid:A,name:'WannaForget',lease:a.lease,peers:[B,C]});assert.deepEqual(updated.players.map(p=>p.uuid),[B]);assert.equal(updated.players[0].name,'SecondKyro');
 assert.equal(api.env.DB.raw.prepare('SELECT token_hash FROM sessions LIMIT 1').get().token_hash.length,64);assert.notEqual(api.env.DB.raw.prepare('SELECT token_hash FROM sessions LIMIT 1').get().token_hash,a.lease);
});
test('closing a client and expiry remove badges; another UUID cannot renew its lease',async()=>{
 const api=setup(),a=await api.post({uuid:A,name:'First',peers:[]});assert.equal((await api.post({uuid:B,name:'Second',lease:a.lease,peers:[]})).status,401);
 await api.post({lease:a.lease},'/v1/leave');assert.deepEqual((await api.post({uuid:B,name:'Second',peers:[A]})).players,[]);
 const next=await api.post({uuid:A,name:'First',peers:[]});api.advance(76000);assert.equal((await api.post({uuid:A,name:'First',lease:next.lease,peers:[]})).status,401);assert.deepEqual((await api.post({uuid:B,name:'Second',peers:[A]})).players,[]);
});
test('malformed and oversized requests are bounded; only queried IDs are returned',async()=>{
 const api=setup();assert.equal((await api.post({uuid:'bad',name:'First',peers:[]})).status,400);assert.equal((await api.post({uuid:A,name:'First',peers:Array(257).fill(B)})).status,400);assert.equal((await api.post({uuid:A,name:'First',peers:[],padding:'x'.repeat(25000)})).error,'body_too_large');
 await api.post({uuid:A,name:'First',peers:[]});assert.deepEqual((await api.post({uuid:B,name:'Second',peers:[C]})).players,[]);
});
test('worker fetch accepts the Cloudflare execution context without treating it as a clock',async()=>{const response=await worker.fetch(new Request('https://kyro.test/health'),{DB:database()},{waitUntil(){}});assert.equal(response.status,200);});
