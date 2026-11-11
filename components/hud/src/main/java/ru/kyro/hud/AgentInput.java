package ru.kyro.hud;

import java.lang.invoke.*;
import java.lang.reflect.*;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.util.*;

/** Keep native callbacks intact; consume only HUD controls and the right-Shift editor. */
final class AgentInput implements InvocationHandler {
    static final Map<Long,AgentInput> keyboards=new HashMap<>();
    static final Method INVOKE_DEFAULT=invokeDefault();
    static Method invokeDefault(){try{return InvocationHandler.class.getMethod("invokeDefault",Object.class,Method.class,Object[].class);}catch(Exception oldJava){return null;}}
    final Object delegate;final String kind;final Class<?> iface;final Map<Method,MethodHandle> defaults=new HashMap<>();
    final Map<Integer,Integer> heldKeys=new HashMap<>();Method callback;Long address;
    final Set<Integer> capturedButtons=new HashSet<>();
    AgentInput(Object delegate,String kind,Class<?> iface){this.delegate=delegate;this.kind=kind;this.iface=iface;}
    static Object wrap(long window,Object delegate,String kind){
        if(delegate==null||Proxy.isProxyClass(delegate.getClass())&&Proxy.getInvocationHandler(delegate) instanceof AgentInput)return delegate;
        try{
            String suffix=kind.equals("mouse")?"MouseButton":kind.equals("cursor")?"CursorPos":kind.equals("scroll")?"Scroll":kind.equals("char")?"Char":kind.equals("charMods")?"CharMods":"Key";
            Class<?> iface=Class.forName("org.lwjgl.glfw.GLFW"+suffix+"CallbackI",false,delegate.getClass().getClassLoader());
            AgentInput handler=new AgentInput(delegate,kind,iface);for(Method method:iface.getMethods())if(method.getName().equals("invoke"))handler.callback=method;
            Object proxy=Proxy.newProxyInstance(iface.getClassLoader(),new Class<?>[]{iface},handler);
            if(kind.equals("key"))keyboards.put(window,handler);return proxy;
        }catch(Throwable error){AgentHooks.log("Input wrapper unavailable",error);return delegate;}
    }
    public Object invoke(Object proxy,Method method,Object[] args)throws Throwable{
        String name=method.getName();
        if(name.equals("invoke")){
            long window=((Number)args[0]).longValue();boolean consumed=false;
            try{
                if(kind.equals("key")){
                    int key=((Number)args[1]).intValue(),action=((Number)args[3]).intValue();
                    if(action==1)heldKeys.put(key,((Number)args[2]).intValue());else if(action==0)heldKeys.remove(key);
                    consumed=AgentHooks.key(window,key,action,((Number)args[4]).intValue());
                }else if(kind.equals("mouse")){
                    int button=((Number)args[1]).intValue(),action=((Number)args[2]).intValue();consumed=AgentHooks.mouse(window,button,action);
                    if(action==1&&consumed)capturedButtons.add(button);if(action==0&&capturedButtons.remove(button))consumed=true;
                }
                else if(kind.equals("scroll"))consumed=AgentHooks.scroll(window,((Number)args[2]).doubleValue());
                else if(kind.equals("cursor"))consumed=AgentHooks.editor;
                else if(kind.equals("char")||kind.equals("charMods"))consumed=AgentHooks.character(window,((Number)args[1]).intValue());
            }catch(Throwable error){AgentHooks.log("Input dispatch failed",error);}
            if(consumed)return null;return method.invoke(delegate,args);
        }
        if(name.equals("address")&&address!=null)return address;
        if(method.isDefault()){
            try{
                if(INVOKE_DEFAULT!=null){Object result=INVOKE_DEFAULT.invoke(null,new Object[]{proxy,method,args==null?new Object[0]:args});if(name.equals("address"))address=((Number)result).longValue();return result;}
                MethodHandle handle=defaults.get(method);
                if(handle==null){
                    MethodHandles.Lookup lookup;
                    try{Method privateLookup=MethodHandles.class.getMethod("privateLookupIn",Class.class,MethodHandles.Lookup.class);lookup=(MethodHandles.Lookup)privateLookup.invoke(null,method.getDeclaringClass(),MethodHandles.lookup());}
                    catch(NoSuchMethodException oldJava){Constructor<MethodHandles.Lookup> constructor=MethodHandles.Lookup.class.getDeclaredConstructor(Class.class,int.class);constructor.setAccessible(true);lookup=constructor.newInstance(method.getDeclaringClass(),15);}
                    handle=lookup.unreflectSpecial(method,method.getDeclaringClass()).bindTo(proxy);defaults.put(method,handle);
                }
                Object result=handle.invokeWithArguments(args==null?Collections.emptyList():Arrays.asList(args));
                if(name.equals("address"))address=((Number)result).longValue();return result;
            }catch(Throwable error){AgentHooks.log("Callback default-method fallback",error);return method.invoke(delegate,args);}
        }
        if(name.equals("equals"))return proxy==args[0];if(name.equals("hashCode"))return System.identityHashCode(proxy);if(name.equals("toString"))return "KYRO "+kind+" callback";
        return method.invoke(delegate,args);
    }
    static void resumePointer(long window){Object client=AgentHooks.game.clientObject();Object mouse=GameAccess.field(client,"mouseHandler");GameAccess.call(mouse,"setIgnoreFirstMove");}
    static void releaseKeys(long window){if(AgentSDL.active())GameAccess.call(GameAccess.type("net.minecraft.client.KeyMapping",AgentHooks.glfw.getClassLoader()),"releaseAll");AgentInput keys=keyboards.get(window);if(keys==null)return;for(Map.Entry<Integer,Integer> key:new HashMap<>(keys.heldKeys).entrySet())try{keys.callback.invoke(keys.delegate,window,key.getKey(),key.getValue(),0,0);}catch(Exception ignored){}keys.heldKeys.clear();}
    static void tap(long window,int key){AgentInput keys=keyboards.get(window);if(keys==null)return;try{keys.callback.invoke(keys.delegate,window,key,0,1,0);keys.callback.invoke(keys.delegate,window,key,0,0,0);}catch(Exception error){AgentHooks.log("Menu key failed",error);}}
    static int binding(String binding,int fallback){
        try{Path options=Paths.get(System.getProperty("kyro.hud.config","config")).toAbsolutePath().getParent().resolve("options.txt");
            for(String line:Files.readAllLines(options,StandardCharsets.UTF_8))if(line.startsWith("key_key."+binding+":")){
                String value=line.substring(line.indexOf(':')+1).trim();String prefix="key.keyboard.";
                if(value.startsWith(prefix)){String key=value.substring(prefix.length());if(key.length()==1)return Character.toUpperCase(key.charAt(0));if(key.equals("enter"))return 257;if(key.equals("tab"))return 258;if(key.equals("grave.accent"))return 96;}
            }
        }catch(Exception ignored){}return fallback;
    }
}
