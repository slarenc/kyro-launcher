package ru.kyro.hud;

import java.lang.reflect.*;
import java.util.*;
import java.util.regex.*;

/** Independent lore timers; the game's authoritative cooldowns take precedence. */
public final class CooldownHooks {
    static final Pattern LABEL=Pattern.compile("(?iuU)(?:cooldown|delay|перезарядк[а-я]*|задержк[а-я]*|откат[а-я]*|кд)[^\\d\\r\\n]{0,32}?(\\d+(?:[.,]\\d+)?)\\s*(мин(?:ут[а-я]*)?|minutes?|min|m|м|сек(?:унд[а-я]*)?|seconds?|secs?|s|с)?(?:\\b|$)");
    static final Map<String,Timer> timers=new LinkedHashMap<>();
    static Candidate pending;static Object owner;
    static final class Timer {final String key,label,itemLabel;final long start,duration;Timer(Candidate c,long now){key=c.key;label=c.label;itemLabel=c.itemLabel;start=now;duration=c.duration;}}
    static final class Candidate {
        final Object player,hand,stack;final String key,label,itemLabel;final int count;final long duration,at;final boolean manual;
        Candidate(Object player,Object hand,Object stack,String key,String label,int count,long duration){this(player,hand,stack,key,label,count,duration,false,label);}
        Candidate(Object player,Object hand,Object stack,String key,String label,int count,long duration,boolean manual,String itemLabel){this.player=player;this.hand=hand;this.stack=stack;this.key=key;this.label=label;this.count=count;this.duration=duration;this.manual=manual;this.itemLabel=itemLabel;at=System.nanoTime();}
    }
    public static String gameModeClass(){return AgentCooldowns.mapped("gameModeClass").replace('.','/');}
    public static String useMethod(){return AgentCooldowns.mapped("useItemMethod");}
    public static String useOnMethod(){return AgentCooldowns.mapped("useItemOnMethod");}
    public static String componentClass(){return AgentCooldowns.mapped("componentClass").replace('.','/');}
    public static String guiClass(){return AgentCooldowns.mapped("guiClass").replace('.','/');}
    public static synchronized void feedback(Object component){
        try{String message=AgentCooldowns.clean(text(component)).toLowerCase(Locale.ROOT);
            if(!(message.contains("подожд")||message.contains("осталось")||message.contains("нельзя")||message.contains("не можете")||message.contains("ещё не")||message.contains("еще не")||message.contains("please wait")||message.contains("remaining")))return;
            if(!(message.contains("сек")||message.contains("кд")||message.contains("откат")||message.contains("перезар")||message.contains("cooldown")||message.contains("wait")))return;
            pending=null;long now=System.nanoTime();Iterator<Timer> it=timers.values().iterator();while(it.hasNext())if(now-it.next().start<1_000_000_000L)it.remove();
        }catch(Throwable ignored){}
    }
    static Object call(Object target,String key,String...aliases)throws Exception{
        String[] all=Arrays.copyOf(aliases,aliases.length+1);all[aliases.length]=AgentCooldowns.mapped(key);Method method=AgentCooldowns.named(target.getClass(),all);return method==null?null:method.invoke(target);
    }
    static Object hand(Object player,Object hand)throws Exception{
        for(String name:new String[]{"getItemInHand","getStackInHand","method_5998",AgentCooldowns.mapped("itemInHandMethod")})if(!name.isEmpty())try{return player.getClass().getMethod(name,hand.getClass()).invoke(player,hand);}catch(NoSuchMethodException ignored){}
        return null;
    }
    static int count(Object stack)throws Exception{Object n=call(stack,"stackCountMethod","getCount","method_7947");return n instanceof Number?((Number)n).intValue():-1;}
    static Object component(Object stack,String key,String...aliases)throws Exception{
        Class<?> type=null;ClassLoader loader=stack.getClass().getClassLoader();
        for(String name:new String[]{"net.minecraft.core.component.DataComponents","net.minecraft.component.DataComponentTypes","net.minecraft.class_9334",AgentCooldowns.mapped("componentsClass")})if(!name.isEmpty())try{type=Class.forName(name,false,loader);break;}catch(ClassNotFoundException ignored){}
        if(type==null)return null;Object token=null;
        String[] fields=Arrays.copyOf(aliases,aliases.length+1);fields[aliases.length]=AgentCooldowns.mapped(key);
        for(String name:fields)if(!name.isEmpty())try{token=type.getField(name).get(null);break;}catch(NoSuchFieldException ignored){}
        if(token==null)return null;
        // Never guess a structural getter: remove(ComponentType) has the same shape.
        for(String name:new String[]{"get","method_58694","method_57824",AgentCooldowns.mapped("componentGetMethod")})if(!name.isEmpty())for(Method method:stack.getClass().getMethods())if(method.getName().equals(name)&&method.getParameterTypes().length==1&&method.getParameterTypes()[0].isInstance(token)&&!method.getReturnType().isPrimitive())return method.invoke(stack,token);
        return null;
    }
    static String text(Object value)throws Exception{if(value instanceof String)return (String)value;Object result=call(value,"","getString","method_10851","getUnformattedText");return result instanceof String?(String)result:"";}
    static long duration(String line){
        Matcher m=LABEL.matcher(AgentCooldowns.clean(line));if(!m.find())return 0;
        try{double n=Double.parseDouble(m.group(1).replace(',','.'));String unit=m.group(2)==null?"":m.group(2).toLowerCase(Locale.ROOT);if(unit.startsWith("м")||unit.startsWith("m")){n*=60;Matcher tail=Pattern.compile("(?iuU)^[\\s.,]*(\\d+(?:[.,]\\d+)?)\\s*(?:сек|с\\b|sec|s\\b)").matcher(AgentCooldowns.clean(line).substring(m.end()));if(tail.find())n+=Double.parseDouble(tail.group(1).replace(',','.'));}return n>0&&n<=86400?Math.round(n*1e9):0;}catch(Exception ignored){return 0;}
    }
    static long lore(Object stack)throws Exception{
        Object lore=component(stack,"loreField","LORE","field_49632");if(lore==null)return 0;Object lines=call(lore,"loreLinesMethod","lines","comp_2400");if(!(lines instanceof Iterable))return 0;
        int scanned=0;for(Object line:(Iterable<?>)lines){if(scanned++>=64)break;long delay=duration(text(line));if(delay>0)return delay;}return 0;
    }
    static String itemLabel(Object stack)throws Exception{return AgentCooldowns.clean(text(call(stack,"stackNameMethod","getHoverName","getName","method_7964")));}
    static String itemKey(Object stack,String label)throws Exception{
        Object item=call(stack,"stackItemMethod","getItem","method_7909"),name=call(stack,"stackNameMethod","getHoverName","getName","method_7964");
        String identity=String.valueOf(item)+"\0"+String.valueOf(name==null?label:name)+"\0"+String.valueOf(component(stack,"customModelField","CUSTOM_MODEL_DATA","field_49637"))+"\0"+String.valueOf(component(stack,"itemModelField","ITEM_MODEL","field_54199"));
        byte[] digest=java.security.MessageDigest.getInstance("SHA-256").digest(identity.getBytes(java.nio.charset.StandardCharsets.UTF_8));StringBuilder hex=new StringBuilder(64);for(byte b:digest){hex.append(Character.forDigit((b>>>4)&15,16));hex.append(Character.forDigit(b&15,16));}return "item:"+item+"|"+hex;
    }
    static PanelLayout.CooldownRule rule(String key){
        PanelLayout settings=AgentHooks.layout;if(settings==null||settings.cooldownRules==null)return null;
        for(PanelLayout.CooldownRule r:settings.cooldownRules)if(r!=null&&key.equals(r.key)&&Double.isFinite(r.seconds)&&r.seconds>0&&r.seconds<=86400)return r;return null;
    }
    static PanelLayout.CooldownRule fromMainHand(Object player)throws Exception{
        if(player==null)return null;Object stack=call(player,"mainHandMethod","getMainHandItem","getMainHandStack","method_6047");if(stack==null||count(stack)==0)return null;String label=itemLabel(stack);if(label.isEmpty())return null;String key=itemKey(stack,label);PanelLayout.CooldownRule existing=rule(key);return new PanelLayout.CooldownRule(key,existing==null?label:existing.label,existing==null?30:existing.seconds);
    }
    static boolean failed(Object result){
        if(result==null)return true;if(result instanceof Enum&&((Enum<?>)result).name().equalsIgnoreCase("FAIL"))return true;
        for(String name:new String[]{"FAIL","field_5814",AgentCooldowns.mapped("interactionFailField")})if(!name.isEmpty())try{Object fail=result.getClass().getField(name).get(null);if(result==fail||!(result instanceof Enum)&&fail!=null&&result.getClass()==fail.getClass())return true;}catch(Exception ignored){}return false;
    }
    static synchronized void forget(String key){timers.remove(key);if(pending!=null&&pending.key.equals(key))pending=null;}
    public static synchronized void beforeUse(Object player,Object hand){
        try{
            // Hook only the local player's action. The renderer updates the owner.
            if(owner!=null&&owner!=player)return;pending=null;Object stack=hand(player,hand);if(stack==null||count(stack)==0)return;String itemLabel=itemLabel(stack);if(itemLabel.isEmpty())itemLabel="Предмет";String key=itemKey(stack,itemLabel);PanelLayout.CooldownRule rule=rule(key);long delay=rule==null?lore(stack):Math.round(rule.seconds*1e9);if(delay<=0)return;String label=rule==null?itemLabel:rule.label;if(rule!=null)key=rule.key;
            if(timers.containsKey(key)&&System.nanoTime()-timers.get(key).start<timers.get(key).duration)return;
            pending=new Candidate(player,hand,stack,key,label,count(stack),delay,rule!=null,itemLabel);
        }catch(Throwable ignored){pending=null;}
    }
    public static synchronized void afterUse(Object result){
        if(pending==null)return;
        try{if(failed(result)){pending=null;return;}Object accepted=call(result,"consumesActionMethod","consumesAction","isAccepted","method_23665");if(Boolean.TRUE.equals(accepted)||pending!=null&&pending.manual)start();}catch(Throwable ignored){}
    }
    static void start(){if(pending==null)return;if(timers.size()>=32)timers.remove(timers.keySet().iterator().next());timers.put(pending.key,new Timer(pending,System.nanoTime()));pending=null;}
    public static synchronized void reset(){pending=null;owner=null;timers.clear();}
    static synchronized List<AgentCooldowns.Entry> merge(Object player,List<AgentCooldowns.Entry> vanilla){
        if(player==null){reset();return vanilla;}if(owner!=null&&owner!=player)reset();owner=player;long now=System.nanoTime();
        if(pending!=null){try{Object current=hand(player,pending.hand);if(pending.count>0&&current!=null&&count(current)<pending.count)start();}catch(Throwable ignored){}if(pending!=null&&now-pending.at>500_000_000L)pending=null;}
        List<AgentCooldowns.Entry> out=new ArrayList<>(vanilla);Iterator<Timer> it=timers.values().iterator();
        while(it.hasNext()){Timer timer=it.next();long left=timer.duration-(now-timer.start);if(left<=0){it.remove();continue;}boolean duplicate=false;for(AgentCooldowns.Entry exact:vanilla)if(exact.label.equals(timer.label)||exact.label.equals(timer.itemLabel)){duplicate=true;break;}if(duplicate)it.remove();else out.add(new AgentCooldowns.Entry(timer.key,timer.label,(int)Math.ceil(left/50_000_000.0),(int)Math.ceil(timer.duration/50_000_000.0),true));}
        out.sort(Comparator.comparingInt((AgentCooldowns.Entry e)->e.remaining).thenComparing(e->e.group));if(out.size()>AgentCooldowns.MAX_ROWS)out=new ArrayList<>(out.subList(0,AgentCooldowns.MAX_ROWS));return Collections.unmodifiableList(out);
    }
}
