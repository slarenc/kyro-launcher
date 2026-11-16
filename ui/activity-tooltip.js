// Shared pointer and keyboard tooltip, outside the scrollable heatmap.
(()=>{
 let active;
 function hide(){document.getElementById('activity-tooltip')?.remove();if(active)active.removeAttribute('aria-describedby');active=null;}
 function show(cell){
  hide();active=cell;
  const tip=document.createElement('div');tip.id='activity-tooltip';tip.className='activity-tooltip';tip.role='tooltip';
  const date=document.createElement('span');date.className='tooltip-date';date.textContent=cell.dataset.activityDate;
  const time=document.createElement('strong');time.textContent=cell.dataset.activityTime;
  const meta=document.createElement('div');meta.className='tooltip-meta';
  const dot=document.createElement('span');dot.className='tooltip-dot';meta.append(dot);
  const count=Number(cell.dataset.activityCount);meta.append(count?`Сессий: ${count}`:cell.classList.contains('future')?'Этот день ещё впереди':'В этот день не играли');
  tip.append(date,time,meta);document.body.append(tip);cell.setAttribute('aria-describedby',tip.id);
  const rect=cell.getBoundingClientRect(),size=tip.getBoundingClientRect();
  tip.style.left=Math.max(12,Math.min(innerWidth-size.width-12,rect.left+rect.width/2-size.width/2))+'px';
  tip.style.top=(rect.top-size.height-12>=8?rect.top-size.height-12:Math.min(innerHeight-size.height-8,rect.bottom+12))+'px';
 }
 document.addEventListener('pointerover',e=>{const cell=e.target.closest?.('[data-activity-date]');if(cell&&cell!==active)show(cell);});
 document.addEventListener('pointerout',e=>{if(e.target===active&&!active.contains(e.relatedTarget))hide();});
 document.addEventListener('focusin',e=>{if(e.target.matches('[data-activity-date]'))show(e.target);});
 document.addEventListener('focusout',e=>{if(e.target===active)hide();});
 document.addEventListener('keydown',e=>{if(e.key==='Escape')hide();});
 document.addEventListener('scroll',hide,true);window.addEventListener('resize',hide);
})();
