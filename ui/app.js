'use strict';
const CURRENT_LAUNCHER_VERSION = '0.8.10';
const $ = (q, root = document) => root.querySelector(q);
const $$ = (q, root = document) => [...root.querySelectorAll(q)];
const esc = s => String(s ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const paths = {
 trash:'M3 6h18M9 6V3h6v3M5 6l1 15h12l1-15M10 10v7M14 10v7',
 servers:'M4 3h16v6H4Zm0 12h16v6H4Zm3-9h.01M7 18h.01M12 9v6',screenshots:'M3 4h18v16H3Zm0 12 5-5 5 5 3-3 5 5M16 8h.01',diagnostics:'M3 12h4l3-7 4 14 3-7h4',
 play:'M7 4 21 12 7 20Z', catalog:'m12 3 10 5-10 5L2 8Zm-10 9 10 5 10-5M2 16l10 5 10-5',
 friends:'M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M16 3a4 4 0 0 1 0 8m6 10v-2a4 4 0 0 0-3-3.87M13 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0',
 activity:'M3 13h4l3-9 4 16 3-7h4', bell:'M18 8a6 6 0 0 0-12 0c0 7-3 7-3 9h18c0-2-3-2-3-9M10 21h4',
 settings:'m10 2-.6 3-2 .9L4.7 5 2 9l2.3 2v2L2 15l2.7 4 2.7-.9 2 .9.6 3h4l.6-3 2-.9 2.7.9 2.7-4-2.3-2v-2L22 9l-2.7-4-2.7.9-2-.9L14 2ZM16 12a4 4 0 1 1-8 0 4 4 0 0 1 8 0',
 search:'M21 21l-5-5M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0', download:'M12 3v12m-5-5 5 5 5-5M4 16v5h16v-5',
 clock:'M12 6v6l4 2M22 12A10 10 0 1 1 2 12a10 10 0 0 1 20 0', folder:'M3 5h6l2 3h10v13H3Z',check:'m4 12 5 5L20 6',
 arrow:'M5 12h14m-5-5 5 5-5 5', box:'m12 2 10 5v10l-10 5-10-5V7Zm0 10L2 7m10 5L22 7m-10 5v10',
 refresh:'M20 7v5h-5M4 17v-5h5M6 6a8 8 0 0 1 13 1M5 17a8 8 0 0 0 13 1', logout:'M10 3H3v18h7m4-5 5-4-5-4m-5 4h10'
};
const icon = name => `<svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="${paths[name] || paths.box}"/></svg>`;
const labels = {play:'Играть',catalog:'Каталог',screenshots:'Скриншоты',diagnostics:'Диагностика',activity:'Активность',notifications:'Уведомления',settings:'Настройки'};
const native = !!window.__TAURI__;
const defaults = {auto_java:true,home_layout:'orbit',home_background:'',username:'Nova',version:'1.21.11',loader:'Fabric',memory_gb:6,java_path:'java',jvm_args:'-XX:+UseG1GC -XX:MaxGCPauseMillis=50',download_threads:8,launch_behavior:'hide',language:'ru',theme:'black',accent:'#a185cf',mod_notifications:true,system_notifications:true,curseforge_api_key:'',microsoft_client_id:''};
let state = {data:{settings:{...defaults},sessions:[],notices:[],installed:[]},account:null,busy:false};
let page = 'play', settingsTab = 'general', catalogKind = 'mod', catalogOffset = 0, catalogQuery = '', catalogData = null, catalogError = '', catalogLoading = false, catalogSeq = 0;
let catalogTimer;
let noticeFilter='all', year=new Date().getFullYear(), releaseVersions=['1.21.11'], toastTimer, saveTimer, loginTimer, refreshTimer;
const invoke = (command,args={}) => native ? window.__TAURI__.core.invoke(command,args) : Promise.reject(new Error('Откройте приложение KYRO. В браузере доступен только предпросмотр интерфейса.'));
function toast(message){ $('#toast').textContent=String(message);$('#toast').hidden=false;clearTimeout(toastTimer);toastTimer=setTimeout(()=>$('#toast').hidden=true,7000); }
function failure(e){toast(e?.message || String(e));}
function duration(seconds){const m=Math.floor(seconds/60);return m>=60?`${Math.floor(m/60)} ч${m%60?` ${m%60} мин`:''}`:`${m} мин`;}
function dateKey(date){return `${date.getFullYear()}-${String(date.getMonth()+1).padStart(2,'0')}-${String(date.getDate()).padStart(2,'0')}`;}
function activityStreak(sessions,now=new Date()){
 const today=new Date(now);today.setHours(0,0,0,0);
 const days=new Set();
 for(const session of sessions){
  const start=new Date(session.started);if(!Number.isFinite(start.getTime())||start>now)continue;
  const seconds=Number(session.seconds);
  const end=new Date(Math.min(now.getTime(),start.getTime()+(Number.isFinite(seconds)?Math.max(0,seconds)*1000:0)));
  const day=new Date(start);day.setHours(0,0,0,0);
  // A session ending exactly at midnight has no play time in the next day.
  const last=new Date(Math.max(start.getTime(),end.getTime()-1));last.setHours(0,0,0,0);
  for(;day<=last;day.setDate(day.getDate()+1))days.add(dateKey(day));
 }
 const cursor=new Date(today);if(!days.has(dateKey(cursor)))cursor.setDate(cursor.getDate()-1);
 let streak=0;while(days.has(dateKey(cursor))){streak++;cursor.setDate(cursor.getDate()-1);}
 return streak;
}
const s=()=>state.data.settings;
async function sync(){if(native)state=await invoke('snapshot');updateChrome();}
function updateChrome(){
 window.KyroI18n.setLanguage(s().language);
 updateLauncherBackground();
 window.KyroFeatures.paintCard();
 document.documentElement.style.setProperty('--accent',s().accent);
 document.documentElement.style.setProperty('--accent-wash',`${s().accent}1c`);
 const rgb=s().accent.slice(1).match(/../g).map(x=>parseInt(x,16));const luminance=rgb.map(c=>{c/=255;return c<=.04045?c/12.92:((c+.055)/1.055)**2.4;}).reduce((n,c,i)=>n+c*[.2126,.7152,.0722][i],0);document.documentElement.style.setProperty('--accent-ink',luminance>.179?'#100d18':'#ffffff');
 const themes={black:['#08070c','#f4f1f8','#89848f','#2a2631'],midnight:['#0d101b','#f4f1f8','#8e93a6','#282e42'],dark:['#15131a','#f4f1f8','#96909f','#342e3d'],light:['#f7f5fa','#211b2b','#797082','#ddd6e5']};
 const colors=themes[s().theme]||themes.black;['--bg','--text','--muted','--line'].forEach((v,i)=>document.documentElement.style.setProperty(v,colors[i]));
 document.documentElement.style.colorScheme=s().theme==='light'?'light':'dark';document.documentElement.dataset.theme=s().theme;
 $('#account-name').textContent=state.account?.name||window.KyroI18n.t('Аккаунты');
 $('#account-type').textContent=state.account?(state.account.kind==='offline'?'Офлайн-аккаунт':'Microsoft · Java Edition'):'Не выбран';
 $('.avatar').textContent=state.account?state.account.name.slice(0,1).toUpperCase():'+';
 $('#footer-version').textContent=`${s().version} · ${s().loader.toUpperCase()}`;
 $('#memory-status').textContent=`${s().memory_gb} GB RAM`;
 $('#ready-status').textContent=state.busy?'ВЫПОЛНЯЕТСЯ ОПЕРАЦИЯ':'JAVA EDITION';
 $('#navigation').innerHTML=Object.entries(labels).map(([key,label])=>`<button data-page="${key}" class="${page===key?'active':''}" ${page===key?'aria-current="page"':''}>${icon(key==='notifications'?'bell':key)}<span>${label}</span>${key==='notifications'&&state.data.notices.some(n=>n.unread)?`<span class="badge">${state.data.notices.filter(n=>n.unread).length}</span>`:''}</button>`).join('');
 $$('#navigation button').forEach(b=>b.onclick=()=>navigate(b.dataset.page));
 $('#breadcrumb').textContent=page==='play'?'MINECRAFT · JAVA EDITION':`KYRO / ${labels[page].toUpperCase()}`;
}
async function persist(){
 try {if(native)await invoke('save_settings',{settings:s()});else localStorage.setItem('kyro-preview-settings',JSON.stringify(s()));updateChrome();return true;}catch(e){failure(e);return false;}
}
function setSetting(key,value,rerender=false){s()[key]=value;updateChrome();clearTimeout(saveTimer);saveTimer=setTimeout(persist,350);if(rerender)render();}
async function refreshActivity(){if(!native||page!=='activity')return;try{await sync();if(page==='activity')render();}catch(e){failure(e);}}
function navigate(next){page=next;updateChrome();render();window.KyroAssist.enter(next);if(page==='activity')void refreshActivity();if(page==='catalog'&&!catalogData&&!catalogLoading)loadCatalog();}
function heading(title,subtitle='',right=''){return `<div class="page-heading"><div><h1>${title}</h1>${subtitle?`<p>${subtitle}</p>`:''}</div>${right}</div>`;}
function empty(title,text,ico='box'){return `<div class="empty">${icon(ico)}<strong>${title}</strong>${text}</div>`;}
function render(){
 document.getElementById('activity-tooltip')?.remove();
 const focused=document.activeElement?.id==='search-query',caret=focused?document.activeElement.selectionStart:null;
 $('#page').innerHTML=(!native?'<div class="preview-banner">Предпросмотр интерфейса · запустите EXE для доступа к Rust, Minecraft и Modrinth.</div>':'')+({play:home,catalog:catalog,activity:activity,notifications:notifications,settings:settings,screenshots:window.KyroAssist.screenshots,diagnostics:window.KyroAssist.diagnostics}[page])();
 bind();
 window.KyroFeatures.bind();
 window.KyroServers.bind();
 window.KyroAssist.bind();
 window.KyroPickers.enhance($('#page'));
 document.querySelectorAll('#page button').forEach(button=>{const text=button.textContent.trim();if(!button.querySelector('svg')&&['Открыть папку','Выбрать профиль','Обновить','Лог этого профиля','Папка диагностики','Сбросить к рекомендуемым'].includes(text)){button.classList.add('soft-action');button.insertAdjacentHTML('afterbegin',icon(text.includes('папк')||text.includes('Папка')?'folder':text==='Обновить'?'refresh':text.includes('Лог')?'box':'arrow'));}});
 document.querySelectorAll('input[type=range]').forEach(el=>{const paint=()=>el.style.setProperty('--range-fill',((Number(el.value)-Number(el.min||0))/(Number(el.max||100)-Number(el.min||0))*100)+'%');el.addEventListener('input',paint);paint();});
 document.querySelectorAll('form,input').forEach(el=>el.setAttribute('autocomplete','off'));
 if(focused&&$('#search-query')){$('#search-query').focus();$('#search-query').setSelectionRange(caret,caret);}
}
function packSelector(){const packs=state.data.modpacks||[];return packs.length?`<label class="pack-selector">Игровой профиль<select id="active-modpack" aria-label="Игровой профиль" ${state.busy?'disabled':''}><option value="">Обычный профиль</option>${packs.map(p=>`<option value="${esc(p.id)}" ${p.id===s().instance_id?'selected':''}>${esc(p.name)} · ${esc(p.version_name)}</option>`).join('')}</select></label>`:'';}
function updateLauncherBackground(){
 const selected=(state.data.modpacks||[]).find(p=>p.id===s().instance_id);
 const candidate=s().home_background||selected?.cover_image||'';
 const image=/^data:image\/(?:png|jpeg|webp);base64,[A-Za-z0-9+/=]+$/.test(candidate)?candidate:'';
 const background=document.getElementById('launcher-background');
 if(!background)return;
 const value=image?'url("'+image+'")':'none';
 if(background.style.backgroundImage!==value)background.style.backgroundImage=value;
 document.getElementById('app').classList.toggle('has-background',!!image);
}
function home(){return `<div class="home ${s().home_layout==='card'?'home-card':''}"><div class="home-copy profile-launch">${packSelector()}<button id="change-profile" class="profile-identity" aria-haspopup="dialog" aria-label="Изменить версию и ядро" title="Выбрать версию Minecraft и ядро" ${state.busy||s().instance_id?'disabled':''}><span class="version-display">${esc(s().version)}</span><span class="profile-core">${esc(s().loader)}<span class="profile-edition">JAVA EDITION</span></span><svg class="profile-chevron" viewBox="0 0 20 20" aria-hidden="true"><path d="m5 7.5 5 5 5-5"/></svg></button><div class="profile-tools"><button id="fps-boost" class="fps-boost ${state.fps_enabled>0?'has-boost':''}" ${state.busy?'disabled':''} title="Настроить оптимизацию Minecraft" aria-label="FPS Boost${state.fps_enabled>0?' включён':''}"><span class="boost-mark" aria-hidden="true">ϟ</span><span>FPS Boost</span><span class="boost-status">${state.fps_enabled>0?'Включён':'Не включён'}</span></button><button id="game-folder" class="home-folder" data-folder="instance" title="Открыть папку текущего профиля" aria-label="Открыть папку текущего профиля">${icon('folder')}</button></div><button class="primary play-button" id="play" ${state.busy&&!state.launch?.can_cancel?'disabled':''}>${icon('play')}<span>${state.launch?.can_cancel?'Отменить':state.busy?'ЗАПУСК…':'Играть'}</span></button><div class="home-note">${state.busy?'Подготавливаем игру…':state.account?'<span class="ready-dot"></span>Готов к запуску':'Добавь аккаунт, чтобы начать игру'}</div></div><div class="home-orbit"></div>${window.KyroServers.card()}</div>`;}
function activity(){
 const sessions=state.data.sessions;
 const total=sessions.reduce((n,x)=>n+x.seconds,0), byDay=new Map();
 sessions.forEach(x=>{const k=dateKey(new Date(x.started));const d=byDay.get(k)||{seconds:0,count:0};d.seconds+=x.seconds;d.count++;byDay.set(k,d);});
 const today=new Date();today.setHours(0,0,0,0);const monday=new Date(today);monday.setDate(monday.getDate()-(monday.getDay()+6)%7);
 const week=sessions.filter(x=>new Date(x.started)>=monday).reduce((n,x)=>n+x.seconds,0);
 const streak=activityStreak(sessions);
 const best=Math.max(0,...sessions.map(x=>x.seconds));
 const start=new Date(year,0,1);start.setDate(start.getDate()-(start.getDay()+6)%7);
 const cells=[];for(let i=0;i<371;i++){
  const d=new Date(start);d.setDate(start.getDate()+i);const k=dateKey(d), entry=byDay.get(k)||{seconds:0,count:0};
  const level=entry.seconds>=10800?4:entry.seconds>=5400?3:entry.seconds>=1800?2:entry.seconds>0?1:0;
  const title=`${d.toLocaleDateString(window.KyroI18n.locale(),{day:'numeric',month:'long',year:'numeric'})} · ${duration(entry.seconds)} · сессий: ${entry.count}`;
  cells.push(`<span tabindex="0" class="cell ${d>today?'future':''}" data-level="${level}" data-activity-date="${esc(d.toLocaleDateString(window.KyroI18n.locale(),{day:'numeric',month:'long',year:'numeric'}))}" data-activity-time="${esc(duration(entry.seconds))}" data-activity-count="${entry.count}" aria-label="${esc(title)}"></span>`);
 }
 const versionMap=new Map();sessions.forEach(x=>{const k=`${x.version} ${x.loader}`;versionMap.set(k,(versionMap.get(k)||0)+x.seconds);});
 const versions=[...versionMap].sort((a,b)=>b[1]-a[1]);
 return heading('Активность','Твоя история Minecraft')+`<div class="stats">${[['ВСЕГО В ИГРЕ',duration(total)],['ЭТА НЕДЕЛЯ',duration(week)],['СЕРИЯ',`${streak} дн.`],['РЕКОРД СЕССИИ',duration(best)]].map(([label,value])=>`<div class="stat"><label>${label}</label><strong>${value}</strong></div>`).join('')}</div><div class="heatmap-wrap"><div class="months">${['янв','фев','мар','апр','май','июн','июл','авг','сен','окт','ноя','дек'].map(m=>`<span>${m}</span>`).join('')}</div><div class="heatmap-body"><div class="weekdays"><span>Пн</span><span>Ср</span><span>Пт</span></div><div class="heatmap">${cells.join('')}</div></div><div class="heatmap-key"><p>Каждая клетка — один день. Наведи курсор: время и число сессий.</p><span>Меньше</span>${[0,1,2,3,4].map(i=>`<span class="cell" data-level="${i}"></span>`).join('')}<span>Больше</span></div></div><div class="activity-lower"><div><div class="activity-block"><h3>ЛЮБИМЫЕ ВЕРСИИ</h3>${versions.length?versions.slice(0,4).map(([name,time])=>`<div class="bar-row"><span>${esc(name)}</span><div class="bar"><span style="width:${time/total*100}%"></span></div><small>${duration(time)}</small></div>`).join(''):'<p class="help">Здесь появятся версии, в которых ты играешь чаще всего.</p>'}</div><div class="activity-block"><h3>ОБ УЧЁТЕ АКТИВНОСТИ</h3><p class="help">Учитываются запуски из KYRO: от старта Java до закрытия игры. Время хранится локально на этом компьютере. Статистика по отдельным серверам требует игрового мода и пока не собирается.</p></div></div><div><h3>ПОСЛЕДНИЕ СЕССИИ</h3>${sessions.length?sessions.slice(-8).reverse().map(x=>`<div class="session-row"><span>${esc(new Date(x.started).toLocaleDateString(window.KyroI18n.locale(),{day:'numeric',month:'short'}))}<small>${esc(new Date(x.started).toLocaleTimeString(window.KyroI18n.locale(),{hour:'2-digit',minute:'2-digit'}))}</small></span><span class="session-profile"><span>${esc(x.version)}</span><small>${esc(x.loader)}</small></span><span>${duration(x.seconds)}</span></div>`).join(''):empty('История начинается здесь','Запусти Minecraft из KYRO — первая сессия появится после закрытия игры.','clock')}</div></div>`;
}
function catalog(){
 if(catalogKind==='compatibility')return window.KyroAssist.compatibility();
 if(catalogKind==='manage')catalogKind='installed';
 const categories=[['modpack','Сборки'],['mod','Моды'],['shader','Шейдеры'],['resourcepack','Ресурспаки'],['installed','Установленные']];
 const data=catalogData, installed=catalogKind==='installed';
 let content='';
 if(installed)content=window.KyroFeatures.manager();
 else if(catalogLoading)content=empty('Ищем проекты…','Загрузка совместимых проектов.','search');
 else if(catalogError)content=empty('Не удалось загрузить каталог',esc(catalogError))+'<button id="retry-catalog">Повторить</button>';
 else if(data?.hits?.length)content=data.hits.map((m,index)=>{
 const pack=catalogKind==='modpack'?(state.data.modpacks||[]).find(p=>p.project_id===m.project_id&&p.game_version===s().version&&p.loader===s().loader):null;
 const installed=catalogKind!=='modpack'&&state.data.installed.some(x=>x.kind===catalogKind&&x.project_id===m.project_id&&x.game_version===s().version&&x.loader===s().loader&&(x.instance_id||'')===(s().instance_id||''));
 const image=typeof m.icon_url==='string'&&['https://cdn.modrinth.com/','https://media.forgecdn.net/','https://mediafilez.forgecdn.net/'].some(prefix=>m.icon_url.startsWith(prefix))?`<img class="mod-icon" src="${esc(m.icon_url)}" alt="" loading="lazy" referrerpolicy="no-referrer">`:`<div class="mod-icon">${icon('box')}</div>`;
 return `<div class="mod-row">${image}<div class="mod-info"><strong>${esc(m.title)}</strong><p>${esc(m.description)}</p><small>${esc(m.author)} · ${new Intl.NumberFormat('ru',{notation:'compact'}).format(m.downloads)} загрузок · ${esc(s().version)}</small></div><button ${pack?`data-launch-pack="${esc(pack.id)}"`:`data-install="${index}"`} ${state.busy||installed?'disabled':''}>${pack?'Запустить':installed?'Установлено':'Установить'}</button></div>`;
 }).join('');else content=empty('Ничего не найдено','Попробуй другой запрос или версию Minecraft.','search');
 return heading('Каталог','Проекты для твоего Minecraft')+`<div class="tabs">${categories.map(([k,t])=>`<button data-kind="${k}" class="${catalogKind===k?'selected':''}">${t}</button>`).join('')}</div>${!installed?`<form class="searchbox" id="catalog-search">${icon('search')}<input id="search-query" autocomplete="off" autocorrect="off" spellcheck="false" placeholder="Найти в каталоге" value="${esc(catalogQuery)}" aria-label="Поиск в каталоге"><span class="search-hint">Поиск при вводе</span></form>`:''}<div class="filters"><select id="catalog-version" aria-label="Версия Minecraft в каталоге">${[...new Set([s().version,...releaseVersions])].map(v=>`<option value="${esc(v)}" ${v===s().version?'selected':''}>Minecraft ${esc(v)}</option>`).join('')}</select><select id="catalog-loader" aria-label="Загрузчик в каталоге">${['Vanilla','Fabric','Quilt','Forge','NeoForge'].map(l=>`<option ${l===s().loader?'selected':''}>${l}</option>`).join('')}</select><span class="result-count">${!installed&&data?`${data.total_hits.toLocaleString('ru')} проектов`:''}</span></div>${catalogKind==='modpack'?'<p class="help">Сборки устанавливаются в отдельные профили. Клиентские необязательные файлы включены; серверные исключены. Поддерживаются Fabric, Quilt, Forge, NeoForge и Vanilla.</p>':''}${!installed&&data?.warning?`<p class="help" role="status">${esc(data.warning)}</p>`:''}<div id="catalog-results">${content}</div>${!installed&&data?`<div class="pagination"><button id="previous-page" ${catalogOffset===0||catalogLoading?'disabled':''}>←</button><span>${Math.floor(catalogOffset/12)+1} / ${data.page_count}</span><button id="next-page" ${Math.floor(catalogOffset/12)+1>=data.page_count||catalogLoading?'disabled':''}>→</button></div>`:''}`;
}
async function loadCatalog(){
 if(catalogKind==='installed'){await window.KyroFeatures.scan(false);return;}
 if(catalogKind==='compatibility'){await window.KyroAssist.load('compatibility');return;}
 if(catalogKind==='manage'){await window.KyroFeatures.scan(false);return;}
 const seq=++catalogSeq;catalogLoading=true;catalogError='';render();
 try{const data=await window.KyroCatalog.search(invoke,{query:catalogQuery,kind:catalogKind,version:s().version,loader:s().loader,offset:catalogOffset});if(seq===catalogSeq)catalogData=data;}
 catch(e){if(seq===catalogSeq){catalogData=null;catalogError=String(e?.message||e);}}
 finally{if(seq===catalogSeq){catalogLoading=false;if(page==='catalog')render();}}
}
function launcherNoticeInstalled(id){
 const match=/^launcher-update-v?(\d+)\.(\d+)\.(\d+)$/.exec(id);
 if(!match)return false;
 const installed=CURRENT_LAUNCHER_VERSION.split('.').map(Number);
 const release=match.slice(1).map(Number);
 for(let i=0;i<3;i++){if(release[i]!==installed[i])return release[i]<installed[i];}
 return true;
}
function notifications(){
 const list=state.data.notices.filter(n=>noticeFilter!=='unread'||n.unread);const groups=new Map();list.forEach(n=>{const key=new Date(n.time).toLocaleDateString(window.KyroI18n.locale(),{day:'numeric',month:'long'});if(!groups.has(key))groups.set(key,[]);groups.get(key).push(n);});
 return heading('Уведомления','Загрузки, обновления и сообщения лаунчера',`<button id="clear-notices" ${state.data.notices.length?'':'disabled'}>${icon('trash')}Очистить</button>`)+`<div class="tabs"><button data-notice-filter="all" class="${noticeFilter==='all'?'selected':''}">Все</button><button data-notice-filter="unread" class="${noticeFilter==='unread'?'selected':''}">Непрочитанные · ${state.data.notices.filter(n=>n.unread).length}</button><button id="read-notices">Прочитать все</button></div>${list.length?[...groups].map(([day,items])=>`<div class="notice-group"><h3>${esc(day.toUpperCase())}</h3>${items.map(n=>`<div class="notice">${n.unread?'<span class="dot"></span>':''}${icon('bell')}<div class="notice-content"><strong>${esc(launcherNoticeInstalled(n.id)?`KYRO ${n.id.replace(/^launcher-update-/,'')} · установлено`:n.title)}</strong><p>${esc(launcherNoticeInstalled(n.id)?'Эта версия уже установлена. Повторное обновление не требуется.':n.body)}</p>${n.id.startsWith('launcher-update-')?(launcherNoticeInstalled(n.id)?'<span class="notice-installed" role="status">✓ Уже установлено</span>':`<button class="primary notice-update" data-launcher-update ${state.busy?'disabled':''}>Обновить KYRO</button>`):''}</div><time>${esc(new Date(n.time).toLocaleTimeString(window.KyroI18n.locale(),{hour:'2-digit',minute:'2-digit'}))}</time><button data-dismiss="${esc(n.id)}" aria-label="Удалить уведомление">×</button></div>`).join('')}</div>`).join(''):`<div class="notice-empty"><div class="notice-empty-art" aria-hidden="true"><span class="notice-empty-ring"></span><span class="notice-empty-icon">${icon('bell')}</span><span class="notice-empty-check">✓</span></div><strong>${noticeFilter==='unread'?'Всё прочитано':'Пока ничего нового'}</strong><p>${noticeFilter==='unread'?'Непрочитанные уведомления появятся здесь.':'Сообщим, когда появятся обновления или завершатся загрузки.'}</p><span class="notice-empty-caption">МОЖНО ВОЗВРАЩАТЬСЯ К ИГРЕ</span></div>`}`;
}
function segments(key,options){return `<div class="segments">${options.map(([v,label])=>`<button data-setting="${key}" data-value="${v}" class="${s()[key]===v?'selected':''}">${label}</button>`).join('')}</div>`;}
function colorPalette(){return `<div class="palette" role="group" aria-label="Палитра цветов">${[['#a185cf','Лаванда'],['#8b5cf6','Фиолетовый'],['#6b9fff','Синий'],['#52c7c3','Бирюзовый'],['#80c99a','Зелёный'],['#e6b865','Золотистый'],['#e894bc','Розовый'],['#e77e85','Коралловый'],['#d9dce7','Серебристый']].map(([hex,name])=>`<button data-setting="accent" data-value="${hex}" style="--swatch:${hex}" aria-label="${name}" title="${name}" aria-pressed="${s().accent.toLowerCase()===hex}" class="swatch ${s().accent.toLowerCase()===hex?'selected':''}"><span></span>${s().accent.toLowerCase()===hex?icon('check'):''}</button>`).join('')}</div>`;}
function folderAction(target,label,path){return `<button data-folder="${target}" class="folder-action"><span>${icon('folder')}${label}</span><small title="${esc(path)}">${esc(path)}</small></button>`;}
function section(title,description,controls){return `<div class="setting-section"><div><h3>${title}</h3><p>${description}</p></div><div class="setting-controls">${controls}</div></div>`;}
function settings(){
 let content='';
 if(settingsTab==='general')content=
 section('ЯЗЫК','Язык интерфейса лаунчера.',segments('language',[['ru','Русский'],['en','English']]))+
 section('ОФОРМЛЕНИЕ','Твой стиль. Ничего лишнего.',`<div class="setting-row"><span>Тема</span>${segments('theme',[['dark','Тёмная'],['midnight','Полночь'],['black','Чёрная'],['light','Светлая']])}</div><div class="setting-row palette-row"><span>Палитра</span>${colorPalette()}</div>`)+
 section('ПРИ ЗАПУСКЕ ИГРЫ','Что делать с окном лаунчера.',segments('launch_behavior',[['hide','Скрыть'],['minimize','Свернуть'],['open','Оставить открытым']])+'<p>После закрытия игры лаунчер вернётся.</p>')+
 section('DISCORD','Показывать активность Minecraft.',`<label class="setting-row">Активность Discord<input type="checkbox" id="discord-enabled" ${s().discord_enabled!==false?'checked':''}></label><label class="setting-row">Показывать адрес сервера<input type="checkbox" id="discord-server" ${s().discord_show_server!==false?'checked':''}></label>`)+
 section('УЧЁТНАЯ ЗАПИСЬ','Minecraft Java Edition.',`<div class="setting-row"><span>${esc(state.account?.name||'Аккаунт не выбран')}</span><button id="settings-account">${state.account?'Управлять':'Добавить аккаунт'}</button></div>`);
 if(settingsTab==='java')content=`<div class="technical-grid"><div>`+
 section('ПАМЯТЬ',`${s().memory_gb} ГБ выделено для игры.`,segments('memory_gb',[[2,'2 ГБ'],[4,'4 ГБ'],[6,'6 ГБ'],[8,'8 ГБ'],[12,'12 ГБ']])+`<input type="range" id="memory-range" aria-label="Память в ГБ" min="2" max="32" value="${s().memory_gb}"><p>Выделяй не больше половины доступной RAM.</p>`)+
 section('JAVA','Minecraft 1.21 требует Java 21 x64.',`<input id="java-path" aria-label="Путь к Java" value="${esc(s().java_path)}" placeholder="C:\\Program Files\\Java\\bin\\java.exe"><p>Можно указать полный путь к java.exe или команду из PATH.</p>`)+
 `</div><div>`+section('АРГУМЕНТЫ JVM','Передаются Java напрямую, без командной оболочки.',`<textarea id="jvm-args" class="code-input" spellcheck="false" aria-label="Аргументы JVM">${esc(s().jvm_args)}</textarea><button id="reset-jvm">Сбросить к рекомендуемым</button>`)+`</div></div>`;
 if(settingsTab==='downloads')content=section('ЗАГРУЗКИ','Параллельная загрузка файлов.',`<div class="setting-row"><span>Потоков: <b id="threads-value">${s().download_threads}</b></span></div><input id="threads" type="range" min="1" max="16" aria-label="Потоки загрузки" value="${s().download_threads}">`)+section('ФАЙЛЫ И ЦЕЛОСТНОСТЬ','Проверка по SHA-1 при загрузке Minecraft и модов.',`<div class="stack"><button id="verify" ${state.busy?'disabled':''}>${icon('check')} Перепроверить файлы Minecraft</button>${folderAction('instance','Папка профиля',state.paths?.profile||`data/instances/${s().version}-${s().loader}`)}${folderAction('launcher','Папка лаунчера',state.paths?.launcher||'Каталог с EXE')}${folderAction('root','Данные лаунчера',state.paths?.data||state.root||'Каталог данных')}<button data-folder="log">${icon('box')} Лог игры</button><button data-folder="core-log">${icon('box')} Лог установки ядра</button></div><p>Папка профиля содержит моды, миры и настройки выбранной версии и ядра. Папка данных содержит общий кеш и настройки KYRO.</p>`);
 if(settingsTab==='game')content=window.KyroAssist.presets();
 if(settingsTab==='general')content+=window.KyroFeatures.settings();
 if(settingsTab==='java')content+=window.KyroFeatures.javaSettings();
 return heading('Настройки')+`<div class="tabs">${[['general','Основные'],['java','Java и память'],['downloads','Загрузки'],['game','Профили графики']].map(([key,label])=>`<button data-settings-tab="${key}" class="${settingsTab===key?'selected':''}">${label}</button>`).join('')}</div>${content}<div class="settings-end"><span id="save-status">Изменения сохраняются автоматически</span><span>KYRO / ${CURRENT_LAUNCHER_VERSION}</span></div>`;
}
function bind(){

 for(const [id,key] of [['discord-enabled','discord_enabled'],['discord-server','discord_show_server']]){const input=$('#'+id);if(input)input.onchange=()=>{s()[key]=input.checked;persist();};}
 const pack=$('#active-modpack');if(pack)pack.onchange=async()=>{try{await invoke('select_modpack',{id:pack.value});await sync();catalogData=null;render();}catch(e){failure(e);}};
 $$('[data-launch-pack]').forEach(b=>b.onclick=async()=>{try{await invoke('select_modpack',{id:b.dataset.launchPack});await sync();navigate('play');await runOperation('Запуск сборки…',async()=>{if(!await window.KyroAssist.confirmCompatibility())return false;await invoke('launch_game');},true);}catch(e){failure(e);}});
 if($('#change-profile'))$('#change-profile').onclick=e=>window.KyroPickers.openVersions(e.currentTarget);
 const core=$('#game-loader');if(core)core.onchange=async()=>{s().loader=core.value;catalogData=null;if(await persist()){await sync();render();}};
 $$('[data-loader]').forEach(b=>b.onclick=async()=>{s().loader=b.dataset.loader;catalogData=null;if(await persist())render();});
 const version=$('#game-version');if(version)version.onchange=async()=>{s().version=version.value;catalogData=null;if(await persist()){await sync();render();}};
 if($('#fps-boost'))$('#fps-boost').onclick=openFpsBoost;
 if($('#play'))$('#play').onclick=()=>state.launch?.can_cancel?$('#cancel-launch').click():runOperation('Подготовка Minecraft…',async()=>{if(!await persist())return false;if(!await window.KyroAssist.confirmCompatibility())return false;await invoke('launch_game');},true);
 $$('[data-kind]').forEach(b=>b.onclick=()=>{catalogKind=b.dataset.kind;catalogOffset=0;catalogData=null;catalogSeq++;catalogLoading=false;render();loadCatalog();});
 if($('#search-query'))$('#search-query').oninput=e=>{catalogQuery=e.target.value;catalogOffset=0;catalogSeq++;clearTimeout(catalogTimer);catalogTimer=setTimeout(()=>{if(page==='catalog'&&!['installed','compatibility'].includes(catalogKind))loadCatalog();},300);};
 if($('#catalog-search'))$('#catalog-search').onsubmit=e=>{e.preventDefault();catalogQuery=$('#search-query').value.trim();catalogOffset=0;loadCatalog();};
 if($('#retry-catalog'))$('#retry-catalog').onclick=loadCatalog;
 if($('#catalog-profile'))$('#catalog-profile').onclick=()=>navigate('play');
 for(const [id,key] of [['catalog-version','version'],['catalog-loader','loader']]){const input=$('#'+id);if(input)input.onchange=async()=>{s()[key]=input.value;catalogOffset=0;catalogData=null;if(await persist()){await sync();loadCatalog();}};}
 if($('#previous-page'))$('#previous-page').onclick=()=>{catalogOffset=Math.max(0,catalogOffset-12);loadCatalog();};
 if($('#next-page'))$('#next-page').onclick=()=>{catalogOffset+=12;loadCatalog();};
 $$('[data-install]').forEach(b=>b.onclick=async()=>{const item=catalogData.hits[Number(b.dataset.install)],kind=catalogKind;const ok=await runOperation(`Установка ${item.title}…`,()=>invoke('install_mod',{project:item.project_id,name:item.title,kind}));if(ok)window.KyroFeatures.invalidateUpdates();if(ok&&kind==='modpack'){catalogData=null;navigate('play');toast('Сборка установлена и выбрана. Нажми «Играть» для запуска.');}});
 $$('[data-remove-mod]').forEach(button=>button.onclick=async()=>{const ok=await runOperation('Удаление мода…',()=>invoke('remove_mod',{filename:button.dataset.removeMod}));if(ok){window.KyroFeatures.invalidateUpdates();toast('Мод удалён из профиля.');}});
 $$('[data-notice-filter]').forEach(b=>b.onclick=()=>{noticeFilter=b.dataset.noticeFilter;render();});
 for(const [id,action]of [['clear-notices','clear'],['read-notices','read']]){if($('#'+id))$('#'+id).onclick=async()=>{try{await invoke('notices',{action,id:null});await sync();render();}catch(e){failure(e);}};}
 $$('[data-dismiss]').forEach(b=>b.onclick=async()=>{try{await invoke('notices',{action:'dismiss',id:b.dataset.dismiss});await sync();render();}catch(e){failure(e);}});
 $$('[data-settings-tab]').forEach(b=>b.onclick=()=>{settingsTab=b.dataset.settingsTab;render();});
 $$('[data-setting]').forEach(b=>b.onclick=async()=>{if(b.dataset.setting==='language'){const previous=s().language;s().language=b.dataset.value;if(!await persist()){s().language=previous;updateChrome();}render();}else setSetting(b.dataset.setting,b.dataset.setting==='memory_gb'?Number(b.dataset.value):b.dataset.value,true);});
 const inputs=[['accent-color','accent',false],['memory-range','memory_gb',true],['java-path','java_path',false],['jvm-args','jvm_args',false],['threads','download_threads',true]];
 inputs.forEach(([id,key,num])=>{const el=$('#'+id);if(el)el.onchange=()=>{setSetting(key,num?Number(el.value):el.value);if(id==='memory-range'||id==='threads'||id==='accent-color')render();};});
 if($('#reset-jvm'))$('#reset-jvm').onclick=()=>setSetting('jvm_args',defaults.jvm_args,true);
 if($('#settings-account'))$('#settings-account').onclick=openAccount;
 if($('#verify'))$('#verify').onclick=()=>runOperation('Проверка Minecraft…',()=>invoke('verify_files'));
 $$('[data-folder]').forEach(b=>b.onclick=()=>invoke('open_folder',{target:b.dataset.folder}).then(path=>toast('Открыто: '+path)).catch(failure));
}
async function runOperation(message,task,isGame=false,isUpdate=false){
 if(state.busy)return toast('Дождитесь завершения текущей операции.');
 state.busy=true;state.updateProgress=isUpdate;$('#operation').hidden=!isUpdate;$('#operation-label').textContent=message;$('#operation-progress').removeAttribute('value');$('#operation-percent').textContent='…';$('#operation-speed').hidden=true;updateChrome();render();$('#cancel-launch').hidden=true;
 try{const result=await task();if(result===false)return false;toast(isGame?'Minecraft запущен. Можно запустить ещё один экземпляр.':'Готово.');return true;}
 catch(e){if(String(e)==='Запуск отменён'){toast('Запуск отменён.');return false;}failure(e);if(isGame)navigate('diagnostics');return false;}
 finally{state.busy=false;state.updateProgress=false;$('#cancel-launch').hidden=true;$('#operation').hidden=true;await sync().catch(failure);render();}
}
function openAccount(){
 clearTimeout(loginTimer);const dialog=$('#account-dialog');
 const accounts=state.data.offline_accounts||[];
 const ms=state.microsoft_account||(state.account?.kind!=='offline'?state.account:null);
 $('#account-content').innerHTML=`<div class="account-list">${accounts.map(a=>`<div class="account-row"><button data-select-account="offline:${esc(a.uuid)}" ${state.busy?'disabled':''}><span class="avatar">${esc(a.name[0])}</span><span><strong>${esc(a.name)}</strong><small>Офлайн-аккаунт</small></span>${state.account?.kind==='offline'&&state.account.uuid===a.uuid?icon('check'):''}</button><button data-remove-account="offline:${esc(a.uuid)}" aria-label="Удалить ${esc(a.name)}" ${state.busy?'disabled':''}>×</button></div>`).join('')}${ms?`<div class="account-row"><button data-select-account="microsoft" ${state.busy?'disabled':''}><span class="avatar">${esc(ms.name[0])}</span><span><strong>${esc(ms.name)}</strong><small>Microsoft</small></span>${state.account?.kind==='microsoft'?icon('check'):''}</button><button id="logout" title="Выйти из Microsoft" ${state.busy?'disabled':''}>${icon('logout')}</button></div>`:''}</div><form id="offline-form"><label class="dialog-field">Ник офлайн-аккаунта<input id="offline-name" placeholder="Например, Nova" minlength="3" maxlength="16" pattern="[A-Za-z0-9_]{3,16}" autocomplete="off" spellcheck="false" required></label><button type="submit" class="primary" ${state.busy?'disabled':''}>Добавить офлайн-аккаунт</button><p class="help">От 3 до 16 латинских букв, цифр или _. Без пароля и Microsoft. Для серверов с проверкой лицензии нужен Microsoft-аккаунт.</p><p id="offline-error" role="alert"></p></form>${!ms?`<details class="microsoft-login"><summary>Войти через Microsoft</summary><label class="dialog-field">Microsoft OAuth Client ID<input id="client-id" value="${esc(s().microsoft_client_id)}" placeholder="ID зарегистрированного приложения" autocomplete="off"></label><p class="help">Нужен Client ID приложения с public client flow и доступом Minecraft API. Пароль вводится только на сайте Microsoft.</p><button id="begin-login" ${state.busy?'disabled':''}>Войти в Microsoft</button><div id="login-status"></div></details>`:''}`;
 if(!dialog.open)dialog.showModal();
 const changeAccount=async(action,value)=>{await invoke('offline_account',{action,value});await sync();render();};
 $('#offline-form').onsubmit=async e=>{e.preventDefault();const b=$('#offline-form button');b.disabled=true;try{await changeAccount('add',$('#offline-name').value.trim());dialog.close();toast('Офлайн-аккаунт добавлен и выбран.');}catch(err){$('#offline-error').textContent=String(err?.message||err);b.disabled=false;}};
 $$('[data-select-account]').forEach(b=>b.onclick=async()=>{try{await changeAccount('select',b.dataset.selectAccount);dialog.close();}catch(e){failure(e);}});
 $$('[data-remove-account]').forEach(b=>b.onclick=async()=>{try{await changeAccount('remove',b.dataset.removeAccount);openAccount();}catch(e){failure(e);}});
 if($('#logout'))$('#logout').onclick=async()=>{try{await invoke('logout');await sync();dialog.close();render();toast('Вы вышли из Microsoft.');}catch(e){failure(e);}};
 if($('#begin-login'))$('#begin-login').onclick=async()=>{
  const button=$('#begin-login');button.disabled=true;s().microsoft_client_id=$('#client-id').value.trim();
  try{if(!await persist())throw new Error('Не удалось сохранить настройки');const v=await invoke('login_begin');
   const uri=v.verification_uri;const safeUri=typeof uri==='string'&&uri.startsWith('https://')?uri:'https://www.microsoft.com/link';
   $('#login-status').innerHTML=`<p>Откройте ${esc(safeUri)} в браузере и введите код:</p><button id="open-login-browser">Открыть Microsoft</button><div class="device-code">${esc(v.user_code)}</div><p class="help busy-indicator">Ожидаем подтверждение…</p>`;
   $('#open-login-browser').onclick=()=>invoke('open_microsoft_login',{url:safeUri}).catch(failure);
   const expiry=Date.now()+(v.expires_in||900)*1000;
   const poll=async()=>{if(!dialog.open)return;if(Date.now()>expiry){$('#login-status').textContent='Срок кода истёк. Попробуйте снова.';button.disabled=false;return;}
    try{const r=await invoke('login_poll',{code:v.device_code});if(r.status==='complete'){await sync();dialog.close();render();toast('Microsoft подключён. Можно запускать Minecraft.');return;}loginTimer=setTimeout(poll,(r.status==='slow_down'?10:(v.interval||5))*1000);}catch(e){$('#login-status').textContent=String(e);button.disabled=false;}
   };loginTimer=setTimeout(poll,(v.interval||5)*1000);
  }catch(e){failure(e);button.disabled=false;}
 };
}
$('#profile-button').onclick=openAccount;$('#dialog-close').onclick=()=>{$('#account-dialog').close();clearTimeout(loginTimer);};
$('#cancel-launch').onclick=async()=>{try{const accepted=await invoke('cancel_launch');toast(accepted?'Отменяем подготовку…':'Отмена подготовки сейчас недоступна.');}catch(e){failure(e);}};
$('#operation-dismiss').onclick=()=>$('#operation').hidden=true;
$('#minimize').onclick=()=>native&&window.__TAURI__.window.getCurrentWindow().minimize();
$('#maximize').onclick=()=>native&&window.__TAURI__.window.getCurrentWindow().toggleMaximize();
$('#close').onclick=()=>{if(state.busy){toast('Дождитесь завершения игры или установки, чтобы сохранить историю сессии.');return;}if(native)window.__TAURI__.window.getCurrentWindow().close();};
async function boot(){
 window.addEventListener('focus',()=>void refreshActivity());
 document.addEventListener('visibilitychange',()=>{if(!document.hidden)void refreshActivity();});
 refreshTimer=setInterval(()=>{if(!document.hidden)void refreshActivity();},60000);
 if(native){window.KyroFeatures.checkUpdate(true);setInterval(()=>window.KyroFeatures.checkUpdate(true),60*1000);window.addEventListener("focus",()=>window.KyroFeatures.checkUpdate(true));}
 try{if(!native){const saved=JSON.parse(localStorage.getItem('kyro-preview-settings')||'null');if(saved)saved&&Object.assign(state.data.settings,saved);}await sync();render();
 if(native){await window.__TAURI__.event.listen('launch-state',e=>{state.launch=e.payload;$('#cancel-launch').hidden=!e.payload.can_cancel;render();});await window.__TAURI__.event.listen('progress',e=>{window.KyroPickers.progress(e.payload.message);if(!state.updateProgress){$('#operation').hidden=true;return;}$('#operation').hidden=false;$('#operation-label').textContent=e.payload.message;const fraction=Number(e.payload.fraction);if(e.payload.fraction!=null&&Number.isFinite(fraction)){const value=Math.max(0,Math.min(1,fraction));$('#operation-progress').value=value;$('#operation-percent').textContent=Math.round(value*100)+'%';}else{$('#operation-progress').removeAttribute('value');$('#operation-percent').textContent='…';}const speed=Number(e.payload.bytes_per_second);$('#operation-speed').hidden=!(Number.isFinite(speed)&&speed>0);$('#operation-speed').textContent='· '+(speed/1048576).toLocaleString(window.KyroI18n.locale(),{maximumFractionDigits:1})+' МБ/с';$('#operation').title=e.payload.message;if(e.payload.message==='Minecraft запущен')$('#cancel-launch').hidden=true;});await window.__TAURI__.event.listen('state-changed',async()=>{await sync();render();});await window.__TAURI__.event.listen('close-blocked',()=>toast('Дождитесь завершения игры или установки.'));
  invoke('versions').then(v=>{releaseVersions=v.filter(v=>!/^1\.(?:[0-9]|1[0-2])(?:\.|$)/.test(v));if(page==='play')render();}).catch(e=>toast('Не удалось обновить список версий: '+e));
 }}catch(e){failure(e);render();}
}
window.KyroServers.configure({state:()=>state,invoke,native,render,sync,toast,error:failure,run:runOperation});
window.KyroPickers.configure({current:()=>({...s(),busy:state.busy}),releases:()=>releaseVersions,load:()=>invoke('version_library'),prepare:()=>runOperation('Подготовка версии…',()=>invoke('verify_files')),select:async(version,loader)=>{if(state.busy)throw new Error('Дождитесь завершения текущей операции');s().instance_id='';s().loader_version='';s().version=version;s().loader=loader;if(!await persist())throw new Error('Не удалось сохранить выбор');await sync();catalogData=null;catalogOffset=0;render();if(page==='catalog')loadCatalog();}});
window.KyroFeatures.configure({state:()=>state,invoke,persist,render,error:failure,run:runOperation,changed:async()=>{await sync();render();},openMods:()=>{catalogKind='installed';catalogData=null;navigate('catalog');}});
window.KyroAssist.configure({state:()=>state,invoke,render,error:failure,run:runOperation,navigate,changed:async()=>{await sync();render();},openJava:()=>{settingsTab='java';navigate('settings');},openMods:()=>{catalogKind='installed';catalogData=null;navigate('catalog');},openCompatibility:()=>{catalogKind='compatibility';catalogData=null;navigate('catalog');}});
boot();

let fpsRequest=0;
async function openFpsBoost(){
 const request=++fpsRequest;
 let dialog=$('#fps-dialog');
 if(!dialog){dialog=document.createElement('dialog');dialog.id='fps-dialog';dialog.setAttribute('aria-labelledby','fps-title');document.body.append(dialog);dialog.addEventListener('close',()=>{fpsRequest++;});}
 const header=label=>`<header class="fps-heading"><div><div class="eyebrow">ОПТИМИЗАЦИЯ</div><h2 id="fps-title">FPS-буст</h2><span class="fps-profile">${esc(label)}</span></div><button class="fps-dismiss" aria-label="Закрыть FPS-буст">×</button></header><p class="fps-intro">Моды для плавной игры. Твои настройки графики сохранятся.</p>`;
 const close=()=>{$('.fps-dismiss',dialog).onclick=()=>dialog.close();const button=$('#fps-close');if(button)button.onclick=()=>dialog.close();};
 dialog.innerHTML=header(s().version+' · '+s().loader)+`<div class="fps-loading" role="status"><span class="fps-spinner" aria-hidden="true"></span><div><strong>Подбираем моды</strong><span>Проверяем совместимость и зависимости…</span></div></div><div class="fps-skeleton" aria-hidden="true"><i></i><i></i><i></i></div><footer class="fps-actions"><span>Только совместимые версии</span><button id="fps-close">Отмена</button></footer>`;
 close();if(!dialog.open)dialog.showModal();
 try{
  const plan=await invoke('fps_plan');if(!dialog.open||request!==fpsRequest)return;
  dialog.innerHTML=header(plan.label)+`<div class="fps-list-heading"><span>ПОДОБРАННЫЕ МОДЫ</span><span>К установке: ${plan.items.length}</span></div><div class="fps-rows" tabindex="0" aria-label="Моды оптимизации">${plan.rows.map(row=>{const install=row.status==='Будет установлен',present=row.status.startsWith('Уже установлен');return `<article class="fps-mod"><span class="fps-mod-mark ${install?'pending':present?'installed':''}" aria-hidden="true">${install?'+':present?'✓':'−'}</span><div><strong>${esc(row.name)}</strong><p>${esc(row.description)}</p><span class="fps-mod-status ${install?'pending':''}">${esc(row.status)}</span></div></article>`;}).join('')}</div><footer class="fps-actions"><span>${plan.items.length?'С зависимостями':'Всё готово'}</span><div><button id="fps-close">Закрыть</button><button id="fps-install" class="primary" ${!plan.items.length?'disabled':''}>Установить · ${plan.items.length}</button></div></footer>`;
  close();$('#fps-install').onclick=async()=>{dialog.close();await runOperation('Установка FPS-буста…',()=>invoke('fps_install',{plan}));window.KyroFeatures.invalidateUpdates();};
 }catch(e){
  if(!dialog.open||request!==fpsRequest)return;
  dialog.innerHTML=header(s().version+' · '+s().loader)+`<div class="fps-error" role="alert"><strong>Не удалось подобрать моды</strong><p>${esc(String(e))}</p></div><footer class="fps-actions"><button id="fps-close">Закрыть</button><button id="fps-retry" class="primary">Повторить</button></footer>`;
  close();$('#fps-retry').onclick=openFpsBoost;
 }
}
