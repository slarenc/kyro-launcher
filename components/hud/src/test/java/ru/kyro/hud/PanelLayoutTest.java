package ru.kyro.hud;
import com.google.gson.Gson;
import java.nio.file.*;
public final class PanelLayoutTest {
    static class Client { public Object overlay,screen; }
    static class LoadingOverlay {}
    static class ReceivingLevelScreen {}
    static void check(boolean condition){if(!condition)throw new AssertionError();}
    public static void main(String[]args)throws Exception {
        AgentGame game=new AgentGame();Client client=new Client();java.lang.reflect.Field cf=AgentGame.class.getDeclaredField("client");cf.setAccessible(true);cf.set(game,client);
        check(!game.loading());client.overlay=new LoadingOverlay();check(game.loading());client.overlay=null;client.screen=new ReceivingLevelScreen();check(game.loading());client.screen=new Object();check(!game.loading());
        Path dir=Files.createTempDirectory("kyro-layout-test");Path file=dir.resolve("panels.json");Gson json=new Gson();
        PanelLayout original=PanelLayout.load(file,json);
        check(original.musicScale()==.65);original.musicScale=.55;check(original.infoScale()==.8);original.infoScale=.7;
        original.move(0,340,160,10,10,960,540);original.move(1,500,350,20,15,960,540);original.move(2,180,210,10,10,960,540);original.music=false;original.info=false;original.save(file,json);
        PanelLayout restored=PanelLayout.load(file,json);
        check(restored.cooldownX==original.cooldownX&&restored.cooldownY==original.cooldownY);
        check(restored.infoX==original.infoX&&restored.infoY==original.infoY);
        check(restored.musicX==original.musicX&&restored.musicY==original.musicY&&!restored.music&&!restored.info);
        check(restored.musicScale()==.55);
        check(restored.infoScale()==.7);restored.move(0,10000,10000,0,0,320,180);check(PanelLayout.coordinate(restored.infoX,320,restored.infoWidth())==320-restored.infoWidth());
        check(PanelLayout.coordinate(restored.musicX,320,restored.musicWidth())<=320-restored.musicWidth());
        restored.move(1,10000,-100,0,0,320,180);
        check(PanelLayout.coordinate(restored.musicX,320,restored.musicWidth())==320-restored.musicWidth()&&restored.musicY==0);
        restored.musicScale=Double.NaN;check(restored.musicScale()==.65);restored.musicScale=3;check(restored.musicScale()==1);restored.musicScale=-3;check(restored.musicScale()==.4);restored.musicScale=.55;
        check(PanelLayout.coordinate(Double.NaN,320,PanelLayout.MUSIC_W)>=0);
        restored.save(file,json);check(!Files.exists(dir.resolve("panels.json.tmp")));
        Files.delete(file);Files.delete(dir);
        System.out.println("KYRO layout: drag, persistence, visibility and resize checks passed");
    }
}
