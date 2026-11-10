package ru.kyro.hud;

import java.lang.reflect.*;

/** Changes visual environment colours only. No world time, light or fog-distance edits. */
public final class SkyHooks {
    static volatile int themeColor=0xa185cf;
    static volatile boolean bound;static Object sky,fog,cloud,sunset;
    static final boolean TRACE=Boolean.getBoolean("kyro.hud.skyTrace");static int traced;
    public static String probeClass(){return BadgeHooks.mapped("skyProbeClass").replace('.','/');}
    public static String valueMethod(){return BadgeHooks.mapped("skyValueMethod");}
    static Object field(Class<?> type,String key,String...names){
        for(String name:names)try{return type.getField(name).get(null);}catch(Exception ignored){}
        String mapped=BadgeHooks.mapped(key);if(!mapped.isEmpty())try{return type.getField(mapped).get(null);}catch(Exception ignored){}return null;
    }
    static synchronized void bind(ClassLoader loader){
        if(bound)return;
        for(String name:new String[]{"net.minecraft.world.attribute.EnvironmentAttributes","net.minecraft.class_12206",BadgeHooks.mapped("skyAttributesClass")})if(!name.isEmpty())try{
            Class<?> type=Class.forName(name,true,loader);
            sky=field(type,"skyAttributeField","SKY_COLOR","field_63746");fog=field(type,"skyFogAttributeField","FOG_COLOR","field_63742");cloud=field(type,"skyCloudAttributeField","CLOUD_COLOR","field_64126");sunset=field(type,"skySunsetAttributeField","SUNRISE_SUNSET_COLOR","field_64339");bound=true;return;
        }catch(Throwable ignored){}
        bound=true;
    }
    static int blend(int original,int color,double amount){
        double mix=PanelLayout.strength(amount);if(mix<=0)return original;int out=original&0xff000000;
        for(int shift=0;shift<=16;shift+=8){int a=original>>>shift&255,b=color>>>shift&255;out|=(int)Math.round(a+(b-a)*mix)<<shift;}return out;
    }
    static int color(int value,int kind,PanelLayout settings){
        if(settings==null||!settings.customSky)return value;
        int chosen=settings.skyUseThemeColor?themeColor:settings.skyColor;
        if(kind==0)return blend(value,chosen,settings.skyMix);
        if(kind==1&&settings.skyFog)return blend(value,settings.skyFogFollow?chosen:settings.skyFogColor,settings.skyFogFollow?settings.skyMix:settings.skyFogMix);
        if(kind==2&&settings.skyClouds)return blend(value,settings.skyCloudColor,settings.skyCloudMix);
        if(kind==3&&settings.skyNoSunset)return value&0x00ffffff;
        return value;
    }
    public static Object apply(Object original,Object attribute){
        PanelLayout settings=AgentHooks.layout;if(settings==null||!settings.customSky||!(original instanceof Integer)||attribute==null)return original;
        try{if(!bound)bind(attribute.getClass().getClassLoader());int kind=attribute==sky?0:attribute==fog?1:attribute==cloud?2:attribute==sunset?3:-1;if(kind<0)return original;int chosen=color((Integer)original,kind,settings);if(TRACE&&chosen!=(Integer)original&&(traced&(1<<kind))==0){traced|=1<<kind;System.out.println("[KYRO HUD] Custom Sky applied "+new String[]{"SKY","FOG","CLOUD","SUNSET"}[kind]);}return Integer.valueOf(chosen);}catch(Throwable ignored){return original;}
    }
}
