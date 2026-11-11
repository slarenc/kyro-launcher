import java.io.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import java.util.jar.*;

/** Relocate private dependencies without adding them to the game's Gson/ASM namespaces. */
public final class RelocateJar {
    static String rename(String s,boolean runtime){String result=s.replace("org/objectweb/asm","ru/kyro/internal/asm").replace("org.objectweb.asm","ru.kyro.internal.asm").replace("com/google/gson","ru/kyro/internal/gson").replace("com.google.gson","ru.kyro.internal.gson");return runtime?result.replace("ru/kyro/hud/","ru/kyro/builtin/").replace("ru.kyro.hud.","ru.kyro.builtin."):result;}
    static byte[] relocate(byte[] bytes,boolean runtime)throws IOException{
        DataInputStream in=new DataInputStream(new ByteArrayInputStream(bytes));ByteArrayOutputStream output=new ByteArrayOutputStream();DataOutputStream out=new DataOutputStream(output);
        out.writeInt(in.readInt());out.writeShort(in.readUnsignedShort());out.writeShort(in.readUnsignedShort());int count=in.readUnsignedShort();out.writeShort(count);
        for(int i=1;i<count;i++){
            int tag=in.readUnsignedByte();out.writeByte(tag);int n;
            switch(tag){
                case 1: int len=in.readUnsignedShort();byte[] data=new byte[len];in.readFully(data);byte[] changed=rename(new String(data,StandardCharsets.ISO_8859_1),runtime).getBytes(StandardCharsets.ISO_8859_1);out.writeShort(changed.length);out.write(changed);continue;
                case 3:case 4:case 9:case 10:case 11:case 12:case 17:case 18:n=4;break;
                case 5:case 6:n=8;i++;break;
                case 7:case 8:case 16:case 19:case 20:n=2;break;
                case 15:n=3;break;default:throw new IOException("Unknown constant tag "+tag);
            }
            for(int j=0;j<n;j++)out.writeByte(in.readUnsignedByte());
        }
        byte[] tail=new byte[8192];int read;while((read=in.read(tail))>=0)out.write(tail,0,read);return output.toByteArray();
    }
    public static void main(String[]args)throws Exception{
      for(int part=1;part<3;part++){
        try(JarFile input=new JarFile(args[0]);JarOutputStream out=new JarOutputStream(Files.newOutputStream(Paths.get(args[part])),input.getManifest())){
            Set<String> seen=new HashSet<>();seen.add("META-INF/MANIFEST.MF");Enumeration<JarEntry> entries=input.entries();
            while(entries.hasMoreElements()){
                JarEntry entry=entries.nextElement();String original=entry.getName();
                boolean launcher=original.startsWith("ru/kyro/hud/HudAgent")||original.startsWith("ru/kyro/hud/HudTransform")||original.startsWith("ru/kyro/hud/ModuleTransform")||original.startsWith("org/objectweb/asm/");
                if((part==1)!=launcher)continue;
                String name=rename(original,part==2);if(entry.isDirectory()||!seen.add(name))continue;
                ByteArrayOutputStream buffer=new ByteArrayOutputStream();try(InputStream stream=input.getInputStream(entry)){byte[] block=new byte[8192];int n;while((n=stream.read(block))>=0)buffer.write(block,0,n);}
                JarEntry target=new JarEntry(name);target.setTime(0);out.putNextEntry(target);byte[] bytes=buffer.toByteArray();out.write(name.endsWith(".class")?relocate(bytes,part==2):bytes);out.closeEntry();
            }
        }
      }
    }
}
