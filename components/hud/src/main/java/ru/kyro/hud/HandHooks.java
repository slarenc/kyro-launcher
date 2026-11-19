package ru.kyro.hud;
import com.google.gson.*;
import java.lang.reflect.*;
import java.util.*;
/** First-person hand transforms, inside the game's own push/pop scope. */
public final class HandHooks {
    public static String alias(String group,String key){try{return BadgeHooks.MAPPINGS.getAsJsonObject(group).get(key).getAsString().replace('.','/');}catch(Exception ignored){return "";}}
    public static String[] methods(String key){try{JsonArray a=BadgeHooks.MAPPINGS.getAsJsonObject("moduleMembers").getAsJsonArray(key);String[] out=new String[a.size()];for(int i=0;i<out.length;i++)out[i]=a.get(i).getAsString();return out;}catch(Exception ignored){return new String[0];}}
    static boolean named(String name,String key,String...defaults){for(String n:defaults)if(name.equals(n))return true;for(String n:methods(key))if(name.equals(n))return true;return false;}
    static final Map<Class<?>,Method> scales=new java.util.concurrent.ConcurrentHashMap<>();
    public static void apply(Object pose,Object player,Object hand){
        PanelLayout settings=AgentHooks.layout;if(settings==null||pose==null||player==null||!(hand instanceof Enum))return;
        if(settings.rightHandScale==1&&settings.leftHandScale==1)return;
        try{
            Object arm=null;for(Method m:player.getClass().getMethods())if(m.getParameterTypes().length==0&&named(m.getName(),"net.minecraft.world.entity.LivingEntity#getMainArm","getMainArm","getMainHand","method_6068")){arm=m.invoke(player);break;}
            if(!(arm instanceof Enum))return;
            // HumanoidArm: LEFT, RIGHT; InteractionHand: MAIN_HAND, OFF_HAND.
            boolean right=((Enum<?>)arm).ordinal()==1;if(((Enum<?>)hand).ordinal()==1)right=!right;
            float value=(float)PanelLayout.handScale(right?settings.rightHandScale:settings.leftHandScale);if(value==1)return;
            Method scale=scales.get(pose.getClass());if(scale==null){for(Method m:pose.getClass().getMethods())if(Arrays.equals(m.getParameterTypes(),new Class<?>[]{float.class,float.class,float.class})&&named(m.getName(),"com.mojang.blaze3d.vertex.PoseStack#scale","scale","method_22905")){scale=m;scales.put(pose.getClass(),m);break;}}
            if(scale!=null)scale.invoke(pose,value,value,value);
        }catch(Throwable error){AgentHooks.log("Hand size transform failed",error);}
    }
}
