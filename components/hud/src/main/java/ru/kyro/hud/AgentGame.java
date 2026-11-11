package ru.kyro.hud;

import java.lang.reflect.*;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.util.*;
import com.google.gson.*;

/** Known named/intermediary aliases plus structural fallback for obfuscated vanilla clients. */
final class AgentGame {
    Object clientObject(){return client;}
    private Object client;private ClassLoader loader;private List<Field> players;Object localPlayer;
    private String clientClass="",playerField="",latencyField="",optionsName="",hideName="";
    private Field optionsField,hiddenField;private long optionsRetry;
    private long retryAt;boolean world;String ping="— ms";
    AgentGame(){
        try{Path file=Paths.get(System.getProperty("kyro.hud.config","config")).resolve("kyro-hud-mapping.json");JsonObject map=new Gson().fromJson(new String(Files.readAllBytes(file),StandardCharsets.UTF_8),JsonObject.class);
            clientClass=string(map,"clientClass");playerField=string(map,"playerField");latencyField=string(map,"latencyField");
            optionsName=string(map,"optionsField");hideName=string(map,"hideField");
        }catch(Exception ignored){}
    }
    static String string(JsonObject json,String key){return json.has(key)?json.get(key).getAsString():"";}
    boolean hidden(){
        if(client==null)return false;
        try{
            if(optionsField!=null&&hiddenField!=null)return hiddenField.getBoolean(optionsField.get(client));
            if(System.nanoTime()<optionsRetry)return false;optionsRetry=System.nanoTime()+2_000_000_000L;
            for(String name:new String[]{optionsName,"options","gameSettings","field_71474_y","field_1690"})try{
                if(name.isEmpty())continue;Field f=client.getClass().getDeclaredField(name);f.setAccessible(true);Object options=f.get(client);if(options==null)continue;
                for(String hidden:new String[]{hideName,"hideGui","hideGUI","hudHidden","field_74319_N","field_1842"})try{if(hidden.isEmpty())continue;Field flag=options.getClass().getDeclaredField(hidden);if(flag.getType()!=boolean.class)continue;flag.setAccessible(true);optionsField=f;hiddenField=flag;return flag.getBoolean(options);}catch(Exception ignored){}
            }catch(Exception ignored){}
        }catch(Exception ignored){}return false;
    }
    void update(ClassLoader gameLoader){
        try{
            if(client==null){if(System.nanoTime()<retryAt)return;retryAt=System.nanoTime()+2_000_000_000L;loader=gameLoader;findClient();}
            if(client==null){world=false;return;}
            Object player=null;
            for(Field f:players){Object value=f.get(client);if(value!=null){player=value;break;}}
            localPlayer=player;world=player!=null;ping="— ms";if(!world){if(AgentHooks.presence!=null)AgentHooks.presence.game(false,null,null);return;}
            Object connection=call(client,new String[]{"getConnection","getNetworkHandler","getNetHandler","func_147114_u","method_1562",BadgeHooks.mapped("clientConnectionMethod")});
            if(connection==null){for(Method m:client.getClass().getDeclaredMethods())if(m.getParameterTypes().length==0&&!m.getReturnType().isPrimitive()&&hasLookup(m.getReturnType())){m.setAccessible(true);connection=m.invoke(client);if(connection!=null)break;}}
            Object uuid=call(player,new String[]{"getUUID","getUuid","getUniqueID","func_110124_au","method_5667"});
            if(!(uuid instanceof UUID))for(Method m:player.getClass().getMethods())if(m.getParameterTypes().length==0&&m.getReturnType()==UUID.class){uuid=m.invoke(player);break;}
            if(AgentHooks.presence!=null)AgentHooks.presence.game(true,connection,uuid);
            Object single=call(client,new String[]{"isInSingleplayer","isLocalServer","func_71356_B","method_1542"});if(Boolean.TRUE.equals(single)){ping="LOCAL";return;}
            if(!(uuid instanceof UUID))return;
            if(connection==null)return;
            Object entry=null;
            for(Method m:connection.getClass().getMethods())if(Arrays.equals(m.getParameterTypes(),new Class<?>[]{UUID.class})&&!m.getReturnType().isPrimitive()){entry=m.invoke(connection,uuid);if(entry!=null)break;}
            if(entry==null)return;
            Object latency=call(entry,new String[]{"getLatency","getResponseTime","func_178853_c","method_2959"});
            if(!(latency instanceof Number))for(String name:new String[]{latencyField,"latency","responseTime","field_178873_i","field_3747"})try{if(!name.isEmpty()){Field f=entry.getClass().getDeclaredField(name);f.setAccessible(true);latency=f.get(entry);break;}}catch(Exception ignored){}
            if(latency instanceof Number){int ms=((Number)latency).intValue();if(ms>=0&&ms<60000)ping=ms+" ms";}
        }catch(Throwable ignored){world=client!=null&&world;}
    }
    private void findClient(){
        for(String name:new String[]{clientClass,"net.minecraft.client.Minecraft","net.minecraft.client.MinecraftClient","net.minecraft.class_310"}){
            if(name.isEmpty())continue;try{if(bind(Class.forName(name,false,loader)))return;}catch(Throwable ignored){}
        }
        for(StackTraceElement frame:Thread.currentThread().getStackTrace()){
            String name=frame.getClassName();if(name.startsWith("java.")||name.startsWith("org.lwjgl.")||name.startsWith("ru.kyro.")||name.startsWith("com.mojang."))continue;
            try{if(bind(Class.forName(name,false,loader)))return;}catch(Throwable ignored){}
        }
    }
    private boolean bind(Class<?> candidate)throws Exception{
        Object found=null;
        for(Method m:candidate.getMethods())if(Modifier.isPublic(m.getModifiers())&&Modifier.isStatic(m.getModifiers())&&m.getParameterTypes().length==0&&m.getReturnType()==candidate){try{found=m.invoke(null);if(found!=null)break;}catch(Exception ignored){}}
        if(found==null)for(Field f:candidate.getDeclaredFields())if(Modifier.isStatic(f.getModifiers())&&f.getType()==candidate){try{f.setAccessible(true);found=f.get(null);if(found!=null)break;}catch(Exception ignored){}}
        if(found==null)return false;
        List<Field> entities=new ArrayList<>();
        for(Field f:candidate.getDeclaredFields())if(!Modifier.isStatic(f.getModifiers())&&!f.getType().isPrimitive()&&entity(f.getType())){try{f.setAccessible(true);entities.add(f);}catch(Exception ignored){}}
        if(entities.isEmpty())return false;
        Collections.sort(entities,(a,b)->Integer.compare(score(b),score(a)));client=found;players=entities;
        System.out.println("[KYRO HUD] Game client: "+candidate.getName());return true;
    }
    private int score(Field f){if(f.getName().equals(playerField)||Arrays.asList("player","thePlayer","field_71439_g","field_1724").contains(f.getName()))return 1000;int depth=0;for(Class<?> c=f.getType();c!=null;c=c.getSuperclass())depth++;return depth;}
    static boolean entity(Class<?> type){String name=type.getName();if(name.startsWith("java.")||name.startsWith("com.mojang."))return false;try{
        int coordinates=0;for(Class<?> c=type;c!=null;c=c.getSuperclass())for(Field f:c.getDeclaredFields())if(!Modifier.isStatic(f.getModifiers())&&f.getType()==double.class)coordinates++;
        if(coordinates<3)return false;for(Method m:type.getMethods())if(m.getParameterTypes().length==0&&m.getReturnType()==UUID.class)return true;
    }catch(Throwable ignored){}return false;}
    static boolean hasLookup(Class<?> type){try{for(Method m:type.getMethods())if(Arrays.equals(m.getParameterTypes(),new Class<?>[]{UUID.class})&&!m.getReturnType().isPrimitive())return true;}catch(Throwable ignored){}return false;}
    static Object call(Object target,String[] names){for(String name:names)try{Method m=target.getClass().getMethod(name);return m.invoke(target);}catch(Exception ignored){}return null;}
}
