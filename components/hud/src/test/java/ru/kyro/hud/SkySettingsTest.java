package ru.kyro.hud;
import java.nio.file.*;
import com.google.gson.Gson;
public final class SkySettingsTest {
 static void check(boolean ok,String message){if(!ok)throw new AssertionError(message);}
 public static void main(String[] args)throws Exception{
  PanelLayout settings=new PanelLayout();settings.customSky=true;settings.skyUseThemeColor=false;settings.skyColor=0xabcdef;settings.skyMix=1;
  check(SkyHooks.color(0x88112233,0,settings)==0x88abcdef,"RGB replacement preserves alpha");settings.skyMix=.5;check(SkyHooks.color(0xff000000,0,settings)==0xff566778,"Half strength");settings.skyMix=0;check(SkyHooks.color(0xff112233,0,settings)==0xff112233,"Zero strength unchanged");
  Path dir=Files.createTempDirectory("kyro-sky"),file=dir.resolve("kyro-panels.json");AgentHooks.config=dir;AgentHooks.layout=settings;AgentHooks.skyColorEdit=0;
  check(AgentHooks.pasteSkyHex(" #7b4CE1 ")&&settings.skyColor==0x7b4ce1&&!settings.skyUseThemeColor&&settings.customSky,"Paste HEX and immediate activation");
  check(!AgentHooks.pasteSkyHex("invalid")&&settings.skyColor==0x7b4ce1,"Invalid paste preserves colour");AgentHooks.skyColorEdit=1;check(AgentHooks.pasteSkyHex("0x123456")&&settings.skyFogColor==0x123456&&!settings.skyFogFollow,"Independent fog HEX");
  AgentHooks.skyColorEdit=2;AgentHooks.pasteSkyHex("aabbcc");check(settings.skyCloudColor==0xaabbcc,"Cloud HEX without prefix");
  settings.save(file,new Gson());PanelLayout loaded=PanelLayout.load(file,new Gson());check(loaded.skyColor==0x7b4ce1&&loaded.skyCloudColor==0xaabbcc&&loaded.customSky,"Saved settings");
  AgentHooks.skyColorEdit=-1;Path report=Paths.get("build/reports/sky");Files.createDirectories(report);javax.imageio.ImageIO.write(AgentHooks.skyImage(),"png",report.resolve("settings.png").toFile());
  Files.delete(file);Files.delete(dir);System.out.println("PASS: RGB/alpha/strength, validated HEX paste, independent colours, persistence and settings rendering");
 }
}
