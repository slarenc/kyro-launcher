package ru.kyro.hud;
import com.google.gson.Gson;
import java.nio.file.*;
public final class PanelLayoutTest {
    static void check(boolean condition){if(!condition)throw new AssertionError();}
    public static void main(String[]args)throws Exception {
        Path dir=Files.createTempDirectory("kyro-layout-test");Path file=dir.resolve("panels.json");Gson json=new Gson();
        PanelLayout original=PanelLayout.load(file,json);
        original.move(0,340,160,10,10,960,540);original.move(1,500,350,20,15,960,540);original.music=false;original.info=false;original.save(file,json);
        PanelLayout restored=PanelLayout.load(file,json);
        check(restored.infoX==original.infoX&&restored.infoY==original.infoY);
        check(restored.musicX==original.musicX&&restored.musicY==original.musicY&&!restored.music&&!restored.info);
        check(PanelLayout.coordinate(restored.musicX,320,PanelLayout.MUSIC_W)<=320-PanelLayout.MUSIC_W);
        restored.move(1,10000,-100,0,0,320,180);
        check(PanelLayout.coordinate(restored.musicX,320,PanelLayout.MUSIC_W)==320-PanelLayout.MUSIC_W&&restored.musicY==0);
        check(PanelLayout.coordinate(Double.NaN,320,PanelLayout.MUSIC_W)>=0);
        restored.save(file,json);check(!Files.exists(dir.resolve("panels.json.tmp")));
        Files.delete(file);Files.delete(dir);
        System.out.println("KYRO layout: drag, persistence, visibility and resize checks passed");
    }
}
