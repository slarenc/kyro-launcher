const fs=require('fs');
const path=require('node:path');
const sharp=require('C:/Users/User/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp');
(async()=>{
 const root=path.resolve(__dirname,'..');
 const source=path.join(root,'ui/kyro-avatar.png');
 const icons=path.join(root,'src-tauri/icons');
 fs.mkdirSync(icons,{recursive:true});
 const png=await sharp(source).resize(256,256,{fit:'contain'}).png().toBuffer();
 fs.writeFileSync(path.join(icons,'icon.png'),png);
 const sizes=[16,24,32,48,64,128,256];
 const images=await Promise.all(sizes.map(size=>sharp(source).resize(size,size,{fit:'contain'}).png().toBuffer()));
 const header=Buffer.alloc(6+16*sizes.length);
 header.writeUInt16LE(1,2);header.writeUInt16LE(sizes.length,4);
 let offset=header.length;
 images.forEach((image,i)=>{
  const entry=6+16*i,size=sizes[i];
  header[entry]=size===256?0:size;header[entry+1]=size===256?0:size;
  header.writeUInt16LE(1,entry+4);header.writeUInt16LE(32,entry+6);
  header.writeUInt32LE(image.length,entry+8);header.writeUInt32LE(offset,entry+12);
  offset+=image.length;
 });
 fs.writeFileSync(path.join(icons,'icon.ico'),Buffer.concat([header,...images]));
 console.log('Updated Windows icons from ui/kyro-avatar.png');
})().catch(e=>{console.error(e);process.exit(1)});
