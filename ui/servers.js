window.KyroServers=(()=>{
 let ctx,editing=false,saving=false;const statuses=new Map();
 const esc=v=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 const list=()=>ctx.state().data.servers||[];

 function card(){
  const servers=list();
  return `<aside class="server-widget" aria-label="Избранные серверы"><div class="server-widget-heading"><span>СЕРВЕРЫ <small>${servers.length}/3</small></span><button id="server-add" ${servers.length>=3||editing?'disabled':''} aria-label="Добавить сервер">＋</button></div>${editing?`<form id="favorite-server-form"><label>Название<input id="server-name" maxlength="100" required placeholder="Мой сервер"></label><label>Адрес<input id="server-address" maxlength="255" required placeholder="mc.example.org:25565"></label><div class="server-form-actions"><button type="button" id="server-cancel">Отмена</button><button class="primary">Добавить</button></div></form>`:''}<div class="server-list">${servers.map(server=>{
   const status=statuses.get(server.id)?.value;
   const source=status?.icon;
   const image=source&&/^data:image\/(png|jpeg|webp);base64,[A-Za-z0-9+/=]+$/.test(source)?`<img src="${source}" alt="Иконка ${esc(server.name)}">`:`<span>${esc(server.name.slice(0,1).toUpperCase())}</span>`;
   return `<div class="server-line"><div class="server-avatar" title="Иконка сервера">${image}</div><button class="server-connect" data-server-play="${esc(server.id)}" title="Играть на ${esc(server.name)}" ${ctx.state().busy?'disabled':''}><strong>${esc(server.address.replace(/:25565$/,''))}</strong><span class="server-presence ${status?.online?'online':''}"><i></i>${!status?'Проверяем…':status.online?`${esc(status.players??'—')} онлайн`:'Нет ответа'}</span></button><button class="server-line-play" data-server-play="${esc(server.id)}" aria-label="Играть на ${esc(server.name)}" ${ctx.state().busy?'disabled':''}>▶</button><button class="server-remove" data-server-remove="${esc(server.id)}" title="Удалить сервер" aria-label="Удалить ${esc(server.name)}">×</button></div>`;
  }).join('')}</div>${!servers.length&&!editing?'<button class="server-empty" id="server-empty-add">＋ Добавить любимый сервер</button>':''}</aside>`;
 }
 function refresh(force=false){for(const server of list()){
  const previous=statuses.get(server.id);if(previous?.pending||(!force&&previous&&Date.now()-previous.time<60000))continue;
  statuses.set(server.id,{...previous,time:Date.now(),pending:true});
  ctx.invoke('server_status',{id:server.id}).then(value=>statuses.set(server.id,{value,time:Date.now()})).catch(()=>statuses.set(server.id,{value:{online:false},time:Date.now()})).finally(()=>{if(document.querySelector('.server-widget')&&!editing)ctx.render();});
 }}
 function bind(){if(!document.querySelector('.server-widget'))return;const on=(id,fn)=>{const el=document.getElementById(id);if(el)el.onclick=fn;};
  on('server-add',()=>{editing=true;ctx.render();document.getElementById('server-name')?.focus();});
  on('server-cancel',()=>{editing=false;ctx.render();});

  on('server-empty-add',()=>{editing=true;ctx.render();document.getElementById('server-name')?.focus();});
  document.querySelectorAll('[data-server-remove]').forEach(b=>b.onclick=async()=>{try{await ctx.invoke('remove_server',{id:b.dataset.serverRemove});await ctx.sync();ctx.render();}catch(e){ctx.error(e);}});
  document.querySelectorAll('[data-server-play]').forEach(b=>b.onclick=()=>{const server=list().find(s=>s.id===b.dataset.serverPlay);ctx.run(`Подключение к ${server.name}…`,()=>ctx.invoke('launch_server',{id:server.id}));});
  const form=document.getElementById('favorite-server-form');if(form)form.onsubmit=async e=>{e.preventDefault();if(saving)return;const name=document.getElementById('server-name').value.trim(),address=document.getElementById('server-address').value.trim();saving=true;form.querySelectorAll('input,button').forEach(el=>el.disabled=true);try{await ctx.invoke('save_server',{name,address});await ctx.sync();editing=false;ctx.render();}catch(err){ctx.error(err);}finally{saving=false;form.querySelectorAll('input,button').forEach(el=>el.disabled=false);}};
  if(ctx.native)refresh();
 }
 return {configure:c=>ctx=c,card,bind};
})();
