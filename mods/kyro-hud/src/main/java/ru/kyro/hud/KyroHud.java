package ru.kyro.hud;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.fabricmc.fabric.api.client.rendering.v1.hud.HudElementRegistry;
import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.Click;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;
import net.minecraft.text.Text;
import net.minecraft.util.Identifier;
import org.lwjgl.glfw.GLFW;
import java.io.*;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.nio.file.*;
import java.time.LocalTime;
import java.time.format.DateTimeFormatter;
import java.io.IOException;
import java.util.*;

public final class KyroHud implements ClientModInitializer {
    static final MinecraftClient MC = MinecraftClient.getInstance();
    static final Gson JSON = new GsonBuilder().setPrettyPrinting().create();
    static final Path CONFIG = FabricLoader.getInstance().getConfigDir().resolve("kyro-panels.json");

    static PanelLayout layout = new PanelLayout();
    static final Player player = new Player();
    static String error = "";
    static final int ACCENT = 0xffcd83ea;
    static final Identifier LOGO=Identifier.of("kyro_hud","textures/logo.png");
    static final DateTimeFormatter CLOCK=DateTimeFormatter.ofPattern("HH:mm");
    static final int[] CORNERS={4,2,1,1,0,0};
    static long statsAt=0;
    static String clock="",fps="",ping="",lastTitle="",lastSubtitle="",shownTitle="",shownSubtitle="";
    public void onInitializeClient() {
        try {
            layout=PanelLayout.load(CONFIG,JSON);

        } catch (Exception e) { error = "Не удалось прочитать настройки"; }
        player.enabled=layout.music;
        player.start();
        var key = KeyBindingHelper.registerKeyBinding(new KeyBinding("key.kyro_hud.edit", InputUtil.Type.KEYSYM,
            GLFW.GLFW_KEY_RIGHT_SHIFT, KeyBinding.Category.create(Identifier.of("kyro_hud", "panels"))));
        ClientTickEvents.END_CLIENT_TICK.register(client -> {
            while (key.wasPressed()) if (client.player != null && client.currentScreen == null) client.setScreen(new Editor());
        });
        HudElementRegistry.addLast(Identifier.of("kyro_hud", "panels"), (ctx, tick) -> {
            if (MC.player != null && !MC.options.hudHidden && !(MC.currentScreen instanceof Editor)) renderPanels(ctx, false);
        });
    }
    static void save() {
        try {
            layout.save(CONFIG,JSON);
            error = "";
        } catch (IOException e) { error = "Не удалось сохранить позиции"; }
    }
    static int x(double fraction, int width) { return coordinate(fraction, MC.getWindow().getScaledWidth(), width); }
    static int y(double fraction, int height) { return coordinate(fraction, MC.getWindow().getScaledHeight(), height); }
    static int coordinate(double value, int screen, int size) {
        return PanelLayout.coordinate(value,screen,size);
    }
    static void round(DrawContext c,int x,int y,int w,int h,int color) {
        int r=Math.min(6,h/2);
        c.fill(x,y+r,x+w,y+h-r,color);
        for(int row=0;row<r;row++){
            int inset=CORNERS[row];
            c.fill(x+inset,y+row,x+w-inset,y+row+1,color);
            c.fill(x+inset,y+h-row-1,x+w-inset,y+h-row,color);
        }
    }
    static void card(DrawContext c,int x,int y,int w,int h) {
        round(c,x,y+1,w,h,0x35000000);
        round(c,x,y,w,h,0xd51a191e);
    }
    static void text(DrawContext c,String value,int x,int y,int color){c.drawText(MC.textRenderer,value,x,y,color,false);}
    static void renderPanels(DrawContext c,boolean edit){
        if(layout.info){
            long now=System.nanoTime();
            if(now-statsAt>500_000_000L){
                statsAt=now;clock=LocalTime.now().format(CLOCK);fps=MC.getCurrentFps()+" FPS";
                var entry=MC.player==null||MC.getNetworkHandler()==null?null:MC.getNetworkHandler().getPlayerListEntry(MC.player.getUuid());
                ping=MC.isInSingleplayer()?"LOCAL":entry==null?"— ms":entry.getLatency()+" ms";
            }
            int x=x(layout.infoX,PanelLayout.INFO_W),y=y(layout.infoY,PanelLayout.INFO_H);
            card(c,x,y,PanelLayout.INFO_W,PanelLayout.INFO_H);
            c.drawTexturedQuad(LOGO,x+5,y+3,x+19,y+17,0,1,0,1);
            text(c,clock,x+27,y+6,0xffe4e1e8);
            c.fill(x+60,y+5,x+61,y+15,0x40877f91);
            text(c,fps,x+69,y+6,0xffe4e1e8);
            c.fill(x+118,y+5,x+119,y+15,0x40877f91);
            text(c,ping,x+127,y+6,0xffaaa4b4);
        }
        if(layout.music){
            int x=x(layout.musicX,PanelLayout.MUSIC_W),y=y(layout.musicY,PanelLayout.MUSIC_H);
            card(c,x,y,PanelLayout.MUSIC_W,PanelLayout.MUSIC_H);
            round(c,x+4,y+4,28,28,0xff302433);
            c.drawTexturedQuad(LOGO,x+8,y+8,x+28,y+28,0,1,0,1);
            MediaState media=player.state;
            String title=player.title(),subtitle=player.subtitle();
            if(!title.equals(lastTitle)){lastTitle=title;shownTitle=MC.textRenderer.trimToWidth(title,126);}
            if(!subtitle.equals(lastSubtitle)){lastSubtitle=subtitle;shownSubtitle=MC.textRenderer.trimToWidth(subtitle,126);}
            text(c,shownTitle,x+39,y+7,0xffeeebf1);
            text(c,shownSubtitle,x+39,y+20,0xff9a929f);
            text(c,"|<",x+171,y+14,media.canPrevious?0xffc5bec9:0xff625c65);
            round(c,x+190,y+7,22,22,media.canToggle?0xffc95cd9:0xff483947);
            text(c,media.playing?"II":">",x+198,y+14,0xffffffff);
            text(c,">|",x+221,y+14,media.canNext?0xffc5bec9:0xff625c65);
            c.fill(x+6,y+34,x+232,y+35,0xff3a303e);
            c.fill(x+6,y+34,x+6+(int)(226*player.progress()),y+35,ACCENT);
        }
    }
    static final class Editor extends Screen {
        int dragging=-1; double dx,dy;
        ButtonWidget infoToggle,musicToggle;
        Editor(){super(Text.literal("KYRO · Редактор панелей"));}
        protected void init(){
            addDrawableChild(ButtonWidget.builder(Text.literal("Готово"),b->close()).dimensions(width/2-50,height-28,100,20).build());
            addDrawableChild(ButtonWidget.builder(Text.literal("Сбросить позиции"),b->{layout.infoX=.02;layout.infoY=.025;layout.musicX=.02;layout.musicY=.83;save();}).dimensions(width/2-80,38,160,20).build());
            infoToggle=addDrawableChild(ButtonWidget.builder(Text.empty(),b->{layout.info=!layout.info;refreshToggles();save();}).dimensions(width/2-145,62,140,20).build());
            musicToggle=addDrawableChild(ButtonWidget.builder(Text.empty(),b->{layout.music=!layout.music;player.enabled=layout.music;refreshToggles();save();}).dimensions(width/2+5,62,140,20).build());
            refreshToggles();
        }
        void refreshToggles(){
            infoToggle.setMessage(Text.literal("Инфо: "+(layout.info?"ВКЛ":"ВЫКЛ")));
            musicToggle.setMessage(Text.literal("Музыка: "+(layout.music?"ВКЛ":"ВЫКЛ")));
        }
        public boolean shouldPause(){return false;}
        public void render(DrawContext c,int mx,int my,float delta){
            c.fill(0,0,width,height,0x44070812);
            c.drawCenteredTextWithShadow(MC.textRenderer,"KYRO / Перетащи панели мышью",width/2,12,ACCENT);
            c.drawCenteredTextWithShadow(MC.textRenderer,error.isEmpty()?"Позиции сохраняются автоматически · Esc — закрыть":error,width/2,25,0xffb8b1cb);
            if(dragging>=0 && GLFW.glfwGetMouseButton(MC.getWindow().getHandle(),0)==GLFW.GLFW_PRESS){
                layout.move(dragging,mx,my,dx,dy,width,height);
            } else if(dragging>=0){dragging=-1;save();}
            renderPanels(c,true); super.render(c,mx,my,delta);
        }
        public boolean mouseClicked(Click click,boolean doubled){
            if(super.mouseClicked(click,doubled))return true;
            if(click.button()!=0)return false;
            double mx=click.x(),my=click.y();
            int px=x(layout.musicX,PanelLayout.MUSIC_W),py=y(layout.musicY,PanelLayout.MUSIC_H);
            if(layout.music&&mx>=px&&mx<=px+PanelLayout.MUSIC_W&&my>=py&&my<=py+PanelLayout.MUSIC_H){
                if(mx>px+168&&my>py+4&&my<py+31){if(mx<px+188)player.skip(-1);else if(mx<px+216)player.toggle();else player.skip(1);return true;}
                dragging=1;dx=mx-px;dy=my-py;return true;
            }
            px=x(layout.infoX,PanelLayout.INFO_W);py=y(layout.infoY,PanelLayout.INFO_H);
            if(layout.info&&mx>=px&&mx<=px+PanelLayout.INFO_W&&my>=py&&my<=py+PanelLayout.INFO_H){dragging=0;dx=mx-px;dy=my-py;return true;}
            return false;
        }
        public void close(){save();super.close();}
        public void removed(){save();}
    }
    static final class MediaState {
        boolean available,playing,canToggle,canNext,canPrevious;
        String title="",artist="",error="";
        long position,duration;
    }
    static final class Player {
        volatile MediaState state=new MediaState();
        final ConcurrentLinkedQueue<String> actions=new ConcurrentLinkedQueue<>();
        volatile Process process;
        volatile boolean enabled=true;
        String title(){return state.available&&!state.title.isBlank()?state.title:"Музыка из браузера";}
        String subtitle(){return !state.error.isBlank()?state.error:!state.available?"Включи трек в браузере":state.artist.isBlank()?(state.playing?"KYRO / NOW PLAYING":"KYRO / PAUSED"):state.artist;}
        boolean playing(){return state.playing;}
        double progress(){return state.duration<=0?0:Math.max(0,Math.min(1,(double)state.position/state.duration));}
        void toggle(){if(state.canToggle)actions.offer("toggle");}
        void skip(int direction){if(direction<0?state.canPrevious:state.canNext)actions.offer(direction<0?"previous":"next");}
        void start(){
            Thread worker=new Thread(()->{
                Path helper=FabricLoader.getInstance().getConfigDir().resolve("kyro-media.exe");
                if(!Files.isRegularFile(helper)){state.error="Установи панели через KYRO";return;}
                try{
                    process=new ProcessBuilder(helper.toAbsolutePath().toString()).redirectError(ProcessBuilder.Redirect.DISCARD).start();
                    try(var input=new BufferedReader(new InputStreamReader(process.getInputStream(),java.nio.charset.StandardCharsets.UTF_8));
                        var output=new BufferedWriter(new OutputStreamWriter(process.getOutputStream(),java.nio.charset.StandardCharsets.UTF_8))){
                        while(!Thread.currentThread().isInterrupted()){
                            if(!enabled){Thread.sleep(1000);continue;}
                            String action=actions.poll();output.write(action==null?"poll":action);output.newLine();output.flush();
                            String line=input.readLine();if(line==null)break;
                            MediaState next=JSON.fromJson(line,MediaState.class);if(next!=null)state=next;
                            Thread.sleep(500);
                        }
                    }
                }catch(Exception e){state=new MediaState();state.error="Медиапанель недоступна";}
                finally{if(process!=null)process.destroy();}
            },"KYRO Media Bridge");
            worker.setDaemon(true);worker.start();
            Runtime.getRuntime().addShutdownHook(new Thread(()->{if(process!=null)process.destroy();}));
        }
    }
}
