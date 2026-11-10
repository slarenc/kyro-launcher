package ru.kyro.hud;

import java.lang.reflect.*;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import com.google.gson.*;
import java.util.*;
import java.util.concurrent.*;

/** Adds a local visual glyph while preserving source text and chat signatures. */
public final class BadgeHooks implements InvocationHandler {
    public static final int GLYPH=0xE100;
    static final Map<Class<?>,Optional<Object>> STYLES=new ConcurrentHashMap<>();
    static final JsonObject MAPPINGS=mappings();
    static final ThreadLocal<Integer> SCOPE=ThreadLocal.withInitial(()->0);
    public static void enter(){SCOPE.set(SCOPE.get()+1);}
    public static void exit(){SCOPE.set(Math.max(0,SCOPE.get()-1));}
    public static String[] scopeClasses(){try{JsonArray values=MAPPINGS.getAsJsonArray("badgeScopes");String[] names=new String[values.size()];for(int i=0;i<names.length;i++)names[i]=values.get(i).getAsString().replace('.','/');return names;}catch(Exception ignored){return new String[0];}}
    static final class Names {final Map<Character,Names> next=new HashMap<>();boolean end;}
    static volatile Names names=new Names();static volatile Set<String> peers=Collections.emptySet();static volatile String owner="";
    public static synchronized void setPeers(Collection<String> values){
        HashSet<String> all=new HashSet<>();for(String value:values)if(value!=null&&value.matches("[A-Za-z0-9_]{3,16}")&&all.size()<256)all.add(value);peers=Collections.unmodifiableSet(all);owner=System.getProperty("kyro.hud.player","");all=new HashSet<>(all);if(owner.matches("[A-Za-z0-9_]{3,16}"))all.add(owner);
        Names root=new Names();for(String value:all){Names node=root;for(int i=0;i<value.length();i++){char c=value.charAt(i);Names child=node.next.get(c);if(child==null){child=new Names();node.next.put(c,child);}node=child;}node.end=true;}names=root;
    }
    static BitSet find(String text){if(!owner.equals(System.getProperty("kyro.hud.player","")))setPeers(peers);Names root=names;BitSet starts=null;for(int i=0;i<text.length();i++){if(i>0&&nameChar(text.charAt(i-1)))continue;Names node=root;for(int at=i;at<text.length()&&at<i+16;at++){node=node.next.get(text.charAt(at));if(node==null)break;if(node.end&&(at+1==text.length()||!nameChar(text.charAt(at+1)))){if(starts==null)starts=new BitSet();starts.set(i);break;}}}return starts;}
    final String text;final BitSet starts;final Object sink;final Method accept;
    BadgeHooks(String text,BitSet starts,Object sink,Class<?> type)throws Exception{
        this.text=text;this.starts=starts;this.sink=sink;Method found=null;
        for(Method m:type.getMethods())if(m.getReturnType()==boolean.class&&m.getParameterTypes().length==3&&m.getParameterTypes()[0]==int.class&&m.getParameterTypes()[2]==int.class){found=m;break;}
        if(found==null)throw new NoSuchMethodException("Character sink");accept=found;
    }
    static JsonObject mappings(){try{
        Path file=Paths.get(System.getProperty("kyro.hud.config","config"),"kyro-hud-mapping.json");
        return new Gson().fromJson(new String(Files.readAllBytes(file),StandardCharsets.UTF_8),JsonObject.class);
    }catch(Exception ignored){return new JsonObject();}}
    static String mapped(String key){return MAPPINGS!=null&&MAPPINGS.has(key)?MAPPINGS.get(key).getAsString():"";}
    public static String textClass(){return mapped("textClass").replace('.','/');}
    static Object identifier(Class<?> type)throws Exception{
        for(Method m:type.getMethods())if(Modifier.isStatic(m.getModifiers())&&m.getReturnType()==type&&Arrays.equals(m.getParameterTypes(),new Class<?>[]{String.class,String.class}))try{
            Object id=m.invoke(null,"kyro","badge");if("kyro:badge".equals(String.valueOf(id)))return id;
        }catch(Exception ignored){}
        try{return type.getConstructor(String.class,String.class).newInstance("kyro","badge");}catch(NoSuchMethodException ignored){}
        throw new NoSuchMethodException("Resource identifier: "+type.getName());
    }
    static Object font(Class<?> type)throws Exception{
        try{return identifier(type);}catch(NoSuchMethodException modernFont){}
        // Newer Minecraft wraps the resource identifier in FontDescription.Resource.
        for(Field f:type.getFields())if(Modifier.isStatic(f.getModifiers())&&type.isAssignableFrom(f.getType())){
            Object standard=f.get(null);if(standard==null)continue;
            for(Constructor<?> ctor:standard.getClass().getConstructors())if(ctor.getParameterTypes().length==1)try{return ctor.newInstance(identifier(ctor.getParameterTypes()[0]));}catch(Exception ignored){}
        }
        throw new NoSuchMethodException("Resource font: "+type.getName());
    }
    static Object createStyle(Class<?> type)throws Exception{
        Object empty=null;for(Field f:type.getFields())if(Modifier.isStatic(f.getModifiers())&&f.getType()==type){empty=f.get(null);if(empty!=null)break;}
        if(empty==null)throw new NoSuchFieldException("Empty text style");
        for(String name:new String[]{"withFont","method_27704",mapped("styleFontMethod")}){
            if(name.isEmpty())continue;
            for(Method m:type.getMethods())if(m.getName().equals(name)&&m.getReturnType()==type&&m.getParameterTypes().length==1){
                Object resource;try{resource=font(m.getParameterTypes()[0]);}catch(ReflectiveOperationException unrelatedOverload){continue;}
                Object result=m.invoke(empty,resource);
                for(String color:new String[]{"withColor","method_36139",mapped("styleColorMethod")})try{if(!color.isEmpty())return type.getMethod(color,int.class).invoke(result,0xFFFFFF);}catch(NoSuchMethodException ignored){}
                return result;
            }
        }
        throw new NoSuchMethodException("Font style: "+type.getName());
    }
    static Object badgeStyle(Object original){
        if(original==null)return null;Class<?> type=original.getClass();Optional<Object> cached=STYLES.get(type);
        if(cached==null){try{cached=Optional.of(createStyle(type));}catch(Exception error){cached=Optional.empty();AgentHooks.log("Isolated badge font unavailable",error);}STYLES.put(type,cached);}
        return cached.orElse(null);
    }
    public static Object wrap(String text,Object sink,Class<?> type){
        if(SCOPE.get()==0)return sink;
        if(text==null||sink==null||Proxy.isProxyClass(sink.getClass())&&Proxy.getInvocationHandler(sink) instanceof BadgeHooks)return sink;BitSet starts=find(text);if(starts==null)return sink;
        try{return Proxy.newProxyInstance(type.getClassLoader(),new Class<?>[]{type},new BadgeHooks(text,starts,sink,type));}
        catch(Exception error){AgentHooks.log("Name badge unavailable",error);return sink;}
    }
    static boolean nameChar(char c){return c>='A'&&c<='Z'||c>='a'&&c<='z'||c>='0'&&c<='9'||c=='_';}
    public Object invoke(Object proxy,Method method,Object[] args)throws Throwable{
        if(method.equals(accept)){
            int index=((Number)args[0]).intValue(),code=((Number)args[2]).intValue();
            if(index>=0&&index<text.length()&&starts.get(index)&&code==text.charAt(index)){
                Object style=badgeStyle(args[1]);
                // Never fall back to the server's font when isolation is unavailable.
                if(style==null)return accept.invoke(sink,args);
                // Both added glyphs keep the original source index: wrapping and selections
                // continue to address the unmodified, server-signed chat component.
                if(!((Boolean)accept.invoke(sink,index,style,GLYPH)))return false;
                if(!((Boolean)accept.invoke(sink,index,style,32)))return false;
            }
            return accept.invoke(sink,args);
        }
        if(method.getName().equals("equals"))return proxy==args[0];
        if(method.getName().equals("hashCode"))return System.identityHashCode(proxy);
        if(method.getName().equals("toString"))return "KYRO name badge";
        return method.invoke(sink,args);
    }
}
