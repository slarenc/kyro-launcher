package ru.kyro.hud;

import com.google.gson.*;
import java.awt.*;
import java.awt.image.BufferedImage;
import java.nio.*;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.util.*;
import java.util.concurrent.*;
import javax.imageio.ImageIO;

/** Local account dashboard. Game access and GL calls stay on the render thread. */
final class AccountMenu {
    static volatile boolean open;
    static final String session=System.getProperty("kyro.accounts.session",UUID.randomUUID().toString());
    static final String rootName=System.getProperty("kyro.accounts.root","");
    static final Path root=Paths.get(rootName),own=root.resolve(session);
    static final ExecutorService io=Executors.newSingleThreadExecutor(r->{Thread t=new Thread(r,"KYRO accounts IPC");t.setDaemon(true);return t;});
    static final java.util.concurrent.atomic.AtomicBoolean working=new java.util.concurrent.atomic.AtomicBoolean();
    static volatile java.util.List<Row> rows=Collections.emptyList();
    static volatile String status="",request="";
    static String server="",nickname="";static int editing,page;static long lastSample,lastCapture;
    static volatile boolean launcherOnline;
    static final class Row {JsonObject account,state;BufferedImage image;String id;boolean alive,online;}
    static String metric(JsonObject o,String key){String value=str(o,key);return value.isEmpty()?"—":value;}
    static String str(JsonObject o,String key){return o!=null&&o.has(key)&&!o.get(key).isJsonNull()?o.get(key).getAsString():"";}
    static long number(JsonObject o,String key){try{return o.get(key).getAsLong();}catch(Exception e){return 0;}}
    static JsonObject read(Path p)throws Exception {if(Files.size(p)>1024*1024)throw new java.io.IOException("IPC payload too large");return AgentHooks.JSON.fromJson(new String(Files.readAllBytes(p),StandardCharsets.UTF_8),JsonObject.class);}
    static void write(Path p,JsonObject value)throws Exception {Files.createDirectories(p.getParent());Path tmp=p.resolveSibling(p.getFileName()+".tmp");Files.write(tmp,AgentHooks.JSON.toJson(value).getBytes(StandardCharsets.UTF_8));Files.move(tmp,p,StandardCopyOption.REPLACE_EXISTING);}
    static boolean validId(String s){return s.matches("[A-Za-z0-9-]{1,120}");}
    static Object call(Object o,String name){return GameAccess.call(o,name);}
    static String text(Object o){return GameAccess.text(o);}
    static JsonObject sample(){
        AgentGame game=AgentHooks.game;Object player=game.localPlayer,client=game.clientObject();JsonObject s=new JsonObject();
        s.addProperty("time",System.currentTimeMillis());s.addProperty("uuid",System.getProperty("kyro.hud.uuid",""));s.addProperty("name",System.getProperty("kyro.hud.player","Minecraft"));s.addProperty("online",game.world);s.addProperty("ping",game.ping);
        for(String[] metric:new String[][]{{"health","getHealth"},{"armor","getArmorValue"}}){Object v=call(player,metric[1]);if(v instanceof Number)s.addProperty(metric[0],(Number)v);}
        Object level=GameAccess.field(client,"level");Object dimension=call(level,"dimension");Object location=call(dimension,"location");if(location==null)location=call(dimension,"identifier");s.addProperty("world",location==null?(game.world?"Мир":"Главное меню"):location.toString());
        Object mode=GameAccess.field(client,"gameMode");s.addProperty("mode",text(call(call(mode,"getPlayerMode"),"getLongDisplayName")));
        Object currentServer=call(client,"getCurrentServer");Object ip=GameAccess.field(currentServer,"ip");s.addProperty("server",ip instanceof String?(String)ip:"");
        JsonArray inventory=new JsonArray();Object inv=call(player,"getInventory");for(int i=0;i<36;i++){Object stack=GameAccess.call(inv,"getItem",i);String label=text(call(stack,"getHoverName"));Object count=call(stack,"getCount");inventory.add(label.isEmpty()||Boolean.TRUE.equals(call(stack,"isEmpty"))?"":label+" ×"+(count instanceof Number?count:"?"));}s.add("inventory",inventory);return s;
    }
    static void tick(){
        if(rootName.isEmpty())return;
        long now=System.nanoTime();if(now-lastSample<500_000_000L)return;lastSample=now;
        try {
            Path command=own.resolve("command.json");if(Files.exists(command)){JsonObject c=read(command);Files.deleteIfExists(command);String action=str(c,"action");
                if(action.equals("control")){AgentHooks.window("glfwShowWindow",new Class<?>[]{long.class},AgentHooks.activeWindow);AgentHooks.window("glfwRestoreWindow",new Class<?>[]{long.class},AgentHooks.activeWindow);AgentHooks.window("glfwFocusWindow",new Class<?>[]{long.class},AgentHooks.activeWindow);if(AgentHooks.editor)AgentHooks.setEditor(false);}
                if(action.equals("disconnect")){
                    // Closing via Minecraft's normal window-close path saves local worlds.
                    AgentHooks.window("glfwSetWindowShouldClose",new Class<?>[]{long.class,boolean.class},AgentHooks.activeWindow,true);
                }
                JsonObject ack=new JsonObject();ack.addProperty("action",action);ack.addProperty("time",System.currentTimeMillis());write(own.resolve("ack.json"),ack);
            }
        }catch(Exception e){status="Ошибка команды: "+e.getClass().getSimpleName();AgentHooks.log("Account command failed",e);}
        if(!working.compareAndSet(false,true))return;
        final JsonObject state;final BufferedImage frame;try{state=sample();frame=AgentHooks.game.world?capture():null;}catch(Throwable e){working.set(false);AgentHooks.log("Account sample failed",e);return;}
        io.execute(()->{try{write(own.resolve("state.json"),state);if(frame!=null){Path tmp=own.resolve("frame.tmp");ImageIO.write(frame,"jpg",tmp.toFile());Files.move(tmp,own.resolve("frame.jpg"),StandardCopyOption.REPLACE_EXISTING);}refresh();}catch(Exception e){status="Связь недоступна: "+e.getClass().getSimpleName();}finally{working.set(false);}});
    }
    static BufferedImage capture(){
        long now=System.nanoTime();if(now-lastCapture<1_000_000_000L||AgentHooks.gl==null)return null;lastCapture=now;
        int w=AgentHooks.pixelWidth,h=AgentHooks.pixelHeight;if(w<=0||h<=0||((long)w*h)>16777216)return null;
        try{
            AgentGL gl=AgentHooks.gl;ByteBuffer pixels=ByteBuffer.allocateDirect(w*h*4);
            int old=((Number)gl.ints("GL11","glGetInteger",35053)).intValue();int alignment=((Number)gl.ints("GL11","glGetInteger",3333)).intValue();int row=((Number)gl.ints("GL11","glGetInteger",3330)).intValue();int skipRows=((Number)gl.ints("GL11","glGetInteger",3331)).intValue();int skipPixels=((Number)gl.ints("GL11","glGetInteger",3332)).intValue();
            gl.ints("GL15","glBindBuffer",35051,0);
            try{gl.ints("GL11","glPixelStorei",3333,1);gl.ints("GL11","glPixelStorei",3330,0);gl.ints("GL11","glPixelStorei",3331,0);gl.ints("GL11","glPixelStorei",3332,0);gl.call("GL11","glReadPixels",new Class<?>[]{int.class,int.class,int.class,int.class,int.class,int.class,ByteBuffer.class},0,0,w,h,6408,5121,pixels);}
            finally{gl.ints("GL15","glBindBuffer",35051,old);gl.ints("GL11","glPixelStorei",3333,alignment);gl.ints("GL11","glPixelStorei",3330,row);gl.ints("GL11","glPixelStorei",3331,skipRows);gl.ints("GL11","glPixelStorei",3332,skipPixels);}
            BufferedImage result=new BufferedImage(320,180,BufferedImage.TYPE_INT_RGB);for(int y=0;y<180;y++)for(int x=0;x<320;x++){int at=(((h-1-y*h/180)*w)+x*w/320)*4;result.setRGB(x,y,(pixels.get(at)&255)<<16|(pixels.get(at+1)&255)<<8|(pixels.get(at+2)&255));}return result;
        }catch(Exception e){AgentHooks.log("Account capture failed",e);return null;}
    }
    static void refresh()throws Exception{
        long now=System.currentTimeMillis();JsonObject catalog=read(root.resolve("catalog.json"));launcherOnline=now-number(catalog,"time")<10000;
        java.util.List<Row> result=new ArrayList<>();Map<String,Row> active=new HashMap<>();
        try(DirectoryStream<Path> dirs=Files.newDirectoryStream(root)){for(Path dir:dirs){if(!Files.isDirectory(dir)||!validId(dir.getFileName().toString()))continue;try{JsonObject s=read(dir.resolve("state.json"));if(now-number(s,"time")>8000)continue;Row r=new Row();r.state=s;r.id=dir.getFileName().toString();r.alive=true;r.online=s.has("online")&&s.get("online").getAsBoolean();if(r.online&&Files.exists(dir.resolve("frame.jpg")))r.image=ImageIO.read(dir.resolve("frame.jpg").toFile());active.put(str(s,"uuid").replace("-",""),r);}catch(Exception ignored){}}}
        for(JsonElement item:catalog.getAsJsonArray("accounts")){JsonObject a=item.getAsJsonObject();Row r=active.remove(str(a,"uuid").replace("-",""));if(r==null)r=new Row();r.account=a;result.add(r);}
        for(Row r:active.values()){r.account=new JsonObject();r.account.addProperty("name",str(r.state,"name"));result.add(r);}rows=Collections.unmodifiableList(result);
        if(!launcherOnline&&request.isEmpty())status="Лаунчер недоступен. Перезапусти игру из KYRO.";
        if(!request.isEmpty()&&Files.exists(root.resolve(request))){status=str(read(root.resolve(request)),"message");Files.deleteIfExists(root.resolve(request));request="";}
    }
    static void send(Row row,String action){try{if(row==null||!row.alive||!validId(row.id))return;JsonObject c=new JsonObject();c.addProperty("action",action);write(root.resolve(row.id).resolve("command.json"),c);status=action.equals("disconnect")?"Завершение клиента с сохранением мира…":"Переключение в окно клиента…";}catch(Exception e){status="Не удалось отправить команду";}}
    static void request(String action,Row row){if(!launcherOnline){status="Запусти лаунчер KYRO";return;}if(!request.isEmpty()){status="Дождись завершения запуска";return;}try{JsonObject r=new JsonObject();r.addProperty("action",action);r.addProperty("key",row==null?"":str(row.account,"key"));r.addProperty("server",server.trim());r.addProperty("name",nickname.trim());request=UUID.randomUUID()+".json";write(root.resolve("requests").resolve(request),r);status="Выполняется…";}catch(Exception e){request="";status="Не удалось отправить запрос";}}
    static void enter(){open=true;page=0;editing=0;String address=str(sample(),"server");if(!address.isEmpty())server=address;}
    static int width(){return 640;}static int height(){return 340;}
    static void label(Graphics2D g,String s,int x,int y,int w,float size,Color color){PanelPainter.text(g,s,x,y,w,size,color,0,false);}
    static void button(Graphics2D g,String s,int x,int y,int w,boolean enabled){g.setColor(PanelPainter.ink(enabled?25:9));g.fillRoundRect(x,y,w,24,7,7);label(g,s,x+7,y+6,w-14,9,enabled?PanelPainter.ACCENT:PanelPainter.QUIET);}
    static BufferedImage paint(){
        BufferedImage image=PanelPainter.canvas(width(),height());Graphics2D g=PanelPainter.graphics(image);PanelPainter.glass(g,width(),height(),14,false);java.util.List<Row> list=rows;int online=0;for(Row r:list)if(r.online)online++;
        label(g,"KYRO / Аккаунты",14,10,300,14,PanelPainter.ACCENT);label(g,online+" из "+list.size()+" Online · кадры 1 FPS",370,14,255,10,PanelPainter.QUIET);
        button(g,"‹ Панели",14,36,90,true);button(g,(editing==1?"▏ ":"")+"Сервер: "+(server.isEmpty()?"главное меню":server),112,36,300,true);button(g,(editing==2?"▏ ":"")+"Ник: "+nickname,420,36,130,true);button(g,"+ Добавить",558,36,68,launcherOnline);
        int maxPage=Math.max(0,(list.size()-1)/2);page=Math.min(page,maxPage);
        for(int i=0;i<2;i++){int index=page*2+i;if(index>=list.size())break;Row r=list.get(index);int x=14+i*310;g.setColor(PanelPainter.ink(10));g.fillRoundRect(x,70,302,220,9,9);label(g,str(r.account,"name"),x+10,78,180,12,PanelPainter.WHITE);label(g,r.online?str(r.state,"ping"):r.alive?"В меню":"Offline",x+200,80,92,9,PanelPainter.QUIET);
            if(r.online&&r.image!=null)g.drawImage(r.image,x+8,100,146,82,null);else label(g,r.alive?"Ожидание мира":"Клиент не запущен",x+14,134,140,9,PanelPainter.QUIET);
            label(g,str(r.state,"world"),x+164,102,130,9,PanelPainter.WHITE);label(g,str(r.state,"mode"),x+164,119,130,9,PanelPainter.QUIET);label(g,"Здоровье: "+(r.online?metric(r.state,"health"):"—"),x+164,141,130,9,PanelPainter.WHITE);label(g,"Броня: "+(r.online?metric(r.state,"armor"):"—"),x+164,158,130,9,PanelPainter.WHITE);
            JsonArray items=r.online&&r.state.has("inventory")?r.state.getAsJsonArray("inventory"):new JsonArray();for(int slot=0;slot<36;slot++){int sx=x+8+(slot%9)*32,sy=190+(slot/9)*14;g.setColor(PanelPainter.ink(15));g.fillRect(sx,sy,30,12);if(slot<items.size())label(g,items.get(slot).getAsString(),sx+2,sy+2,27,5,PanelPainter.WHITE);}
            button(g,"Control",x+8,254,90,r.alive);button(g,"Play as",x+105,254,90,r.alive||launcherOnline);button(g,"Disconnect",x+202,254,90,r.alive);
        }
        if(list.isEmpty())label(g,rootName.isEmpty()?"Этот клиент запущен без канала KYRO. Перезапусти игру из новой сборки.":"Добавь офлайн-ник выше или Microsoft-аккаунт в лаунчере.",20,150,600,11,PanelPainter.QUIET);
        label(g,status.isEmpty()?"Control / Play as — окно клиента · Disconnect — закрыть клиент с сохранением":status,14,299,500,9,PanelPainter.QUIET);button(g,"‹",526,298,28,page>0);label(g,(page+1)+" / "+(maxPage+1),560,305,36,8,PanelPainter.QUIET);button(g,"›",598,298,28,page<maxPage);g.dispose();return image;
    }
    static void click(double x,double y)throws Exception{
        editing=0;if(y>=36&&y<60){if(x<104){open=false;return;}if(x<412){editing=1;return;}if(x<550){editing=2;return;}request("add",null);return;}
        if(y>=298){if(x>=598)page=Math.min(page+1,Math.max(0,(rows.size()-1)/2));else if(x>=526&&x<554)page=Math.max(0,page-1);return;}
        if(y>=254&&y<278){int col=x>=324?1:0,index=page*2+col;if(index>=rows.size())return;Row r=rows.get(index);double local=x-(14+col*310);if(local>=8&&local<98)send(r,"control");else if(local>=105&&local<195){if(r.alive)send(r,"control");else request("launch",r);}else if(local>=202&&local<292)send(r,"disconnect");}
    }
    static void character(int point){if(editing==0||Character.isISOControl(point))return;String c=new String(Character.toChars(point));if(editing==1&&server.length()<255)server+=c;if(editing==2&&nickname.length()<16&&c.matches("[A-Za-z0-9_]"))nickname+=c;}
    static void key(int key,int action,int mods){if(action==0)return;if(key==259){if(editing==1&&!server.isEmpty())server=server.substring(0,server.length()-1);if(editing==2&&!nickname.isEmpty())nickname=nickname.substring(0,nickname.length()-1);}if(key==86&&(mods&2)!=0&&editing!=0)try{Object clip=AgentHooks.window("glfwGetClipboardString",new Class<?>[]{long.class},AgentHooks.activeWindow);if(clip instanceof String)for(int point:((String)clip).codePoints().limit(255).toArray())character(point);}catch(Exception ignored){}if(key==257)editing=0;if(key==65&&(mods&2)!=0){if(editing==1)server="";if(editing==2)nickname="";}}
}
