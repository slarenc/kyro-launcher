package ru.kyro.hudtest;
import java.lang.reflect.*;
import net.minecraft.world.attribute.*;
/** Exercises the distributed ASM hook through an actual method call. */
public final class SkyHarness {
 static void check(boolean ok,String message){if(!ok)throw new AssertionError(message);}
 public static void main(String[] args)throws Exception{
  Class<?> hooks=Class.forName("ru.kyro.builtin.AgentHooks",true,null);Field field=hooks.getDeclaredField("layout");field.setAccessible(true);Object settings=field.get(null);Class<?> type=settings.getClass();
  type.getField("customSky").setBoolean(settings,false);for(String flag:new String[]{"skyFog","skyFogFollow","skyNoSunset"})type.getField(flag).setBoolean(settings,true);
  EnvironmentAttributeProbe probe=new EnvironmentAttributeProbe();Object original=probe.getValue(EnvironmentAttributes.SKY_COLOR,.5f);check(original.equals(0xff336699),"Disabled sky preserves the exact colour");
  type.getField("customSky").setBoolean(settings,true);type.getField("skyUseThemeColor").setBoolean(settings,false);type.getField("skyColor").setInt(settings,0xaa44ee);type.getField("skyMix").setDouble(settings,1);
  check(probe.getValue(EnvironmentAttributes.SKY_COLOR,0f).equals(0xffaa44ee),"Real transformed method returns chosen sky colour");
  check(probe.getValue(EnvironmentAttributes.FOG_COLOR,0f).equals(0x88aa44ee),"Fog follows sky and preserves alpha");
  check(probe.getValue(EnvironmentAttributes.SUNRISE_SUNSET_COLOR,0f).equals(0x00ff8800),"Sunset removed via alpha only");
  check(probe.getValue(EnvironmentAttributes.WATER_FOG_COLOR,0f)==EnvironmentAttributes.WATER_FOG_COLOR.value,"Underwater fog untouched");
  check(probe.getValue(EnvironmentAttributes.FOG_END_DISTANCE,0f)==EnvironmentAttributes.FOG_END_DISTANCE.value,"Fog distance untouched");
  type.getField("skyFog").setBoolean(settings,false);check(probe.getValue(EnvironmentAttributes.FOG_COLOR,0f).equals(0x88336699),"Fog toggle restores original colour");
  type.getField("customSky").setBoolean(settings,false);check(probe.getValue(EnvironmentAttributes.SKY_COLOR,0f).equals(original),"Master switch restores original sky immediately");
  System.out.println("PASS: production sky transformer, selected colours, alpha, fog toggle, sunset, untouched underwater fog/distance and immediate disable");
 }
}
