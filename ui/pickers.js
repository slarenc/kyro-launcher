/* KYRO controls: rendered inside the app, never by the operating system. */
window.KyroPickers = (() => {
 const $=(s,r=document)=>r.querySelector(s);
 const esc=s=>String(s??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 const chevron='<svg viewBox="0 0 20 20" aria-hidden="true"><path d="m5 7.5 5 5 5-5"/></svg>';
 const check='<svg viewBox="0 0 20 20" aria-hidden="true"><path d="m4 10 4 4 8-8"/></svg>';
 const searchIcon='<svg viewBox="0 0 20 20" aria-hidden="true"><circle cx="8.5" cy="8.5" r="5.5"/><path d="m13 13 4 4"/></svg>';
 const cores=['Vanilla','Fabric','Quilt','NeoForge'];
 const coreClass=l=>l.toLowerCase().replace(/[^a-z]/g,'');
 let config, menu, trigger, dialog, library, query='',downloaded=false,loading=false,opening=false,restoreFocus;
 const closed=new Set();
 function closeMenu(focus=false){if(menu){menu.remove();menu=null;}if(trigger){trigger.setAttribute('aria-expanded','false');if(focus&&trigger.isConnected)trigger.focus();trigger=null;}}
 function menuFor(select,button){
  if(trigger===button){closeMenu(true);return;}closeMenu();trigger=button;button.setAttribute('aria-expanded','true');
  menu=document.createElement('div');menu.className='lux-menu';menu.id='kyro-select-list';menu.setAttribute('role','listbox');menu.setAttribute('aria-label',select.getAttribute('aria-label')||'Выбор');
  menu.innerHTML=[...select.options].map((o,i)=>`<button type="button" role="option" aria-selected="${o.selected}" data-option="${i}" ${o.disabled?'disabled':''}><span class="${cores.includes(o.value)?'core-tag core-'+coreClass(o.value):'option-label'}">${esc(o.text)}</span>${o.selected?check:'<span class="option-check"></span>'}</button>`).join('');document.body.append(menu);
  const rect=button.getBoundingClientRect(),width=Math.min(Math.max(rect.width,224),innerWidth-24);menu.style.width=width+'px';menu.style.left=Math.min(Math.max(12,rect.left),innerWidth-width-12)+'px';
  const below=innerHeight-rect.bottom-16,above=rect.top-16;menu.style.maxHeight=Math.max(120,Math.min(360,Math.max(below,above)))+'px';if(below>=Math.min(220,above)){menu.style.top=(rect.bottom+8)+'px';}else{menu.style.bottom=(innerHeight-rect.top+8)+'px';}
  const options=[...menu.querySelectorAll('[role=option]:not(:disabled)')];
  options.forEach(b=>b.onclick=()=>{const value=select.options[Number(b.dataset.option)].value;closeMenu();select.value=value;button.querySelector('.lux-value').textContent=select.selectedOptions[0]?.text||value;select.dispatchEvent(new Event('change',{bubbles:true}));if(button.isConnected)button.focus();});
  menu.onkeydown=e=>{const i=options.indexOf(document.activeElement);if(['ArrowDown','ArrowUp','Home','End'].includes(e.key)){e.preventDefault();const next=e.key==='Home'?0:e.key==='End'?options.length-1:(i+(e.key==='ArrowDown'?1:-1)+options.length)%options.length;options[next]?.focus();}else if(e.key==='Tab'){closeMenu();}};
  const selected=options.find(b=>b.getAttribute('aria-selected')==='true')||options[0];selected?.focus({preventScroll:true});selected?.scrollIntoView({block:'nearest'});
 }
 function enhance(root=document){
  if(menu&&trigger&&!trigger.isConnected)closeMenu();
  for(const select of root.querySelectorAll('select:not([data-lux-ready])')){
   select.dataset.luxReady='true';select.classList.add('native-select-source');select.setAttribute('aria-hidden','true');select.tabIndex=-1;
   const button=document.createElement('button');button.type='button';button.className='lux-select '+[...select.classList].filter(c=>c!=='native-select-source').join(' ');button.disabled=select.disabled;
   button.id=select.id+'-trigger';button.setAttribute('aria-label',select.getAttribute('aria-label')||'Выбрать');button.setAttribute('aria-haspopup',['game-version','catalog-version'].includes(select.id)?'dialog':'listbox');button.setAttribute('aria-expanded','false');
   button.innerHTML=`<span class="lux-value">${esc(select.selectedOptions[0]?.text||select.value)}</span>${chevron}`;select.after(button);
   button.onclick=()=>['game-version','catalog-version'].includes(select.id)?openVersions(button):menuFor(select,button);
   button.onkeydown=e=>{if(['ArrowDown','ArrowUp'].includes(e.key)){e.preventDefault();button.click();}};
  }
 }
 function ensureDialog(){
  if(dialog)return;dialog=document.createElement('dialog');dialog.id='version-dialog';dialog.className='version-dialog';
  dialog.innerHTML=`<header class="version-dialog-heading"><div><span class="eyebrow">ТВОЙ MINECRAFT</span><h2>Версии и ядра</h2></div><button type="button" class="version-close" aria-label="Закрыть версии">×</button></header><div class="version-tools"><label class="version-search">${searchIcon}<input id="version-search" placeholder="Поиск версии или ядра" aria-label="Поиск версии или ядра" autocomplete="off" spellcheck="false"></label><button type="button" id="downloaded-filter" aria-pressed="false">↓ <span>Скачанные</span></button><button type="button" id="prepare-version" aria-label="Подготовить выбранную версию" title="Скачать и проверить выбранную версию">+</button></div><div id="version-list" class="version-list"></div><footer class="version-dialog-footer"><span id="version-count"></span><span id="version-active"></span></footer><p id="version-progress" role="status"></p>`;
  document.body.append(dialog);
  $('.version-close',dialog).onclick=()=>dialog.close();
  dialog.addEventListener('click',e=>{if(e.target===dialog){const r=dialog.getBoundingClientRect();if(e.clientX<r.left||e.clientX>r.right||e.clientY<r.top||e.clientY>r.bottom)dialog.close();}});
  dialog.addEventListener('close',()=>{const button=restoreFocus?.isConnected?restoreFocus:document.getElementById(restoreFocus?.id);button?.focus();});
  $('#version-search',dialog).oninput=e=>{query=e.target.value;drawList(true);};
  $('#downloaded-filter',dialog).onclick=()=>{downloaded=!downloaded;drawList(true);};
  $('#prepare-version',dialog).onclick=async()=>{
   if(config.current().busy)return;const button=$('#prepare-version',dialog);button.disabled=true;$('#version-progress',dialog).textContent='Подготовка выбранной версии…';
   try{const pending=config.prepare();drawList();const ok=await pending;if(ok===false)throw new Error(document.querySelector('#toast')?.textContent||'Подготовка не завершена');library=await config.load();$('#version-progress',dialog).textContent='Файлы проверены. Версия доступна в «Скачанных».';drawList();}catch(e){$('#version-progress',dialog).textContent=String(e?.message||e);}finally{drawList();button.disabled=false;}
  };
 }
 function fallback(){return {releases:[...new Set([config.current().version,...config.releases()])].map(id=>({id,date:null})),prepared:[]};}
 async function openVersions(button){
  if(!config||config.current().busy||opening)return;opening=true;closeMenu();ensureDialog();restoreFocus=button||document.activeElement;query='';downloaded=false;closed.clear();$('#version-search',dialog).value='';$('#version-progress',dialog).textContent='';
  if(!dialog.open)dialog.showModal();loading=true;library=library||fallback();drawList(true);$('#version-search',dialog).focus();
  try{library=await config.load();}catch(e){$('#version-progress',dialog).textContent='Список не обновлён. Показаны сохранённые версии.';}finally{loading=false;opening=false;if(dialog.open)drawList(true);}
 }
 function drawList(resetScroll=false){
  const current=config.current(),term=query.trim().toLowerCase();
  const ready=new Set((library?.prepared||[]).map(v=>v.version+'/'+v.loader));
  const releases=[...(library?.releases||[])];if(!releases.some(v=>v.id===current.version))releases.unshift({id:current.version,date:null});
  if(term)releases.sort((a,b)=>Number(b.id.toLowerCase()===term)-Number(a.id.toLowerCase()===term));
  let rows=0,groups=0;
  const html=releases.map((release,index)=>{
   const allRows=cores.filter(loader=>(!downloaded||ready.has(release.id+'/'+loader))&&(!term||(release.id+' '+loader).toLowerCase().includes(term)));
   if(!allRows.length)return '';rows+=allRows.length;groups++;
   const expanded=term||downloaded?true:(release.id===current.version||index<3)?!closed.has(release.id):false;
   const date=release.date?new Date(release.date).toLocaleDateString('ru-RU',{day:'numeric',month:'short'}).replace('.',''):'';
   return `<details class="version-group" data-group="${esc(release.id)}" ${expanded?'open':''}><summary>${chevron}<strong>${esc(release.id)}</strong><small>${allRows.length} ${allRows.length===1?'ядро':'ядра'}</small></summary><div>${allRows.map(loader=>{
    const active=current.version===release.id&&current.loader===loader,installed=ready.has(release.id+'/'+loader);
    return `<button type="button" class="version-row ${active?'selected':''}" data-version="${esc(release.id)}" data-core="${loader}" aria-label="Minecraft ${esc(release.id)} ${loader}" aria-pressed="${active}" ${current.busy?'disabled':''}><span class="version-row-id">${esc(release.id)}</span><span class="core-tag core-${coreClass(loader)}">${loader}</span><span class="version-row-meta">${installed?'<span class="downloaded-dot" title="Файлы подготовлены">↓</span>':''}<time>${esc(date)}</time>${active?check:'<span class="option-check"></span>'}</span></button>`;
   }).join('')}</div></details>`;
  }).join('');
  $('#version-list',dialog).innerHTML=html||`<div class="version-empty"><span>${downloaded?'Пока нет скачанных версий':'Версия не найдена'}</span><p>${downloaded?'Выбери версию и нажми +, чтобы подготовить файлы.':'Попробуй другой номер версии или название ядра.'}</p></div>`;
  if(resetScroll)$('#version-list',dialog).scrollTop=0;
  $('#downloaded-filter',dialog).setAttribute('aria-pressed',String(downloaded));
  $('#version-count',dialog).textContent=loading?'Обновляем список…':`Версий: ${groups} · Вариантов: ${rows}`;
  $('#version-active',dialog).textContent=`${current.version} / ${current.loader}`;
  $('#prepare-version',dialog).disabled=current.busy;
  for(const group of dialog.querySelectorAll('details'))group.ontoggle=()=>{if(group.open)closed.delete(group.dataset.group);else closed.add(group.dataset.group);};
  for(const button of dialog.querySelectorAll('[data-version]'))button.onclick=async()=>{
   if(config.current().busy)return;button.disabled=true;
   try{await config.select(button.dataset.version,button.dataset.core);dialog.close();}catch(e){$('#version-progress',dialog).textContent=String(e?.message||e);button.disabled=false;}
  };
 }
 document.addEventListener('pointerdown',e=>{if(menu&&!menu.contains(e.target)&&!trigger?.contains(e.target))closeMenu();});
 document.addEventListener('keydown',e=>{if(e.key==='Escape'&&menu){e.preventDefault();e.stopPropagation();closeMenu(true);}});
 window.addEventListener('resize',()=>closeMenu());document.addEventListener('scroll',e=>{if(menu&&!menu.contains(e.target))closeMenu();},true);
 return {configure(value){config=value;},enhance,openVersions,progress(message){if(dialog?.open)$('#version-progress',dialog).textContent=message;}};
})();
