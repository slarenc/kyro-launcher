package ru.kyro.hudtest;
import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;
/** Reads actual Minecraft classes and components using the distributed runtime. */
public final class ActualCooldownHarness {
    static Properties names=new Properties();
    static Class<?> clazz(String key)throws Exception{return Class.forName(names.getProperty(key));}
    static Object invoke(Object target,String key,Object...args)throws Exception{
        Class<?> type=target instanceof Class?(Class<?>)target:target.getClass();for(Method m:type.getMethods())if(m.getName().equals(names.getProperty(key))&&m.getParameterTypes().length==args.length){boolean fits=true;for(int i=0;i<args.length;i++)if(!m.getParameterTypes()[i].isPrimitive()&&!m.getParameterTypes()[i].isInstance(args[i]))fits=false;if(fits)return m.invoke(target instanceof Class?null:target,args);}throw new NoSuchMethodException(key);
    }
    public static final class Player {final Object manager,stack;Player(Object m,Object s){manager=m;stack=s;}public Object getCooldowns(){return manager;}public Object getMainHandItem(){return stack;}public Object getItemInHand(Hand h){return stack;}}
    public enum Hand {MAIN}
    public static final class Result {public boolean consumesAction(){return true;}}
    static Object value(Object object,String key)throws Exception{Field f=object.getClass().getDeclaredField(key);f.setAccessible(true);return f.get(object);}
    public static void main(String[] args)throws Exception{
        try(java.io.InputStream in=Files.newInputStream(Paths.get(args[0]))){names.load(in);}
        invoke(clazz("sharedClass"),"detectMethod");invoke(clazz("bootstrapClass"),"bootstrapMethod");
        Object item=clazz("itemsClass").getField(names.getProperty("pearlField")).get(null),stack=null;
        for(Constructor<?> ctor:clazz("stackClass").getConstructors())if(ctor.getParameterTypes().length==1&&ctor.getParameterTypes()[0].isInstance(item)){stack=ctor.newInstance(item);break;}
        if(stack==null)throw new AssertionError("Actual ItemStack constructor missing");
        Object token=clazz("componentsClass").getField(names.getProperty("loreField")).get(null),line=invoke(clazz("componentClass"),"literalMethod","КД: 12 сек");
        Object lore=clazz("loreClass").getConstructor(List.class).newInstance(Collections.singletonList(line));invoke(stack,"stackSetMethod",token,lore);
        Object manager=clazz("cooldownClass").getConstructor().newInstance();Player player=new Player(manager,stack);invoke(manager,"addMethod",stack,80);
        Class<?> readerClass=Class.forName("ru.kyro.builtin.AgentCooldowns",true,null);Constructor<?> ctor=readerClass.getDeclaredConstructor();ctor.setAccessible(true);Object reader=ctor.newInstance();Method sample=readerClass.getDeclaredMethod("sample",Object.class);sample.setAccessible(true);
        List<?> rows=(List<?>)sample.invoke(reader,player);if(rows.size()!=1||!value(rows.get(0),"remaining").equals(80))throw new AssertionError("Actual cooldown group not read");
        for(int i=0;i<30;i++)invoke(manager,"tickMethod");rows=(List<?>)sample.invoke(reader,player);if(rows.size()!=1||!value(rows.get(0),"remaining").equals(50))throw new AssertionError("Actual tick counter not read");
        Class<?> hooks=Class.forName("ru.kyro.builtin.CooldownHooks",true,null);Method readLore=hooks.getDeclaredMethod("lore",Object.class);readLore.setAccessible(true);if(!readLore.invoke(null,stack).equals(12_000_000_000L)){Method comp=hooks.getDeclaredMethod("component",Object.class,String.class,String[].class);comp.setAccessible(true);Method text=hooks.getDeclaredMethod("text",Object.class);text.setAccessible(true);throw new AssertionError("Actual lore duration not parsed: "+readLore.invoke(null,stack)+" component="+comp.invoke(null,stack,"loreField",new String[]{"LORE","field_49632"})+" text="+text.invoke(null,line));}
        hooks.getMethod("beforeUse",Object.class,Object.class).invoke(null,player,Hand.MAIN);hooks.getMethod("afterUse",Object.class).invoke(null,new Result());Method merge=hooks.getDeclaredMethod("merge",Object.class,List.class);merge.setAccessible(true);rows=(List<?>)merge.invoke(null,player,Collections.emptyList());if(rows.size()!=1||!value(rows.get(0),"estimated").equals(true)||!value(rows.get(0),"total").equals(240))throw new AssertionError("Actual ItemLore component use not tracked, rows="+rows.size());
        // Loading and reflecting forces verification of the real transformed use-item method.
        Class.forName(names.getProperty("gameModeClass"),false,ActualCooldownHarness.class.getClassLoader()).getDeclaredMethods();
        Class<?> sky=Class.forName("ru.kyro.builtin.SkyHooks",true,null),agent=Class.forName("ru.kyro.builtin.AgentHooks",true,null);Field setting=agent.getDeclaredField("layout");setting.setAccessible(true);Object layout=setting.get(null);layout.getClass().getField("customSky").setBoolean(layout,true);layout.getClass().getField("skyUseThemeColor").setBoolean(layout,false);layout.getClass().getField("skyColor").setInt(layout,0x7856ab);layout.getClass().getField("skyMix").setDouble(layout,1);
        Object skyAttribute=clazz("skyAttributesClass").getField(names.getProperty("skyAttributeField")).get(null);Object colored=sky.getMethod("apply",Object.class,Object.class).invoke(null,0x88334455,skyAttribute);if(!colored.equals(0x887856ab))throw new AssertionError("Actual sky attribute identity not resolved");Class.forName(names.getProperty("skyProbeClass"),false,ActualCooldownHarness.class.getClassLoader()).getDeclaredMethods();
        System.out.println("PASS: actual Custom Sky attribute identity, chosen HEX/alpha and transformed environment probe verification");
        System.out.println("PASS: actual Minecraft ItemStack, ItemLore components, native cooldown map, advancing ticks, lore use timer and transformed game-mode verification");
    }
}
