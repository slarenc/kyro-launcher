const TTL=75,MAX_PEERS=256,MAX_BODY=24000;
const json=(value,status=200)=>new Response(JSON.stringify(value),{status,headers:{'Content-Type':'application/json; charset=utf-8','Cache-Control':'no-store','Access-Control-Allow-Origin':'*'}});
const uuid=value=>{const text=String(value||'').replaceAll('-','').toLowerCase();if(!/^[0-9a-f]{32}$/.test(text))throw Error('invalid_uuid');return text;};
const hex=bytes=>Array.from(bytes,byte=>byte.toString(16).padStart(2,'0')).join('');
const hash=async value=>hex(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(value))));
async function body(request){
 if(!request.headers.get('Content-Type')?.toLowerCase().startsWith('application/json'))throw Error('json_required');
 const reader=request.body?.getReader();if(!reader)throw Error('empty_body');let size=0,chunks=[];
 try{while(true){const part=await reader.read();if(part.done)break;size+=part.value.length;if(size>MAX_BODY){await reader.cancel();throw Error('body_too_large');}chunks.push(part.value);}}finally{reader.releaseLock();}
 const all=new Uint8Array(size);let offset=0;for(const chunk of chunks){all.set(chunk,offset);offset+=chunk.length;}return JSON.parse(new TextDecoder().decode(all));
}
export async function handle(request,env,clock=()=>Date.now()){
 const url=new URL(request.url),now=Math.floor(clock()/1000);
 if(request.method==='OPTIONS')return new Response(null,{status:204,headers:{'Access-Control-Allow-Origin':'*','Access-Control-Allow-Methods':'POST, GET, OPTIONS','Access-Control-Allow-Headers':'Content-Type'}});
 if(url.pathname==='/')return new Response('KYRO Presence\nShort-lived Minecraft client badge status. No public player directory.\n',{headers:{'Content-Type':'text/plain; charset=utf-8'}});
 if(!env.DB)return json({error:'storage_unavailable'},503);
 if(url.pathname==='/health'){try{await env.DB.prepare('SELECT COUNT(*) AS count FROM sessions').first();return json({ok:true,protocol:1});}catch{return json({ok:false},503);}}
 if(!['/v1/sync','/v1/leave'].includes(url.pathname))return json({error:'not_found'},404);
 if(request.method!=='POST')return json({error:'method_not_allowed'},405);
 try{
  const ip=request.headers.get('CF-Connecting-IP')||'local',bucket=await hash(`${ip}/${Math.floor(now/60)}`);
  await env.DB.prepare('INSERT INTO rate_limits (bucket,count,expires) VALUES (?,1,?) ON CONFLICT(bucket) DO UPDATE SET count=count+1').bind(bucket,now+120).run();
  const limit=await env.DB.prepare('SELECT count FROM rate_limits WHERE bucket=?').bind(bucket).first();if(limit.count>120)return json({error:'rate_limited'},429);
  const input=await body(request);
  if(!input||Array.isArray(input)||typeof input!=='object')throw Error('invalid_body');
  if(url.pathname==='/v1/leave'){
   if(!/^[0-9a-f]{64}$/.test(input.lease||''))return json({error:'invalid_lease'},400);
   await env.DB.prepare('DELETE FROM sessions WHERE token_hash=?').bind(await hash(input.lease)).run();return json({ok:true});
  }
  const id=uuid(input.uuid),name=String(input.name||'');if(!/^[A-Za-z0-9_]{3,16}$/.test(name))throw Error('invalid_name');
  if(!Array.isArray(input.peers)||input.peers.length>MAX_PEERS)throw Error('invalid_peers');const peers=[...new Set(input.peers.map(uuid))];
  let lease=input.lease;
  if(lease){
   if(!/^[0-9a-f]{64}$/.test(lease))throw Error('invalid_lease');
   const session=await env.DB.prepare('SELECT uuid,name FROM sessions WHERE token_hash=? AND expires>?').bind(await hash(lease),now).first();
   if(!session||session.uuid!==id||session.name!==name)return json({error:'expired_lease'},401);
   await env.DB.prepare('UPDATE sessions SET expires=? WHERE token_hash=?').bind(now+TTL,await hash(lease)).run();
  }else{
   const count=await env.DB.prepare('SELECT COUNT(*) AS count FROM sessions WHERE uuid=? AND expires>?').bind(id,now).first();if(count.count>=4)return json({error:'too_many_sessions'},429);
   lease=hex(crypto.getRandomValues(new Uint8Array(32)));await env.DB.prepare('INSERT INTO sessions(token_hash,uuid,name,expires) VALUES (?,?,?,?)').bind(await hash(lease),id,name,now+TTL).run();
  }
  const players=[];
  for(let at=0;at<peers.length;at+=80){const group=peers.slice(at,at+80),marks=group.map(()=>'?').join(',');
   const rows=await env.DB.prepare(`SELECT uuid,name,MAX(expires) AS expires FROM sessions WHERE expires>? AND uuid IN (${marks}) GROUP BY uuid,name`).bind(now,...group).all();
   for(const row of rows.results||[])players.push({uuid:row.uuid,name:row.name,ttl:Math.max(1,Math.min(TTL,row.expires-now))});
  }
  await env.DB.batch([
   env.DB.prepare('DELETE FROM sessions WHERE token_hash IN (SELECT token_hash FROM sessions WHERE expires<=? LIMIT 512)').bind(now),
   env.DB.prepare('DELETE FROM rate_limits WHERE bucket IN (SELECT bucket FROM rate_limits WHERE expires<=? LIMIT 512)').bind(now)
  ]);
  return json({protocol:1,lease,ttl:TTL,poll:20,players});
 }catch(error){const expected=['invalid_uuid','invalid_name','invalid_peers','invalid_lease','invalid_body','json_required','empty_body','body_too_large'];const known=expected.includes(error.message)||error instanceof SyntaxError;return json({error:known?(expected.includes(error.message)?error.message:'invalid_request'):'server_error'},known?400:500);}
}
export default {fetch(request,env){return handle(request,env);}};
