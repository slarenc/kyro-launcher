package ru.kyro.hud;

import com.google.gson.Gson;
import java.io.IOException;
import java.nio.file.*;

/** Screen fractions keep panels on screen when resolution or GUI scale changes. */
public final class PanelLayout {
    public static final int INFO_W=178,INFO_H=20,MUSIC_W=238,MUSIC_H=36;
    public double infoX=.02, infoY=.025, musicX=.02, musicY=.83;
    public boolean info=true,music=true;
    public static int coordinate(double value,int screen,int size) {
        if(!Double.isFinite(value))value=.02;
        return (int)Math.max(0,Math.min(Math.max(0,screen-size),value*screen));
    }
    public void move(int panel,double mx,double my,double dx,double dy,int width,int height) {
        int w=panel==0?INFO_W:MUSIC_W,h=panel==0?INFO_H:MUSIC_H;
        double x=Math.max(0,Math.min(width-w,mx-dx))/Math.max(1,width);
        double y=Math.max(0,Math.min(height-h,my-dy))/Math.max(1,height);
        if(panel==0){infoX=x;infoY=y;}else{musicX=x;musicY=y;}
    }
    public void save(Path file,Gson json)throws IOException {
        Files.createDirectories(file.getParent());
        Path temporary=file.resolveSibling(file.getFileName()+".tmp");
        Files.writeString(temporary,json.toJson(this));
        Files.move(temporary,file,StandardCopyOption.REPLACE_EXISTING);
    }
    public static PanelLayout load(Path file,Gson json)throws IOException {
        if(!Files.exists(file))return new PanelLayout();
        PanelLayout result=json.fromJson(Files.readString(file),PanelLayout.class);
        if(result==null)return new PanelLayout();
        result.infoX=sane(result.infoX);result.infoY=sane(result.infoY);
        result.musicX=sane(result.musicX);result.musicY=sane(result.musicY);
        return result;
    }
    static double sane(double value){return Double.isFinite(value)?Math.max(0,Math.min(1,value)):.02;}
}
