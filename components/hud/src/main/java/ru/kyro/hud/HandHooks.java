package ru.kyro.hud;
import com.google.gson.*;
import java.lang.reflect.*;
import java.util.*;
/** First-person hand transforms, inside the game's own push/pop scope. */
public final class HandHooks {
    public static String alias(String group,String key){try{return BadgeHooks.MAPPINGS.getAsJsonObject(group).get(key).getAsString().replace('.','/');}catch(Exception ignored){return "";}}
    public static String[] methods(String key){try{JsonArray a=BadgeHooks.MAPPINGS.getAsJsonObject("moduleMembers").getAsJsonArray(key);String[] out=new String[a.size()];for(int i=0;i<out.length;i++)out[i]=a.get(i).getAsString();return out;}catch(Exception ignored){return new String[0];}}
    static boolean named(String name,String key,String...defaults){for(String n:defaults)if(name.equals(n))return true;for(String n:methods(key))if(name.equals(n))return true;return false;}
    static final Map<Class<?>,Method> scales=new java.util.concurrent.ConcurrentHashMap<>(),pushes=new java.util.concurrent.ConcurrentHashMap<>(),pops=new java.util.concurrent.ConcurrentHashMap<>();
    static final java.util.Set<Boolean> traced=java.util.Collections.synchronizedSet(new java.util.HashSet<Boolean>());
    static boolean previewReady(){
        Object client=AgentHooks.game.clientObject();if(client==null)return false;
        for(String field:new String[]{"screen","currentScreen","field_1755","overlay","field_18175"})if(GameAccess.field(client,field)!=null)return false;
        for(String key:new String[]{"screen","overlay"})try{String field=BadgeHooks.MAPPINGS.getAsJsonObject("moduleFields").getAsJsonObject("net.minecraft.client.Minecraft#"+key).get("name").getAsString();if(GameAccess.field(client,field)!=null)return false;}catch(Exception ignored){}
        return true;
    }
    static void stack(Object pose,boolean push)throws Exception{
        Map<Class<?>,Method> cache=push?pushes:pops;Method found=cache.get(pose.getClass());
        if(found==null)for(Method m:pose.getClass().getMethods())if(m.getParameterTypes().length==0&&named(m.getName(),"com.mojang.blaze3d.vertex.PoseStack#"+(push?"pushPose":"popPose"),push?"pushPose":"popPose",push?"push":"pop",push?"method_22903":"method_22909")){found=m;cache.put(pose.getClass(),m);break;}
        if(found==null)throw new NoSuchMethodException("PoseStack push/pop");found.invoke(pose);
    }
    static final Object NO_POSE=new Object();
    static final ThreadLocal<Deque<Object>> ACTIVE=ThreadLocal.withInitial(ArrayDeque::new);
    static void scope(Object pose){try{stack(pose,true);ACTIVE.get().push(pose);}catch(Throwable error){ACTIVE.get().push(NO_POSE);AgentHooks.log("Hand size scope failed",error);}}
    public static void beginSide(Object pose,boolean right){scope(pose);try{if(ACTIVE.get().peek()!=NO_POSE)scale(pose,right);}catch(Throwable error){AgentHooks.log("Hand size transform failed",error);}}
    public static void beginContext(Object pose,Object context){scope(pose);try{if(context instanceof Enum&&ACTIVE.get().peek()!=NO_POSE){int value=((Enum<?>)context).ordinal();if(value==3||value==4)scale(pose,value==4);}}catch(Throwable error){AgentHooks.log("Hand size transform failed",error);}}
    public static void end(){Deque<Object> scopes=ACTIVE.get();if(scopes.isEmpty())return;Object pose=scopes.pop();try{if(pose!=NO_POSE)stack(pose,false);}catch(Throwable error){AgentHooks.log("Hand size restore failed",error);}}
    static void scale(Object pose,boolean right)throws Exception{PanelLayout settings=AgentHooks.layout;if(settings==null)return;
            float value=(float)PanelLayout.handScale(right?settings.rightHandScale:settings.leftHandScale);if(value==1)return;
            Method scale=scales.get(pose.getClass());if(scale==null){for(Method m:pose.getClass().getMethods())if(Arrays.equals(m.getParameterTypes(),new Class<?>[]{float.class,float.class,float.class})&&named(m.getName(),"com.mojang.blaze3d.vertex.PoseStack#scale","scale","method_22905")){scale=m;scales.put(pose.getClass(),m);break;}}
            if(scale==null)throw new NoSuchMethodException("PoseStack scale");
            // Post-multiplication after positioning leaves the grip origin in place.
            scale.invoke(pose,value,value,value);
            if(Boolean.getBoolean("kyro.hud.handTrace")&&traced.add(right))System.out.println("[KYRO HUD] Hand size applied: "+(right?"RIGHT":"LEFT")+" "+value);
    }

}
