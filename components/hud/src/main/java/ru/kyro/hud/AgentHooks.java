package ru.kyro.hud;

import com.google.gson.*;
import java.awt.*;
import java.awt.image.BufferedImage;
import java.lang.reflect.*;
import java.nio.*;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.time.*;
import java.time.format.DateTimeFormatter;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicBoolean;

/** Version-independent runtime, appended by KYRO to the launched JVM only. */
public final class AgentHooks {
    static final Gson JSON=new GsonBuilder().setPrettyPrinting().create();
    static Path config;static PanelLayout layout;static MediaBridge media;static AgentGame game;static BadgePresence presence;
    static AgentGL gl;static Class<?> glfw;static long activeWindow;static boolean ready,failed,mediaStarted;
    static final Map<String,Method> windowMethods=new HashMap<>();
    static final IntBuffer wx=ByteBuffer.allocateDirect(4).order(ByteOrder.nativeOrder()).asIntBuffer(),wy=ByteBuffer.allocateDirect(4).order(ByteOrder.nativeOrder()).asIntBuffer();
    static final IntBuffer px=ByteBuffer.allocateDirect(4).order(ByteOrder.nativeOrder()).asIntBuffer(),py=ByteBuffer.allocateDirect(4).order(ByteOrder.nativeOrder()).asIntBuffer();
    static final DoubleBuffer mx=ByteBuffer.allocateDirect(8).order(ByteOrder.nativeOrder()).asDoubleBuffer(),my=ByteBuffer.allocateDirect(8).order(ByteOrder.nativeOrder()).asDoubleBuffer();
    static int pixelWidth,pixelHeight,guiWidth,guiHeight;static float scale=1;
    static double mouseX,mouseY,musicX,musicY,expand,musicHeight=48,seek=-1,dragX,dragY,holdPosition;
    static int dragging=-1,holding;static long holdAt,previewUntil,lastFrame,lastPaint,statsAt,fpsAt;static int fpsFrames,fps;
    static String clock="",ping="— ms",controlKey="";static boolean hot,remaining,wasLocked;
    public static volatile boolean editor;
    static boolean skyEditor;
    static int skyColorEdit=-1;static String skyHex="";static boolean skySelectAll,skyHexError;
    static int editorHeight(){return skyEditor?274:278;}
    static final AtomicBoolean painting=new AtomicBoolean();static volatile Frame pending;static Frame shown;
    static final ExecutorService painter=Executors.newSingleThreadExecutor(r->{Thread t=new Thread(r,"KYRO built-in HUD");t.setDaemon(true);return t;});
    static final Set<String> errors=Collections.synchronizedSet(new HashSet<String>());
    static long previewStart;static int captures;static boolean previewHidden;
    static final AgentCooldowns cooldownReader=new AgentCooldowns();
    static long cooldownAt;static java.util.List<AgentCooldowns.Entry> cooldownRows=Collections.emptyList();
    static final class Frame {final BufferedImage info,music,editor,cooldowns;Frame(BufferedImage i,BufferedImage m,BufferedImage e,BufferedImage c){info=i;music=m;editor=e;cooldowns=c;}}
    public static void boot(){
        config=Paths.get(System.getProperty("kyro.hud.config","config")).toAbsolutePath();
        try{layout=PanelLayout.load(config.resolve("kyro-panels.json"),JSON);}catch(Exception error){layout=new PanelLayout();log("Settings could not be read",error);}
        media=new MediaBridge(Paths.get(System.getProperty("kyro.hud.media",config.resolve("kyro-media.exe").toString())));media.enabled=layout.music;game=new AgentGame();presence=new BadgePresence();
        Runtime.getRuntime().addShutdownHook(new Thread(()->{presence.close();media.close();painter.shutdownNow();save();},"KYRO HUD shutdown"));
    }
    public static void log(String message,Throwable error){if(!errors.add(message))return;String text="[KYRO HUD] "+message+": "+error;System.err.println(text);try{Files.createDirectories(config);Files.write(config.resolve("kyro-hud.log"),(text+System.lineSeparator()).getBytes(StandardCharsets.UTF_8),StandardOpenOption.CREATE,StandardOpenOption.APPEND);}catch(Exception ignored){}}
    static synchronized void save(){try{layout.save(config.resolve("kyro-panels.json"),JSON);}catch(Exception error){log("Settings could not be saved",error);}}
    public static Object wrap(long window,Object callback,String kind){return AgentInput.wrap(window,callback,kind);}
    static Object window(String name,Class<?>[] signature,Object...args)throws Exception{Method m=windowMethods.get(name);if(m==null){m=glfw.getMethod(name,signature);windowMethods.put(name,m);}return m.invoke(null,args);}
    static int cursor()throws Exception{return ((Number)window("glfwGetInputMode",new Class<?>[]{long.class,int.class},activeWindow,208897)).intValue();}
    static void cursor(int mode)throws Exception{window("glfwSetInputMode",new Class<?>[]{long.class,int.class,int.class},activeWindow,208897,mode);}
    static void pointer()throws Exception{window("glfwGetCursorPos",new Class<?>[]{long.class,DoubleBuffer.class,DoubleBuffer.class},activeWindow,mx,my);mouseX=mx.get(0)*pixelWidth/Math.max(1,wx.get(0))/scale;mouseY=my.get(0)*pixelHeight/Math.max(1,wy.get(0))/scale;}
    static int x(double fraction,int size){return PanelLayout.coordinate(fraction,guiWidth,size);}static int y(double fraction,int size){return PanelLayout.coordinate(fraction,guiHeight,size);}
    static boolean inside(double a,double b,double x,double y,double w,double h){return a>=x&&a<x+w&&b>=y&&b<y+h;}
    public static void frame(long window,Class<?> glfwClass){
        if(failed)return;
        try{
            glfw=glfwClass;
            if(gl!=null&&activeWindow!=window)return;
            if(((Number)window("glfwGetCurrentContext",new Class<?>[0])).longValue()!=window)return;
            // Starting child processes from premain can race signed-JAR verification
            // with the JDK logging backend. Wait until the game has a render context.
            if(!mediaStarted){mediaStarted=true;media.start();presence.start();}
            activeWindow=window;long now=System.nanoTime();fpsFrames++;
            if(fpsAt==0)fpsAt=now;if(now-fpsAt>=500_000_000L){fps=(int)Math.round(fpsFrames*1e9/(now-fpsAt));fpsFrames=0;fpsAt=now;}
            if(now-statsAt>=500_000_000L){statsAt=now;ClassLoader context=Thread.currentThread().getContextClassLoader();game.update(context==null?glfwClass.getClassLoader():context);clock=LocalTime.now().format(DateTimeFormatter.ofPattern("HH:mm"));ping=game.ping;}
            boolean preview=Boolean.getBoolean("kyro.hud.agentPreview");
            if(preview&&Boolean.getBoolean("kyro.hud.previewHidden")&&!previewHidden){window("glfwHideWindow",new Class<?>[]{long.class},window);previewHidden=true;}
            if(preview&&Boolean.getBoolean("kyro.hud.requireWorld")&&!game.world)return;
            if(!game.world&&!preview){cooldownReader.sample(null);CooldownHooks.reset();cooldownRows=Collections.emptyList();ready=false;if(editor)setEditor(false);return;}
            if(now-cooldownAt>=50_000_000L){cooldownAt=now;cooldownRows=CooldownHooks.merge(game.localPlayer,cooldownReader.sample(game.localPlayer));}
            window("glfwGetFramebufferSize",new Class<?>[]{long.class,IntBuffer.class,IntBuffer.class},window,px,py);window("glfwGetWindowSize",new Class<?>[]{long.class,IntBuffer.class,IntBuffer.class},window,wx,wy);
            pixelWidth=px.get(0);pixelHeight=py.get(0);if(pixelWidth<=0||pixelHeight<=0)return;
            scale=(float)Math.min(Math.max(1,Math.floor(pixelHeight/360.0)),Math.min(pixelWidth/340.0,pixelHeight/340.0));guiWidth=(int)(pixelWidth/scale);guiHeight=(int)(pixelHeight/scale);pointer();ready=true;
            if(!editor&&game.hidden())return;
            if(!editor&&!layout.info&&!layout.music&&(!layout.cooldowns||cooldownRows.isEmpty())){if(layout.customSky&&now-lastPaint>=1_000_000_000L&&painting.compareAndSet(false,true)){lastPaint=now;painter.execute(()->{try{PanelPainter.refreshAppearance();}finally{painting.set(false);}});}return;}
            boolean free=cursor()!=212995;
            if(editor&&!free){cursor(212993);free=true;}
            PanelPainter.Track track=media.snapshot();double musicScale=layout.musicScale();hot=free&&inside(mouseX,mouseY,musicX,musicY,272*musicScale,musicHeight);
            if(preview){if(previewStart==0)previewStart=now;double elapsed=(now-previewStart)/1e9;if(elapsed>6)hot=true;if(elapsed>10&&!editor)setEditor(true);}
            double target=track.available()&&((layout.expandOnHover&&hot)||layout.announceTracks&&media.trackSeconds()<3.2||holding!=0||seek>=0&&free)?1:0;
            double dt=lastFrame==0?1/60.0:Math.min(.1,(now-lastFrame)/1e9);lastFrame=now;expand+=(target-expand)*(1-Math.exp(-dt*12));if(Math.abs(target-expand)<.002)expand=target;
            musicHeight=(48+68*expand)*musicScale;musicX=x(layout.musicX,layout.musicWidth());musicY=Math.min(y(layout.musicY,layout.musicHeight()),Math.max(0,guiHeight-musicHeight-8));
            if(dragging>=0){layout.move(dragging,mouseX,mouseY,dragX,dragY,guiWidth,guiHeight);musicX=x(layout.musicX,layout.musicWidth());musicY=Math.min(y(layout.musicY,layout.musicHeight()),Math.max(0,guiHeight-musicHeight-8));}
            if((holding!=0||seek>=0)&&!controlKey.equals(track.source+"\0"+track.title+"\0"+track.artist)){holding=0;seek=-1;previewUntil=0;}
            if(seek>=0&&previewUntil==Long.MAX_VALUE)seek=PanelPainter.clamp(((mouseX-musicX)/musicScale-12)/248);
            if(holding!=0&&now-holdAt>380_000_000L&&track.canSeek&&track.duration>0){double elapsed=(now-holdAt-380_000_000L)/1e9;seek=PanelPainter.clamp((holdPosition+holding*elapsed*(4+Math.min(28,elapsed*4)))/track.duration);}
            if(seek>=0&&holding==0&&previewUntil!=0&&now>previewUntil){seek=-1;previewUntil=0;}
            if(now-lastPaint>=33_333_333L&&painting.compareAndSet(false,true)){
                lastPaint=now;final double a=expand,seconds=media.trackSeconds(),previewSeek=seek,ms=musicScale;final boolean h=hot,e=editor,remain=remaining,info=layout.info,music=layout.music,hover=layout.expandOnHover,announce=layout.announceTracks;final String time=clock,latency=ping,frames=fps+" FPS";
                final boolean cds=layout.cooldowns;final java.util.List<AgentCooldowns.Entry> rows=cooldownRows;
                painter.execute(()->{try{PanelPainter.refreshAppearance();pending=new Frame(info?PanelPainter.info(time,frames,latency,now/1e9,false):null,music?PanelPainter.music(track,a,h,seconds,previewSeek,remain):null,e?editorImage(info,music,hover,announce,ms):null,cds?PanelPainter.cooldowns(rows):null);}catch(Throwable error){log("Panel painting failed",error);}finally{painting.set(false);}});
            }
            Frame update=pending;boolean dirty=update!=null;if(dirty){shown=update;pending=null;}if(shown==null)return;
            if(gl==null)gl=new AgentGL(glfwClass.getClassLoader());
            gl.begin(pixelWidth,pixelHeight);
            try{
                if(layout.info)gl.draw(0,shown.info,dirty,x(layout.infoX,layout.infoWidth())*scale,y(layout.infoY,layout.infoHeight())*scale,(float)(226*layout.infoScale()*scale),(float)(24*layout.infoScale()*scale));
                if(layout.music)gl.draw(1,shown.music,dirty,(float)musicX*scale,(float)musicY*scale,(float)(272*musicScale*scale),(float)(116*musicScale*scale));
                if(!editor&&layout.cooldowns&&shown.cooldowns!=null&&!cooldownRows.isEmpty()){double cs=layout.cooldownScale();gl.draw(3,shown.cooldowns,dirty,(float)((guiWidth-AgentCooldowns.WIDTH*cs)/2*scale),(guiHeight/2f+22)*scale,(float)(AgentCooldowns.WIDTH*cs*scale),(float)(shown.cooldowns.getHeight()/2f*cs*scale));}
                if(editor)gl.draw(2,shown.editor,dirty,(guiWidth-320)/2f*scale,46*scale,320*scale,shown.editor==null?editorHeight()*scale:shown.editor.getHeight()/2f*scale);
                if(preview){double elapsed=(now-previewStart)/1e9;if(captures==0&&elapsed>4||captures==1&&elapsed>9||captures==2&&elapsed>13){capture(captures++);}if(elapsed>15)window("glfwSetWindowShouldClose",new Class<?>[]{long.class,boolean.class},window,true);}
            }finally{gl.end();}
        }catch(Throwable error){failed=true;media.close();log("Overlay disabled after a rendering error",error);}
    }
    static BufferedImage editorImage(boolean info,boolean music,boolean hover,boolean announce,double musicScale){
        if(skyEditor)return skyImage();
        BufferedImage image=PanelPainter.canvas(320,278);Graphics2D g=PanelPainter.graphics(image);PanelPainter.glass(g,320,278,14,false);
        PanelPainter.text(g,"KYRO / Панели",14,10,290,13,PanelPainter.ACCENT,0,true);PanelPainter.text(g,"Тяни панели за фон · Right Shift / Esc — закрыть",14,30,290,8,PanelPainter.QUIET,0,false);
        button(g,12,50,"Верхний HUD",info);button(g,164,50,"Музыка",music);button(g,12,78,"Раскрытие",hover);button(g,164,78,"Новый трек",announce);
        PanelPainter.text(g,"Размер музыки · "+Math.round(musicScale*100)+"%",20,113,202,10,PanelPainter.QUIET,0,false);
        g.setColor(PanelPainter.ink(18));g.fillRoundRect(234,108,28,22,8,8);g.fillRoundRect(272,108,28,22,8,8);PanelPainter.text(g,"−",243,110,18,14,PanelPainter.WHITE,0,true);PanelPainter.text(g,"+",280,110,18,14,PanelPainter.WHITE,0,true);
        PanelPainter.text(g,"Размер острова · "+Math.round(layout.infoScale()*100)+"%",20,141,202,10,PanelPainter.QUIET,0,false);
        g.setColor(PanelPainter.ink(18));g.fillRoundRect(234,136,28,22,8,8);g.fillRoundRect(272,136,28,22,8,8);PanelPainter.text(g,"−",243,138,18,14,PanelPainter.WHITE,0,true);PanelPainter.text(g,"+",280,138,18,14,PanelPainter.WHITE,0,true);
        button(g,12,164,"КД предметов",layout.cooldowns);PanelPainter.text(g,"Рядом с прицелом",172,171,132,8,PanelPainter.QUIET,0,false);
        PanelPainter.text(g,"Размер КД · "+Math.round(layout.cooldownScale()*100)+"%",20,197,202,10,PanelPainter.QUIET,0,false);
        g.setColor(PanelPainter.ink(18));g.fillRoundRect(234,192,28,24,8,8);g.fillRoundRect(272,192,28,24,8,8);PanelPainter.text(g,"−",243,194,18,14,PanelPainter.WHITE,0,true);PanelPainter.text(g,"+",280,194,18,14,PanelPainter.WHITE,0,true);
        button(g,12,220,"Custom Sky",layout.customSky);action(g,164,220,"Настроить ›");
        PanelPainter.text(g,"Сбросить позиции",18,254,145,9,PanelPainter.WHITE,0,false);PanelPainter.text(g,"Готово",252,254,55,9,PanelPainter.ACCENT,0,true);g.dispose();return image;
    }
    static final int[] SKY_COLORS={0xa185cf,0x72b7f7,0xed9bc8,0xffb875,0x52669c,0x8fd2c3,0xc6b5e5};
    static int nextSkyColor(int color){for(int i=0;i<SKY_COLORS.length;i++)if(SKY_COLORS[i]==(color&0xffffff))return SKY_COLORS[(i+1)%SKY_COLORS.length];return SKY_COLORS[0];}
    static void action(Graphics2D g,int x,int y,String text){g.setColor(PanelPainter.ink(18));g.fillRoundRect(x,y,144,24,10,10);PanelPainter.text(g,text,x+8,y+5,128,9,PanelPainter.ACCENT,0,true);}
    static void skyRow(Graphics2D g,int y,String name,double mix,int color,boolean enabled){
        int index=y==80?0:y==142?1:2;boolean editing=skyColorEdit==index;
        PanelPainter.text(g,name+" · "+Math.round(PanelLayout.strength(mix)*100)+"%",20,y+5,82,9,enabled?PanelPainter.WHITE:PanelPainter.QUIET,0,false);
        g.setColor(PanelPainter.ink(editing?32:16));g.fillRoundRect(105,y+2,90,20,6,6);if(editing){g.setColor(PanelPainter.ACCENT);g.drawRoundRect(105,y+2,90,20,6,6);}
        String hex=editing?"#"+skyHex:String.format(java.util.Locale.ROOT,"#%06X",color&0xffffff);PanelPainter.text(g,hex,112,y+6,80,8.5f,editing?PanelPainter.ACCENT:PanelPainter.WHITE,0,true);
        g.setColor(new Color(color&0xffffff));g.fillRoundRect(202,y+3,22,18,5,5);g.setColor(PanelPainter.ink(enabled?22:10));g.fillRoundRect(234,y,28,24,8,8);g.fillRoundRect(272,y,28,24,8,8);PanelPainter.text(g,"−",243,y+2,18,14,enabled?PanelPainter.WHITE:PanelPainter.QUIET,0,true);PanelPainter.text(g,"+",280,y+2,18,14,enabled?PanelPainter.WHITE:PanelPainter.QUIET,0,true);
    }
    static BufferedImage skyImage(){
        BufferedImage image=PanelPainter.canvas(320,274);Graphics2D g=PanelPainter.graphics(image);PanelPainter.glass(g,320,274,14,false);
        PanelPainter.text(g,"KYRO / Custom Sky",14,10,290,13,PanelPainter.ACCENT,0,true);PanelPainter.text(g,skyHexError?"Нужен цвет из 6 HEX-цифр: #A185CF":"HEX #RRGGBB · Ctrl+V — вставить · −/+ — сила",14,30,290,8,skyHexError?new Color(232,117,131):PanelPainter.QUIET,0,false);
        button(g,12,50,"Custom Sky",layout.customSky);button(g,164,50,"Цвет темы",layout.skyUseThemeColor);
        skyRow(g,80,"Небо",layout.skyMix,layout.skyUseThemeColor?SkyHooks.themeColor:layout.skyColor,true);
        button(g,12,112,"Красить туман",layout.skyFog);button(g,164,112,"Как небо",layout.skyFogFollow);
        skyRow(g,142,"Туман",layout.skyFogFollow?layout.skyMix:layout.skyFogMix,layout.skyFogFollow?(layout.skyUseThemeColor?SkyHooks.themeColor:layout.skyColor):layout.skyFogColor,layout.skyFog&&!layout.skyFogFollow);
        button(g,12,174,"Красить облака",layout.skyClouds);button(g,164,174,"Убрать зарю",layout.skyNoSunset);
        skyRow(g,204,"Облака",layout.skyCloudMix,layout.skyCloudColor,layout.skyClouds);
        PanelPainter.text(g,"‹ Назад",18,250,145,9,PanelPainter.WHITE,0,true);PanelPainter.text(g,"Готово",252,250,55,9,PanelPainter.ACCENT,0,true);g.dispose();return image;
    }
    static void skyClick(double xx,double yy)throws Exception{
        skyHexError=false;for(int row=0;row<3;row++){int y=row==0?80:row==1?142:204;if(inside(xx,yy,105,y+2,90,20)){skyColorEdit=row;int color=row==0?(layout.skyUseThemeColor?SkyHooks.themeColor:layout.skyColor):row==1?(layout.skyFogFollow?(layout.skyUseThemeColor?SkyHooks.themeColor:layout.skyColor):layout.skyFogColor):layout.skyCloudColor;skyHex=String.format(java.util.Locale.ROOT,"%06X",color&0xffffff);skySelectAll=true;return;}}
        skyColorEdit=-1;
        if(inside(xx,yy,12,50,144,24))layout.customSky=!layout.customSky;else if(inside(xx,yy,164,50,144,24))layout.skyUseThemeColor=!layout.skyUseThemeColor;
        else if(inside(xx,yy,12,112,144,24))layout.skyFog=!layout.skyFog;else if(inside(xx,yy,164,112,144,24))layout.skyFogFollow=!layout.skyFogFollow;
        else if(inside(xx,yy,12,174,144,24))layout.skyClouds=!layout.skyClouds;else if(inside(xx,yy,164,174,144,24))layout.skyNoSunset=!layout.skyNoSunset;
        else if(inside(xx,yy,12,244,145,24))skyEditor=false;else if(inside(xx,yy,240,244,70,24))setEditor(false);
        else for(int row=0;row<3;row++){int y=row==0?80:row==1?142:204;boolean allowed=row==0||row==1&&layout.skyFog&&!layout.skyFogFollow||row==2&&layout.skyClouds;if(!allowed)continue;
            if(inside(xx,yy,202,y,22,24)){if(row==0){layout.skyColor=nextSkyColor(layout.skyColor);layout.skyUseThemeColor=false;}else if(row==1)layout.skyFogColor=nextSkyColor(layout.skyFogColor);else layout.skyCloudColor=nextSkyColor(layout.skyCloudColor);}
            int direction=inside(xx,yy,234,y,28,24)?-1:inside(xx,yy,272,y,28,24)?1:0;if(direction!=0){double old=row==0?layout.skyMix:row==1?layout.skyFogMix:layout.skyCloudMix;double value=PanelLayout.strength(Math.round((PanelLayout.strength(old)+direction*.05)*20)/20.0);if(row==0)layout.skyMix=value;else if(row==1)layout.skyFogMix=value;else layout.skyCloudMix=value;}
        }save();
    }
    static boolean pasteSkyHex(String value){
        if(value==null)return false;String hex=value.trim();if(hex.startsWith("#"))hex=hex.substring(1);else if(hex.startsWith("0x")||hex.startsWith("0X"))hex=hex.substring(2);if(!hex.matches("[0-9a-fA-F]{6}")){skyHexError=true;return false;}skyHexError=false;skyHex=hex.toUpperCase(java.util.Locale.ROOT);skySelectAll=true;applySkyHex();return true;
    }
    static void applySkyHex(){
        if(skyColorEdit<0||!skyHex.matches("[0-9a-fA-F]{6}"))return;int color=Integer.parseInt(skyHex,16);
        layout.customSky=true;if(skyColorEdit==0){layout.skyColor=color;layout.skyUseThemeColor=false;}else if(skyColorEdit==1){layout.skyFogColor=color;layout.skyFogFollow=false;layout.skyFog=true;}else{layout.skyCloudColor=color;layout.skyClouds=true;}save();
    }
    static void skyHexKey(int key,int action,int mods)throws Exception{
        if(action!=1&&action!=2)return;
        if(key==51&&(mods&1)!=0)return;skyHexError=false;
        if((mods&2)!=0){if(key==65)skySelectAll=true;else if(key==86)try{Object value=window("glfwGetClipboardString",new Class<?>[]{long.class},activeWindow);if(value instanceof String)pasteSkyHex((String)value);}catch(Exception ignored){skyHexError=true;}return;}
        if(key==257||key==335){applySkyHex();skyColorEdit=-1;return;}
        if(key==259||key==261){skyHex=skySelectAll?"":skyHex.isEmpty()?"":skyHex.substring(0,skyHex.length()-1);skySelectAll=false;return;}
        char digit=key>=48&&key<=57?(char)key:key>=65&&key<=70?(char)key:key>=320&&key<=329?(char)('0'+key-320):0;if(digit!=0){if(skySelectAll){skyHex="";skySelectAll=false;}if(skyHex.length()<6){skyHex+=digit;applySkyHex();}}
    }
    static void button(Graphics2D g,int x,int y,String name,boolean on){g.setColor(PanelPainter.ink(18));g.fillRoundRect(x,y,144,24,10,10);PanelPainter.text(g,name,x+8,y+5,102,9,PanelPainter.WHITE,0,false);g.setColor(on?PanelPainter.ACCENT:new Color(80,74,88));g.fillRoundRect(x+111,y+7,25,11,11,11);g.setColor(Color.WHITE);g.fillOval(x+(on?126:113),y+9,7,7);}
    static void setEditor(boolean open)throws Exception{
        if(editor==open)return;
        if(open){wasLocked=cursor()==212995;AgentInput.releaseKeys(activeWindow);
            skyEditor=false;skyColorEdit=-1;editor=true;if(wasLocked)cursor(212993);
        }else{dragging=-1;holding=0;seek=-1;save();if(wasLocked)cursor(212995);editor=false;}
    }
    public static boolean key(long window,int key,int action)throws Exception{
        return key(window,key,action,0);
    }
    public static boolean key(long window,int key,int action,int mods)throws Exception{
        if(!ready||window!=activeWindow)return false;
        if(key==344){if(action==1)setEditor(!editor);return true;}
        if(editor){if(key==256&&action==1)setEditor(false);else if(skyEditor&&skyColorEdit>=0)skyHexKey(key,action,mods);return true;}return false;
    }
    public static boolean mouse(long window,int button,int action)throws Exception{
        if(!ready||window!=activeWindow)return false;pointer();if(!editor&&cursor()==212995)return false;
        if(button!=0)return editor;
        if(action==0){if(dragging>=0){dragging=-1;save();return true;}PanelPainter.Track t=media.snapshot();if(holding!=0){if(seek>=0){media.command("seek",seek*t.duration*1000);previewUntil=System.nanoTime()+700_000_000L;}else media.command(holding<0?"previous":"next",0);holding=0;return true;}if(seek>=0&&previewUntil==Long.MAX_VALUE){media.command("seek",seek*t.duration*1000);previewUntil=System.nanoTime()+700_000_000L;return true;}return editor;}
        if(action!=1)return editor;
        double mx=mouseX,my=mouseY;
        if(editor){double ex=(guiWidth-320)/2.0,ey=46;if(inside(mx,my,ex,ey,320,editorHeight())){
            double xx=mx-ex,yy=my-ey;
            if(skyEditor){skyClick(xx,yy);return true;}
            if(inside(xx,yy,12,50,144,24))layout.info=!layout.info;else if(inside(xx,yy,164,50,144,24)){layout.music=!layout.music;media.enabled=layout.music;}
            else if(inside(xx,yy,12,78,144,24))layout.expandOnHover=!layout.expandOnHover;else if(inside(xx,yy,164,78,144,24))layout.announceTracks=!layout.announceTracks;
            else if(inside(xx,yy,234,108,28,22))layout.musicScale=Math.max(.4,Math.round((layout.musicScale()-.05)*20)/20.0);
            else if(inside(xx,yy,272,108,28,22))layout.musicScale=Math.min(1,Math.round((layout.musicScale()+.05)*20)/20.0);
            else if(inside(xx,yy,234,136,28,22))layout.infoScale=Math.max(.5,Math.round((layout.infoScale()-.05)*20)/20.0);
            else if(inside(xx,yy,272,136,28,22))layout.infoScale=Math.min(1,Math.round((layout.infoScale()+.05)*20)/20.0);
            else if(inside(xx,yy,12,164,144,24))layout.cooldowns=!layout.cooldowns;
            else if(inside(xx,yy,234,192,28,24))layout.cooldownScale=Math.max(.5,Math.round((layout.cooldownScale()-.05)*20)/20.0);else if(inside(xx,yy,272,192,28,24))layout.cooldownScale=Math.min(1.25,Math.round((layout.cooldownScale()+.05)*20)/20.0);
            else if(inside(xx,yy,12,220,144,24))layout.customSky=!layout.customSky;else if(inside(xx,yy,164,220,144,24))skyEditor=true;
            else if(inside(xx,yy,12,248,164,22)){layout.infoX=.02;layout.infoY=.025;layout.musicX=.02;layout.musicY=.83;}else if(inside(xx,yy,240,248,70,22))setEditor(false);save();return true;
        }}
        if(layout.music&&inside(mx,my,musicX,musicY,272*layout.musicScale(),musicHeight)){
            double xx=(mx-musicX)/layout.musicScale(),yy=(my-musicY)/layout.musicScale();PanelPainter.Track t=media.snapshot();controlKey=t.source+"\0"+t.title+"\0"+t.artist;
            double playX=PanelPainter.lerp(208,122,expand),playY=PanelPainter.lerp(10,79,expand),wingY=PanelPainter.lerp(14,83,expand);
            if(t.available){
                if(inside(xx,yy,playX,playY,28,28)||inside(xx,yy,6+6*expand,6+6*expand,36+12*expand,36+12*expand)){media.command("toggle",0);return true;}
                if(inside(xx,yy,PanelPainter.lerp(178,92,expand),wingY,30,20)||inside(xx,yy,PanelPainter.lerp(236,150,expand),wingY,30,20)){boolean previous=xx<playX;if(previous?t.canPrevious:t.canNext){holding=previous?-1:1;holdAt=System.nanoTime();holdPosition=t.position;}return true;}
                if(t.canSeek&&t.duration>0&&inside(xx,yy,10,expand>.5?58:40,252,expand>.5?16:8)){seek=PanelPainter.clamp((xx-12)/248);previewUntil=Long.MAX_VALUE;return true;}
                if(expand>.9&&inside(xx,yy,210,73,54,14)){remaining=!remaining;return true;}
                if(expand>.9&&t.volume>=0&&inside(xx,yy,204,90,60,17)){media.command("volume",PanelPainter.clamp((xx-207)/53));return true;}
            }
            if(editor){dragging=1;dragX=mx-musicX;dragY=my-y(layout.musicY,layout.musicHeight());}return true;
        }
        if(editor&&layout.info&&inside(mx,my,x(layout.infoX,layout.infoWidth()),y(layout.infoY,layout.infoHeight()),layout.infoWidth(),layout.infoHeight())){dragging=0;dragX=mx-x(layout.infoX,layout.infoWidth());dragY=my-y(layout.infoY,layout.infoHeight());return true;}
        return editor;
    }
    public static boolean scroll(long window,double amount)throws Exception{if(!ready||window!=activeWindow)return false;pointer();if(!editor&&cursor()==212995)return false;if(layout.music&&inside(mouseX,mouseY,musicX,musicY,272*layout.musicScale(),musicHeight)){PanelPainter.Track t=media.snapshot();if(t.volume>=0)media.command("volume",PanelPainter.clamp(t.volume+amount*.05));return true;}return editor;}
    static void capture(int number){
        String dir=System.getProperty("kyro.hud.capture");if(dir==null)return;
        try{ByteBuffer pixels=ByteBuffer.allocateDirect(pixelWidth*pixelHeight*4);int old=((Number)gl.ints("GL11","glGetInteger",35053)).intValue();gl.ints("GL15","glBindBuffer",35051,0);
            try{gl.call("GL11","glReadPixels",new Class<?>[]{int.class,int.class,int.class,int.class,int.class,int.class,ByteBuffer.class},0,0,pixelWidth,pixelHeight,6408,5121,pixels);}finally{gl.ints("GL15","glBindBuffer",35051,old);}
            BufferedImage image=new BufferedImage(pixelWidth,pixelHeight,BufferedImage.TYPE_INT_ARGB);for(int y=0;y<pixelHeight;y++)for(int x=0;x<pixelWidth;x++){int offset=(y*pixelWidth+x)*4;int r=pixels.get(offset)&255,g=pixels.get(offset+1)&255,b=pixels.get(offset+2)&255;image.setRGB(x,pixelHeight-1-y,0xff000000|r<<16|g<<8|b);}
            Path target=Paths.get(dir);Files.createDirectories(target);javax.imageio.ImageIO.write(image,"png",target.resolve("agent-"+number+".png").toFile());System.out.println("[KYRO HUD] Captured preview "+number+" (world="+game.world+")");
        }catch(Exception error){log("Preview capture failed",error);}
    }
}
