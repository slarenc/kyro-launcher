package ru.kyro.hud;

import javax.imageio.ImageIO;
import java.awt.*;
import java.awt.geom.*;
import java.awt.image.BufferedImage;
import java.io.IOException;
import java.io.InputStream;
import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;

/** Supersampled native counterpart of the Paradox strip/card and island geometry. */
final class PanelPainter {
    static final int SCALE=2, MUSIC_W=272, STRIP_H=48, CARD_H=116, INFO_W=226, INFO_H=24;
    static Color ACCENT=new Color(161,133,207), WHITE=new Color(239,237,243), QUIET=new Color(160,153,168);
    static boolean light;static String appearance="";static long appearanceCheck;
    static Color ink(int alpha){return new Color(light?33:255,light?27:255,light?43:255,alpha);}
    static void appearance(String theme,String accent){
        String key=theme+accent;if(key.equals(appearance))return;
        Color color;try{color=Color.decode(accent);}catch(Exception ignored){color=new Color(161,133,207);}
        light="light".equals(theme);ACCENT=color;SkyHooks.themeColor=color.getRGB();WHITE=light?new Color(33,27,43):new Color(239,237,243);QUIET=light?new Color(105,96,116):new Color(160,153,168);
        appearance=key;SKIN.remove();
    }
    static void refreshAppearance(){
        long now=System.nanoTime();if(now<appearanceCheck)return;appearanceCheck=now+1_000_000_000L;
        String path=System.getProperty("kyro.hud.appearance");if(path==null)return;
        try{com.google.gson.JsonObject json=new com.google.gson.Gson().fromJson(new String(java.nio.file.Files.readAllBytes(java.nio.file.Paths.get(path)),java.nio.charset.StandardCharsets.UTF_8),com.google.gson.JsonObject.class);appearance(json.get("theme").getAsString(),json.get("accent").getAsString());}catch(Exception ignored){}
    }
    static final Font REGULAR=font("Hud-Regular.ttf"), MEDIUM=font("Hud-SemiBold.ttf");
    static final BufferedImage LOGO=logo();
    static final class Track {
        final boolean available,playing,canToggle,canPrevious,canNext,canSeek;
        final String title,artist,source,error;
        final double position,duration,volume;final BufferedImage cover;
        Track(boolean available,boolean playing,boolean canToggle,boolean canPrevious,boolean canNext,boolean canSeek,String title,String artist,String source,String error,double position,double duration,double volume,BufferedImage cover){
            this.available=available;this.playing=playing;this.canToggle=canToggle;this.canPrevious=canPrevious;this.canNext=canNext;this.canSeek=canSeek;
            this.title=title;this.artist=artist;this.source=source;this.error=error;this.position=position;this.duration=duration;this.volume=volume;this.cover=cover;
        }
        boolean available(){return available;}boolean playing(){return playing;}boolean canToggle(){return canToggle;}boolean canPrevious(){return canPrevious;}boolean canNext(){return canNext;}boolean canSeek(){return canSeek;}
        String title(){return title;}String artist(){return artist;}String source(){return source;}String error(){return error;}
        double position(){return position;}double duration(){return duration;}double volume(){return volume;}BufferedImage cover(){return cover;}
        double progress(){return duration<=0?0:clamp(position/duration);}
    }
    static Font font(String file){
        try(InputStream in=resource("assets/kyro_hud/fonts/"+file)){
            if(in==null)throw new IOException(file); return Font.createFont(Font.TRUETYPE_FONT,in);
        }catch(Exception e){return new Font(Font.SANS_SERIF,Font.PLAIN,12);}
    }
    static BufferedImage logo(){
        try(InputStream in=resource("assets/kyro_hud/textures/logo.png")){
            return in==null?null:ImageIO.read(in);
        }catch(IOException e){return null;}
    }
    static InputStream resource(String name)throws IOException{
        InputStream direct=PanelPainter.class.getResourceAsStream("/"+name);if(direct!=null)return direct;
        String runtime=System.getProperty("kyro.hud.runtime");if(runtime==null)return null;
        try(java.util.zip.ZipFile jar=new java.util.zip.ZipFile(runtime)){
            java.util.zip.ZipEntry entry=jar.getEntry(name);if(entry==null)return null;
            try(InputStream in=jar.getInputStream(entry)){ByteArrayOutputStream bytes=new ByteArrayOutputStream();byte[] buffer=new byte[8192];int n;while((n=in.read(buffer))>=0)bytes.write(buffer,0,n);return new ByteArrayInputStream(bytes.toByteArray());}
        }
    }
    static BufferedImage canvas(int w,int h){return new BufferedImage(w*SCALE,h*SCALE,BufferedImage.TYPE_INT_ARGB);}
    static Graphics2D graphics(BufferedImage image){
        Graphics2D g=image.createGraphics();g.scale(SCALE,SCALE);
        g.setRenderingHint(RenderingHints.KEY_ANTIALIASING,RenderingHints.VALUE_ANTIALIAS_ON);
        g.setRenderingHint(RenderingHints.KEY_TEXT_ANTIALIASING,RenderingHints.VALUE_TEXT_ANTIALIAS_ON);
        g.setRenderingHint(RenderingHints.KEY_FRACTIONALMETRICS,RenderingHints.VALUE_FRACTIONALMETRICS_ON);
        g.setRenderingHint(RenderingHints.KEY_INTERPOLATION,RenderingHints.VALUE_INTERPOLATION_BICUBIC);
        return g;
    }
    static void glass(Graphics2D g,double w,double h,double radius,boolean hot){
        Shape outline=new RoundRectangle2D.Double(.5,.5,w-1,h-1,radius*2,radius*2);
        g.setColor(light?new Color(247,245,250,242):new Color(20,20,23,222));g.fill(outline);
        g.setPaint(new GradientPaint(0,0,new Color(255,255,255,16),0,(float)(h*.6),new Color(255,255,255,0)));
        g.fill(outline);g.setColor(ink(hot?48:24));g.setStroke(new BasicStroke(.75f));g.draw(outline);
    }
    static BufferedImage info(String clock,String fps,String ping,double seconds,boolean hot){
        BufferedImage image=canvas(INFO_W,INFO_H);Graphics2D g=graphics(image);
        glass(g,INFO_W,INFO_H,12,hot);
        if(LOGO!=null)g.drawImage(LOGO,10,4,16,16,null);
        g.setFont(MEDIUM.deriveFont(10.5f));
        String name="KYRO";float nx=32;
        for(int i=0;i<name.length();i++){
            float wave=(float)(.67+.33*Math.sin(seconds*2.4-i*.65));
            g.setColor(new Color(ACCENT.getRed(),ACCENT.getGreen(),ACCENT.getBlue(),(int)(175+80*wave)));
            String ch=name.substring(i,i+1);g.drawString(ch,nx,15.5f);nx+=(float)g.getFont().getStringBounds(ch,g.getFontRenderContext()).getWidth()+.1f;
        }
        text(g,clock,73,7,28,9,WHITE,0,false);
        separator(g,108);text(g,fps,120,7,46,9,WHITE,0,false);
        separator(g,172);text(g,ping,183,7,36,9,QUIET,0,false);
        g.dispose();return image;
    }
    static void separator(Graphics2D g,float x){g.setColor(ink(35));g.fill(new Rectangle2D.Float(x,7, .7f,10));}
    static BufferedImage cooldowns(java.util.List<AgentCooldowns.Entry> entries){
        if(entries.isEmpty())return null;int count=Math.min(AgentCooldowns.MAX_ROWS,entries.size()),height=count*AgentCooldowns.ROW;BufferedImage image=canvas(AgentCooldowns.WIDTH,height);Graphics2D g=graphics(image);
        for(int i=0;i<count;i++){
            AgentCooldowns.Entry entry=entries.get(i);double y=i*AgentCooldowns.ROW;Graphics2D row=(Graphics2D)g.create();row.translate(0,y);glass(row,AgentCooldowns.WIDTH,18,7,false);
            text(row,entry.label,7,3,90,8.5f,WHITE,0,true);double seconds=Math.ceil(entry.remaining/2.0)/10.0;String time=seconds>=60?time(Math.ceil(seconds)):String.format(java.util.Locale.ROOT,"%.1f с",seconds);
            if(entry.estimated)time="≈"+time;text(row,time,101,3,35,8,ACCENT,0,true);row.setColor(ink(22));row.fill(new RoundRectangle2D.Double(7,15,126,1.2,2,2));row.setColor(ACCENT);row.fill(new RoundRectangle2D.Double(7,15,126*clamp((double)entry.remaining/entry.total),1.2,2,2));row.dispose();
        }g.dispose();return image;
    }
    static final class Skin {
        final BufferedImage cover,image;final double expand;final boolean hot;
        Skin(BufferedImage cover,double expand,boolean hot,BufferedImage image){this.cover=cover;this.expand=expand;this.hot=hot;this.image=image;}
    }
    static final ThreadLocal<Skin> SKIN=new ThreadLocal<>();
    static BufferedImage background(BufferedImage cover,double a,boolean hot){
        Skin cached=SKIN.get();if(cached!=null&&cached.cover==cover&&cached.expand==a&&cached.hot==hot)return cached.image;
        double h=STRIP_H+(CARD_H-STRIP_H)*a,r=12+2*a;
        BufferedImage image=canvas(MUSIC_W,CARD_H);Graphics2D g=graphics(image);
        glass(g,MUSIC_W,h,r,hot);
        Shape block=new RoundRectangle2D.Double(1,1,MUSIC_W-2,h-2,r*2,r*2);g.clip(block);
        Color tint=cover==null?ACCENT:average(cover);
        if(cover!=null){
            g.setPaint(new RadialGradientPaint(new Point2D.Double(36,22),180,new float[]{0,1},new Color[]{new Color(tint.getRed(),tint.getGreen(),tint.getBlue(),68),new Color(0,0,0,0)}));
            g.fill(block);
        }
        double cx=lerp(6,12,a),cy=lerp(6,12,a),cs=lerp(36,48,a);
        Shape before=g.getClip();g.clip(new RoundRectangle2D.Double(cx,cy,cs,cs,18,18));
        if(cover!=null)g.drawImage(cover,(int)cx,(int)cy,(int)cs,(int)cs,null);
        else {g.setColor(ink(14));g.fill(new Rectangle2D.Double(cx,cy,cs,cs));if(LOGO!=null)g.drawImage(LOGO,(int)cx+4,(int)cy+4,(int)cs-8,(int)cs-8,null);}
        g.setClip(before);
        g.dispose();SKIN.set(new Skin(cover,a,hot,image));return image;
    }
    static BufferedImage music(Track t,double expand,boolean hot,double seconds,double seekPreview,boolean remaining){
        double a=Math.round(clamp(expand)*120)/120.0,h=STRIP_H+(CARD_H-STRIP_H)*a,r=12+2*a;
        BufferedImage image=canvas(MUSIC_W,CARD_H);image.getRaster().setRect(background(t.cover,a,hot).getRaster());
        Graphics2D g=graphics(image);g.clip(new RoundRectangle2D.Double(1,1,MUSIC_W-2,h-2,r*2,r*2));
        if(!t.available){
            text(g,"Музыка",50,9,190,10.5f,WHITE,0,false);
            text(g,t.error.trim().isEmpty()?"Включи трек в браузере":t.error,50,27,192,8,QUIET,seconds,false);
            bars(g,250,17,seconds,false);g.dispose();return image;
        }
        double tx=lerp(50,70,a),ty=lerp(9,13,a),tw=lerp(122,190,a);
        String title=a<.5?(t.artist.trim().isEmpty()?t.title:t.title+" · "+t.artist):t.title;
        text(g,title,tx,ty,tw,(float)lerp(10.5,12,a),WHITE,seconds,true);
        if(a>.5&&!t.artist.trim().isEmpty())text(g,t.artist,70,31,190,9.5f,QUIET,seconds,true);
        text(g,service(t.source),tx,lerp(27,46,a),tw,(float)lerp(8,8.5,a),QUIET,seconds,false);
        double px=lerp(178,92,a),py=lerp(14,83,a),playX=lerp(208,122,a),playY=lerp(10,79,a),nextX=lerp(224,138,a);
        wing(g,px,py,true,t.canPrevious,hot);wing(g,nextX,py,false,t.canNext,hot);
        play(g,playX,playY,t.playing,t.canToggle,hot,seconds);
        double progress=seekPreview>=0?clamp(seekPreview):t.progress();
        double bx=12,by=lerp(45,66,a),bw=248,bh=lerp(1.5,3,a);
        if(t.duration>0){
            g.setColor(ink(32));g.fill(new RoundRectangle2D.Double(bx,by,bw,bh,4,4));
            g.setColor(ACCENT);g.fill(new RoundRectangle2D.Double(bx,by,bw*progress,bh,4,4));
            if(a>.6){
                double pos=seekPreview>=0?progress*t.duration:t.position;
                text(g,time(pos),12,75,44,8,QUIET,0,false);
                String duration=remaining?"−"+time(Math.max(0,t.duration-pos)):time(t.duration);
                text(g,duration,216,75,44,8,QUIET,0,false);
                if(hot&&t.canSeek){g.setColor(WHITE);g.fill(new Ellipse2D.Double(bx+bw*progress-3,by-1.5,6,6));}
            }
        }else if(a>.6){text(g,"● LIVE",12,72,120,8,ACCENT,0,false);}
        if(a>.9&&t.volume>=0){
            text(g,"VOL",184,94,22,7,QUIET,0,false);
            g.setColor(ink(30));g.fill(new RoundRectangle2D.Double(207,97,53,3,3,3));
            g.setColor(ACCENT);g.fill(new RoundRectangle2D.Double(207,97,53*clamp(t.volume),3,3,3));
        }
        g.dispose();return image;
    }
    static void wing(Graphics2D g,double x,double y,boolean previous,boolean enabled,boolean hot){
        g.setColor(ink(enabled?(hot?45:24):9));g.fill(new RoundRectangle2D.Double(x,y,42,20,20,20));
        g.setColor(enabled?WHITE:QUIET);
        double ix=previous?x+12:x+28,iy=y+10,dir=previous?-1:1;
        Path2D arrow=new Path2D.Double();arrow.moveTo(ix+dir*3,iy);arrow.lineTo(ix-dir*2,iy-3.5);arrow.lineTo(ix-dir*2,iy+3.5);arrow.closePath();g.fill(arrow);
        g.fill(new Rectangle2D.Double(previous?ix-5:ix+4,iy-3.5,1.4,7));
    }
    static void play(Graphics2D g,double x,double y,boolean playing,boolean enabled,boolean hot,double seconds){
        g.setColor(enabled?ACCENT:new Color(120,110,130));g.fill(new Ellipse2D.Double(x,y,28,28));
        g.setPaint(new GradientPaint((float)x,(float)y,new Color(255,255,255,hot?65:35),(float)x,(float)(y+28),new Color(255,255,255,0)));g.fill(new Ellipse2D.Double(x,y,28,28));
        boolean pale=ACCENT.getRed()*.299+ACCENT.getGreen()*.587+ACCENT.getBlue()*.114>170;
        g.setColor(enabled?(pale?new Color(33,27,43):Color.WHITE):new Color(180,168,185));
        if(playing){g.fill(new RoundRectangle2D.Double(x+10,y+9,2.5,10,1,1));g.fill(new RoundRectangle2D.Double(x+15.5,y+9,2.5,10,1,1));}
        else{Path2D path=new Path2D.Double();path.moveTo(x+11,y+8.5);path.lineTo(x+20,y+14);path.lineTo(x+11,y+19.5);path.closePath();g.fill(path);}
    }
    static void text(Graphics2D g,String value,double x,double y,double width,float size,Color color,double seconds,boolean medium){
        if(value==null||width<=0)return;
        g.setFont((medium?MEDIUM:REGULAR).deriveFont(size));g.setColor(color);
        Shape old=g.getClip();g.clip(new Rectangle2D.Double(x,y,width,size+5));
        double full=g.getFont().getStringBounds(value,g.getFontRenderContext()).getWidth();
        double offset=0;
        if(full>width){double run=full+24;double elapsed=Math.max(0,seconds-1.3);offset=(elapsed*20)%(run+26);offset=Math.min(offset,run);}
        g.drawString(value,(float)(x-offset),(float)(y+size));
        if(full>width)g.drawString(value,(float)(x-offset+full+24),(float)(y+size));
        g.setClip(old);
    }
    static void bars(Graphics2D g,int x,int y,double seconds,boolean playing){
        g.setColor(ACCENT);for(int i=0;i<4;i++){double h=playing?3+9*Math.abs(Math.sin(seconds*3+i)):3;g.fill(new RoundRectangle2D.Double(x+i*3,y+7-h/2,1.5,h,1.5,1.5));}
    }
    static String service(String source){
        String s=source.toLowerCase(java.util.Locale.ROOT);
        if(s.contains("spotify"))return "Spotify";if(s.contains("msedge"))return "Microsoft Edge";
        if(s.contains("chrome"))return "Google Chrome";if(s.contains("firefox"))return "Firefox";
        if(s.contains("yandex"))return "Яндекс Музыка";if(s.contains("vlc"))return "VLC";
        if(s.contains("brave"))return "Brave";if(s.contains("opera"))return "Opera";
        if(s.contains("mpv"))return "mpv";return "KYRO · Медиаплеер";
    }
    static Color average(BufferedImage image){long r=0,g=0,b=0;int n=0;for(int y=0;y<image.getHeight();y+=Math.max(1,image.getHeight()/8))for(int x=0;x<image.getWidth();x+=Math.max(1,image.getWidth()/8)){int c=image.getRGB(x,y);r+=(c>>16)&255;g+=(c>>8)&255;b+=c&255;n++;}return new Color((int)(r/n),(int)(g/n),(int)(b/n));}
    static String time(double seconds){long s=(long)Math.max(0,seconds);return s>=3600?String.format(java.util.Locale.ROOT,"%d:%02d:%02d",s/3600,s/60%60,s%60):String.format(java.util.Locale.ROOT,"%d:%02d",s/60,s%60);}
    static double lerp(double a,double b,double t){return a+(b-a)*t;}
    static double clamp(double value){return Double.isFinite(value)?Math.max(0,Math.min(1,value)):0;}
}
