package ru.kyro.hud;

import com.google.gson.Gson;
import java.io.IOException;
import java.nio.file.*;

/** Screen fractions keep panels on screen when resolution or GUI scale changes. */
public final class PanelLayout {
    public static final int INFO_W=226,INFO_H=24,MUSIC_W=272,MUSIC_H=48;
    public double infoX=.02, infoY=.025, musicX=.02, musicY=.83;
    public boolean info=true,music=true,cooldowns=true;
    public boolean cooldownNames=true,cooldownMinutes=true;
    public java.util.List<CooldownRule> cooldownRules=new java.util.ArrayList<>();
    public static final class CooldownRule {
        public String key="",label="";public double seconds;
        public CooldownRule(){}
        public CooldownRule(String key,String label,double seconds){this.key=key;this.label=label;this.seconds=seconds;}
    }
    public boolean customSky=false,skyUseThemeColor=true,skyFog=true,skyFogFollow=true,skyClouds=true,skyNoSunset=true;
    public int skyColor=0xa185cf,skyFogColor=0xa185cf,skyCloudColor=0xc6b5e5;
    public double skyMix=.65,skyFogMix=.5,skyCloudMix=.35;
    static double strength(double value){return Double.isFinite(value)?Math.max(0,Math.min(1,value)):.5;}
    public boolean expandOnHover=true,announceTracks=true;
    public double musicScale=.65;
    public int cooldownSizeRevision=1;public double cooldownScale=.7;
    public double cooldownScale(){return Double.isFinite(cooldownScale)?Math.max(.2,Math.min(1.5,cooldownScale)):.7;}
    public double infoScale=.8;
    public double infoScale(){return Double.isFinite(infoScale)?Math.max(.5,Math.min(1,infoScale)):.8;}
    public int infoWidth(){return (int)Math.ceil(INFO_W*infoScale());}
    public int infoHeight(){return (int)Math.ceil(INFO_H*infoScale());}
    public double musicScale(){return Double.isFinite(musicScale)?Math.max(.4,Math.min(1,musicScale)):.65;}
    public int musicWidth(){return (int)Math.ceil(MUSIC_W*musicScale());}
    public int musicHeight(){return (int)Math.ceil(MUSIC_H*musicScale());}
    public static int coordinate(double value,int screen,int size) {
        if(!Double.isFinite(value))value=.02;
        return (int)Math.max(0,Math.min(Math.max(0,screen-size),value*screen));
    }
    public void move(int panel,double mx,double my,double dx,double dy,int width,int height) {
        int w=panel==0?infoWidth():musicWidth(),h=panel==0?infoHeight():musicHeight();
        double x=Math.max(0,Math.min(width-w,mx-dx))/Math.max(1,width);
        double y=Math.max(0,Math.min(height-h,my-dy))/Math.max(1,height);
        if(panel==0){infoX=x;infoY=y;}else{musicX=x;musicY=y;}
    }
    public void save(Path file,Gson json)throws IOException {
        Files.createDirectories(file.getParent());
        Path temporary=file.resolveSibling(file.getFileName()+".tmp");
        Files.write(temporary,json.toJson(this).getBytes(java.nio.charset.StandardCharsets.UTF_8));
        Files.move(temporary,file,StandardCopyOption.REPLACE_EXISTING);
    }
    public static PanelLayout load(Path file,Gson json)throws IOException {
        if(!Files.exists(file))return new PanelLayout();
        PanelLayout result=json.fromJson(new String(Files.readAllBytes(file),java.nio.charset.StandardCharsets.UTF_8),PanelLayout.class);
        if(result==null)return new PanelLayout();
        if(!new com.google.gson.JsonParser().parse(new String(Files.readAllBytes(file),java.nio.charset.StandardCharsets.UTF_8)).getAsJsonObject().has("cooldownSizeRevision"))result.cooldownScale=.7;
        result.infoX=sane(result.infoX);result.infoY=sane(result.infoY);
        result.musicX=sane(result.musicX);result.musicY=sane(result.musicY);
        result.musicScale=result.musicScale();
        result.cooldownScale=result.cooldownScale();
        java.util.List<CooldownRule> rules=new java.util.ArrayList<>();if(result.cooldownRules!=null)for(CooldownRule rule:result.cooldownRules)if(rule!=null&&rule.key!=null&&!rule.key.isEmpty()&&rule.key.length()<=4096&&rule.label!=null&&!rule.label.trim().isEmpty()&&rule.label.length()<=256&&Double.isFinite(rule.seconds)&&rule.seconds>0&&rule.seconds<=86400&&rules.size()<24)rules.add(rule);result.cooldownRules=rules;
        result.infoScale=result.infoScale();
        return result;
    }
    static double sane(double value){return Double.isFinite(value)?Math.max(0,Math.min(1,value)):.02;}
}
