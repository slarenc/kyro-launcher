window.KyroFeatures=(()=>{
 const $=(s)=>document.querySelector(s),$$=(s)=>[...document.querySelectorAll(s)];
 const esc=s=>String(s??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 let config,mods=null,loading=false,modsError='',update=null,checking=false,updateError='',scanSeq=0;
 const key=()=>{const s=config.state().data.settings;return [s.version,s.loader,s.instance_id||''].join('/');};let modsKey='';
 const updateCards=new Map(),pendingCards=new Set(),cardEpoch=new Map();const CARD_TTL=5*60*1000;
 function rememberUpdates(profile,data){updateCards.set(profile,{data,time:Date.now(),count:data.files.filter(m=>m.update).length,error:data.warning||''});while(updateCards.size>20)updateCards.delete(updateCards.keys().next().value);}
 function paintCard(){if(!config)return;const card=$('#mod-updates-card');if(!card)return;const profile=key(),entry=updateCards.get(profile),busy=config.state().busy;
  const count=entry?.count||0,last=count%100,word=last>=11&&last<=14?'обновлений':count%10===1?'обновление':count%10>=2&&count%10<=4?'обновления':'обновлений';
  const text=entry?.error?'Не удалось проверить':entry?entry.data.files.length?(count?`Доступно ${count} ${word}`:'Обновлений нет'):'Моды не установлены':busy?'Проверим после игры':'Проверяем…';
  const boost=$('#fps-boost');if(boost)boost.classList.toggle('has-boost',(entry?.data?.fps_enabled||0)>0);
  const label=card.querySelector('strong');if(label.textContent!==text)label.textContent=text;card.setAttribute('aria-label',`Обновления модов: ${text}. Открыть менеджер`);card.classList.toggle('has-updates',!!count&&!entry?.error);card.title=entry?.error||'Обновления модов текущего профиля — открыть менеджер';
  card.onclick=()=>{const saved=updateCards.get(key());if(saved?.data){mods=saved.data;modsKey=key();}config.openMods();scan(true);};
  if(!busy&&!pendingCards.has(profile)&&(!entry||Date.now()-entry.time>CARD_TTL)){pendingCards.add(profile);const epoch=cardEpoch.get(profile)||0;Promise.resolve().then(()=>config.invoke('scan_mods',{refresh:true})).then(data=>{if(epoch===(cardEpoch.get(profile)||0))rememberUpdates(profile,data);}).catch(e=>{if(epoch===(cardEpoch.get(profile)||0))updateCards.set(profile,{time:Date.now(),count:0,error:String(e),data:{files:[]}});}).finally(()=>{pendingCards.delete(profile);paintCard();});}
 }
 function invalidateUpdates(){const profile=key();cardEpoch.set(profile,(cardEpoch.get(profile)||0)+1);updateCards.delete(profile);paintCard();}
 async function scan(refresh=false){const seq=++scanSeq,profile=key();if(refresh)cardEpoch.set(profile,(cardEpoch.get(profile)||0)+1);loading=true;modsError='';config.render();try{const data=await config.invoke('scan_mods',{refresh});if(seq===scanSeq&&profile===key()){mods=data;modsKey=profile;if(refresh)rememberUpdates(profile,data);}}catch(e){if(seq===scanSeq){modsError=String(e);if(refresh)updateCards.set(profile,{time:Date.now(),count:0,error:String(e),data:{files:[]}});}}finally{if(seq===scanSeq){loading=false;config.render();paintCard();}}}
 function manager(){if(modsKey!==key())mods=null;
  return `<div class="manager-toolbar"><div><h2>Мои моды</h2><p class="help">Моды текущего профиля. Перед изменением закрой Minecraft.</p></div><button id="refresh-mods" ${loading?'disabled':''}>${loading?'Проверяем…':'Проверить обновления'}</button></div>${modsError?`<p class="help" role="alert">${esc(modsError)}</p>`:''}${mods?.warning?`<p class="help">${esc(mods.warning)}</p>`:''}${!mods?`<p class="help">${loading?'Чтение модов…':'Открой менеджер, чтобы прочитать файлы.'}</p>`:mods.files.length?mods.files.map((m,i)=>`<div class="managed-mod ${m.enabled?'':'mod-disabled'}"><div class="mod-info"><strong>${esc(m.name)}</strong><p>${esc(m.filename)}</p><small>${esc(m.version_number||'Локальный JAR')} · ${(m.size/1048576).toFixed(1)} МБ${m.dependencies?.length?` · Зависимости: ${m.dependencies.map(d=>esc(d.name||d.id)).join(', ')}`:''}</small></div>${m.update?`<button data-update-mod="${i}" ${config.state().busy?'disabled':''}>Обновить · ${esc(m.update.version)}</button>`:''}<label class="mod-switch"><input type="checkbox" data-toggle-mod="${i}" aria-label="Включить ${esc(m.name)}" ${m.enabled?'checked':''} ${config.state().busy?'disabled':''}><span>${m.enabled?'Включён':'Выключен'}</span></label></div>`).join(''):'<div class="empty"><strong>Модов пока нет</strong>Установи моды из каталога или открой профиль сборки.</div>'}`;
 }
 async function checkUpdate(silent=false){if(checking)return;checking=true;updateError='';if(!silent)config.render();try{update=await config.invoke('check_launcher_update');if(update.available)await config.changed();}catch(e){updateError=String(e);}finally{checking=false;if(!silent)config.render();}}
 async function installUpdate(){
  return config.run('Обновление KYRO…',async()=>{
   const info=await config.invoke('check_launcher_update');update=info;
   if(!info.available)throw new Error('Установлена актуальная версия KYRO.');
   if(!info.installable)throw new Error('Эту версию пока нельзя установить автоматически. Попробуй позже.');
   await config.invoke('install_launcher_update');
  },false,true);
 }
 function settings(){const state=config.state(),s=state.data.settings,pack=(state.data.modpacks||[]).find(p=>p.id===s.instance_id);return `<div class="setting-section"><div><h3>ОФОРМЛЕНИЕ ЛАУНЧЕРА</h3><p>Выбранный фон отображается во всём окне и на всех вкладках.</p></div><div class="setting-controls"><div class="segments"><button data-home-layout="orbit" class="${s.home_layout!=='card'?'selected':''}">Орбита</button><button data-home-layout="card" class="${s.home_layout==='card'?'selected':''}">Карточка</button></div><div class="image-actions"><button id="choose-background">Загрузить фон</button><button id="clear-background" ${s.home_background?'':'disabled'}>Убрать фон</button><input type="file" id="background-file" accept="image/png,image/jpeg,image/webp" hidden></div>${pack?`<div class="image-actions"><button id="choose-cover">Обложка: ${esc(pack.name)}</button><button id="clear-cover" ${pack.cover_image?'':'disabled'}>Убрать обложку</button><input type="file" id="cover-file" accept="image/png,image/jpeg,image/webp" hidden></div>`:''}<p>PNG, JPEG или WebP до 4 МБ. Файлы хранятся локально.</p></div></div><div class="setting-section"><div><h3>ОБНОВЛЕНИЯ KYRO</h3><p>Текущая версия ${CURRENT_LAUNCHER_VERSION}. Настройки и игровые профили сохраняются.</p></div><div class="setting-controls"><button id="check-kyro-update" ${checking?'disabled':''}>${checking?'Проверяем GitHub…':'Проверить обновления'}</button>${updateError?`<p role="alert">${esc(updateError)}</p>`:update?`<p>${update.available?`Доступна ${esc(update.version)}`:'Установлена актуальная версия'}</p>${update.available?`<p class="update-notes">${esc(update.notes||'')}</p>`:''}${update.installable?'<button id="install-kyro-update" class="primary">Обновить и перезапустить</button>':''}${update.available&&!update.installable?'<p>Автоматическое обновление пока недоступно.</p>':''}`:''}</div></div>`;}
 function javaSettings(){const s=config.state().data.settings;return `<div class="setting-section"><div><h3>АВТОМАТИЧЕСКАЯ JAVA</h3><p>Подходящая Temurin загружается в данные KYRO, без системной установки.</p></div><div class="setting-controls"><label class="setting-row"><span>Подбирать Java для Minecraft</span><input type="checkbox" id="auto-java" ${s.auto_java!==false?'checked':''}></label><button id="install-java" ${config.state().busy?'disabled':''}>Скачать Java для выбранной версии</button></div></div>`;}
 async function change(key,value){config.state().data.settings[key]=value;if(await config.persist())await config.changed();}
 function imageInput(input,callback){input.onchange=async()=>{const file=input.files[0];if(!file)return;if(file.size>4*1024*1024||!['image/png','image/jpeg','image/webp'].includes(file.type)){config.error('Выбери PNG, JPEG или WebP до 4 МБ.');return;}try{const image=await new Promise((resolve,reject)=>{const reader=new FileReader();reader.onload=()=>resolve(reader.result);reader.onerror=reject;reader.readAsDataURL(file);});await callback(image);}catch(e){config.error(e);}};}
 function bind(){
  if($('#refresh-mods'))$('#refresh-mods').onclick=()=>scan(true);
  $$('[data-toggle-mod]').forEach(b=>b.onchange=async()=>{const m=mods.files[Number(b.dataset.toggleMod)];const ok=await config.run('Переключение мода…',()=>config.invoke('toggle_mod',{filename:m.filename,enabled:b.checked}));if(ok)await scan(false);});
  $$('[data-update-mod]').forEach(b=>b.onclick=async()=>{const m=mods.files[Number(b.dataset.updateMod)];const ok=await config.run('Обновление мода и зависимостей…',()=>config.invoke('update_mod',{filename:m.filename}),false,true);if(ok)await scan(true);});
  $$('[data-home-layout]').forEach(b=>b.onclick=()=>change('home_layout',b.dataset.homeLayout));
  if($('#choose-background'))$('#choose-background').onclick=()=>$('#background-file').click();
  if($('#background-file'))imageInput($('#background-file'),image=>change('home_background',image));
  if($('#clear-background'))$('#clear-background').onclick=()=>change('home_background','');
  const cover=async image=>{await config.invoke('set_pack_cover',{id:config.state().data.settings.instance_id,image});await config.changed();};
  if($('#choose-cover'))$('#choose-cover').onclick=()=>$('#cover-file').click();
  if($('#cover-file'))imageInput($('#cover-file'),cover);
  if($('#clear-cover'))$('#clear-cover').onclick=()=>cover('');
  if($('#auto-java'))$('#auto-java').onchange=e=>change('auto_java',e.target.checked);
  if($('#install-java'))$('#install-java').onclick=()=>config.run('Подготовка автоматической Java…',()=>config.invoke('install_java'));
  if($('#check-kyro-update'))$('#check-kyro-update').onclick=()=>checkUpdate(false);
  if($('#install-kyro-update'))$('#install-kyro-update').onclick=installUpdate;
  $$('[data-launcher-update]').forEach(button=>button.onclick=installUpdate);
 }
 return {configure(c){config=c;window.addEventListener('focus',paintCard);setInterval(()=>{if(!document.hidden)paintCard();},60000);},manager,settings,javaSettings,bind,scan,checkUpdate,paintCard,invalidateUpdates};
})();
