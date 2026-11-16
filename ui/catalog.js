// Each page consumes 12 entries from each provider. Keep provider IDs unchanged
// so installation and dependencies are routed to the correct service.
window.KyroCatalog={async search(invoke,args){
 const requests=[['Modrinth','search_modrinth']];
 if(args.kind!=='modpack'&&args.offset<9996)requests.push(['CurseForge','search_curseforge']);
 const results=await Promise.allSettled(requests.map(([,command])=>invoke(command,args)));
 const hits=[],warnings=[];let total=0,pages=1,success=false;
 results.forEach((result,index)=>{
  if(result.status==='rejected'){warnings.push(`${requests[index][0]}: ${result.reason?.message||result.reason}`);return;}
  success=true;const data=result.value;
  hits.push(...(data.hits||[]));total+=data.total_hits||0;
  pages=Math.max(pages,Math.ceil((data.total_hits||0)/12));
 });
 if(!success)throw new Error(warnings.join(' · '));
 hits.sort((a,b)=>(b.downloads||0)-(a.downloads||0));
 return {hits,total_hits:total,page_count:pages,warning:warnings.length?'Часть каталога недоступна. '+warnings.join(' · '):''};
}};
