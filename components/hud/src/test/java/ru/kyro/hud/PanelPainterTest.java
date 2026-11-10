package ru.kyro.hud;

import java.awt.*;
import java.awt.image.BufferedImage;
import java.nio.file.*;
import javax.imageio.ImageIO;

public final class PanelPainterTest {
    static void check(boolean ok,String message){if(!ok)throw new AssertionError(message);}
    public static void main(String[] args)throws Exception{
        check(PanelPainter.LOGO!=null,"KYRO logo is bundled");
        check(!PanelPainter.REGULAR.getFamily().equals(Font.SANS_SERIF),"HUD font loaded");
        check(PanelPainter.time(3661).equals("1:01:01"),"hour-long tracks");
        check(PanelPainter.time(-1).equals("0:00"),"negative timestamps");
        check(MediaBridge.decode("not valid base64")==null,"bad art does not break the HUD");
        try(MediaBridge bridge=new MediaBridge(Paths.get("unused-qa-helper.exe"))){
            MediaBridge.State state=new MediaBridge.State();state.available=true;state.source="QA player";state.volume=.8;bridge.state=state;
            bridge.command("volume",bridge.snapshot().volume()-.05);bridge.command("volume",bridge.snapshot().volume()-.05);
            check(Math.abs(bridge.snapshot().volume()-.7)<.001,"rapid volume steps accumulate before a Windows reply");
            MediaBridge.State other=new MediaBridge.State();other.source="Other player";other.volume=.4;bridge.state=other;
            check(bridge.snapshot().volume()==.4,"pending volume does not leak across sources");
        }
        BufferedImage art=new BufferedImage(180,180,BufferedImage.TYPE_INT_ARGB);Graphics2D ag=art.createGraphics();
        ag.setPaint(new GradientPaint(0,0,new Color(84,51,116),180,180,new Color(222,133,168)));ag.fillRect(0,0,180,180);ag.drawImage(PanelPainter.LOGO,20,20,140,140,null);ag.dispose();
        PanelPainter.Track track=new PanelPainter.Track(true,true,true,true,true,true,"Ночная орбита · Night orbit","KYRO Radio","Spotify.exe","",83,245,.64,art);
        BufferedImage info=PanelPainter.info("12:30","404 FPS","34 ms",2,false);
        BufferedImage strip=PanelPainter.music(track,0,false,2,-1,false);
        BufferedImage expanded=PanelPainter.music(track,1,true,2,-1,true);
        check((info.getRGB(0,0)>>>24)==0,"transparent rounded island corners");
        check((strip.getRGB(10,150)>>>24)==0,"collapsed player leaves transparent space");
        check((expanded.getRGB(200,150)>>>24)>0,"expanded player paints its card");
        PanelPainter.Track idle=new PanelPainter.Track(false,false,false,false,false,false,"","","","",0,0,-1,null);
        PanelPainter.music(idle,0,false,0,-1,false);
        PanelPainter.Track stream=new PanelPainter.Track(true,false,false,false,false,false,"Прямой эфир","","browser","",0,0,-1,null);
        PanelPainter.music(stream,1,true,2,-1,false);
        PanelPainter.appearance("light","#52c7c3");
        BufferedImage lightInfo=PanelPainter.info("12:30","404 FPS","34 ms",2,false),lightMusic=PanelPainter.music(track,1,true,2,-1,true);
        check(new Color(lightInfo.getRGB(220,22),true).getRed()>220,"light theme uses a white island");
        check(PanelPainter.WHITE.getRed()<60&&PanelPainter.ACCENT.equals(Color.decode("#52c7c3")),"light text contrast and launcher accent");
        check(new Color(lightMusic.getRGB(530,210),true).getRed()>200,"music skin cache changes with the theme");
        Path themes=Paths.get("build/reports/panels");Files.createDirectories(themes);ImageIO.write(lightInfo,"png",themes.resolve("island-light.png").toFile());ImageIO.write(lightMusic,"png",themes.resolve("music-light.png").toFile());
        PanelPainter.appearance("black","#a185cf");
        BufferedImage preview=new BufferedImage(1200,440,BufferedImage.TYPE_INT_ARGB);Graphics2D g=preview.createGraphics();
        g.setColor(new Color(38,38,40));g.fillRect(0,0,1200,440);g.setFont(new Font(Font.SANS_SERIF,Font.PLAIN,16));g.setColor(new Color(170,165,179));
        g.drawString("KYRO / IN GAME",30,30);g.drawString("KYRO / CURSOR FREE",620,30);
        g.drawImage(info,30,55,null);g.drawImage(info,620,55,null);g.drawImage(strip,30,160,null);g.drawImage(expanded,620,140,null);
        g.drawString("Right Shift: move panels · Chat / inventory: player controls",30,410);g.dispose();
        Path out=Paths.get("build/reports/panels");Files.createDirectories(out);ImageIO.write(preview,"png",out.resolve("preview.png").toFile());
        ImageIO.write(info,"png",out.resolve("island.png").toFile());ImageIO.write(strip.getSubimage(0,0,544,96),"png",out.resolve("music.png").toFile());
        long start=System.nanoTime();for(int i=0;i<30;i++)PanelPainter.music(track,1,true,i/30.0,-1,false);
        System.out.printf(java.util.Locale.ROOT,"KYRO painter: assets, transparency, idle/live/expanded states passed; %.2f ms/frame%n",(System.nanoTime()-start)/30e6);
    }
}
