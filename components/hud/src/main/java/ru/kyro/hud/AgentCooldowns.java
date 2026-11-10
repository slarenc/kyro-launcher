package ru.kyro.hud;

import java.lang.reflect.*;
import java.util.*;

/** Read the game's actual cooldown groups/ticks, including custom ItemStack groups. */
final class AgentCooldowns {
    static final int WIDTH=140,ROW=21,MAX_ROWS=3;
    static final class Entry {
        final String group,label;final int remaining,total;final boolean estimated;
        Entry(String group,String label,int remaining,int total){this(group,label,remaining,total,false);}
        Entry(String group,String label,int remaining,int total,boolean estimated){this.group=group;this.label=label;this.remaining=remaining;this.total=total;this.estimated=estimated;}
        double seconds(){return remaining/20.0;}
    }
    static final class Active {final int start;final String label;Active(int start,String label){this.start=start;this.label=label;}}
    final Map<String,String> labels=new LinkedHashMap<String,String>(){protected boolean removeEldestEntry(Map.Entry<String,String> entry){return size()>128;}};
    final Map<String,Active> active=new HashMap<>();
    Object lastPlayer;Class<?> playerType,managerType;Method getter,groupMethod,itemMethod,nameMethod;Field mapField,tickField;List<Method> hands=Collections.emptyList();
    final Map<Class<?>,List<Field>> times=new HashMap<>();
    static String mapped(String key){return BadgeHooks.mapped(key);}
    static Method named(Class<?> type,String...names){for(String name:names)if(name!=null&&!name.isEmpty())try{return type.getMethod(name);}catch(Exception ignored){}return null;}
    static List<Field> fields(Class<?> type){List<Field> result=new ArrayList<>();for(Class<?> c=type;c!=null;c=c.getSuperclass())for(Field field:c.getDeclaredFields())if(!Modifier.isStatic(field.getModifiers()))result.add(field);return result;}
    static boolean manager(Class<?> type){try{boolean map=false,tick=false,percent=false;for(Field f:fields(type)){map|=Map.class.isAssignableFrom(f.getType());tick|=f.getType()==int.class;}for(Method m:type.getMethods()){Class<?>[] p=m.getParameterTypes();if(m.getReturnType()==float.class&&p.length==2&&p[1]==float.class)percent=true;}return map&&tick&&percent;}catch(Throwable ignored){return false;}}
    void bindPlayer(Class<?> type){
        playerType=type;getter=named(type,"getCooldowns","getItemCooldownManager","method_7357",mapped("playerCooldownMethod"));
        if(getter==null)for(Method method:type.getMethods())if(method.getParameterTypes().length==0&&manager(method.getReturnType())){getter=method;break;}
        LinkedHashSet<Method> found=new LinkedHashSet<>();for(String alias:new String[]{"getMainHandItem","getMainHandStack","method_6047",mapped("mainHandMethod"),"getOffhandItem","getOffHandStack","method_6079",mapped("offHandMethod")}){Method m=named(type,alias);if(m!=null)found.add(m);}
        hands=new ArrayList<>(found);managerType=null;
    }
    void bindManager(Class<?> type)throws Exception{
        managerType=type;mapField=null;tickField=null;groupMethod=null;itemMethod=null;nameMethod=null;
        for(Field field:fields(type)){if(mapField==null&&Map.class.isAssignableFrom(field.getType()))mapField=field;if(tickField==null&&field.getType()==int.class)tickField=field;}
        if(mapField==null||tickField==null)throw new NoSuchFieldException("Cooldown map/tick counter");mapField.setAccessible(true);tickField.setAccessible(true);
    }
    Object group(Object manager,Object stack)throws Exception{
        if(groupMethod==null)for(Method m:manager.getClass().getMethods()){Class<?>[] p=m.getParameterTypes();if(p.length==1&&p[0]==stack.getClass()&&!m.getReturnType().isPrimitive()&&m.getDeclaringClass()!=Object.class){groupMethod=m;break;}}
        if(groupMethod!=null)return groupMethod.invoke(manager,stack);
        if(itemMethod==null)itemMethod=named(stack.getClass(),"getItem","method_7909",mapped("stackItemMethod"));return itemMethod==null?null:itemMethod.invoke(stack);
    }
    String label(Object stack){try{
        if(nameMethod==null)nameMethod=named(stack.getClass(),"getHoverName","getName","getDisplayName","method_7964",mapped("stackNameMethod"));if(nameMethod==null)return "";
        Object component=nameMethod.invoke(stack);Object text=component instanceof String?component:AgentGame.call(component,new String[]{"getString","method_10851","getUnformattedText"});return clean(text instanceof String?(String)text:"");
    }catch(Exception ignored){return "";}}
    static String clean(String value){
        if(value.length()>256)value=value.substring(0,256);value=value.replaceAll("(?i)§[0-9A-FK-ORX]","");StringBuilder out=new StringBuilder();
        for(int at=0;at<value.length();){int code=value.codePointAt(at);at+=Character.charCount(code);int kind=Character.getType(code);if(kind!=Character.PRIVATE_USE&&!Character.isISOControl(code)&&kind!=Character.FORMAT)out.appendCodePoint(code);}
        return out.toString().replaceAll("\\s+"," ").trim();
    }
    static String fallback(String group){String id=group.substring(group.lastIndexOf(':')+1);if(id.equals("ender_pearl"))return "Эндер-жемчуг";if(id.equals("chorus_fruit"))return "Плод хоруса";if(id.equals("shield"))return "Щит";if(id.equals("wind_charge"))return "Заряд ветра";if(id.equals("goat_horn"))return "Козий рог";return "Предмет";}
    List<Entry> sample(Object player){
        if(player==null){lastPlayer=null;active.clear();labels.clear();return Collections.emptyList();}
        try{
            if(lastPlayer!=player){lastPlayer=player;active.clear();labels.clear();}if(playerType!=player.getClass())bindPlayer(player.getClass());if(getter==null)return Collections.emptyList();
            Object cooldowns=getter.invoke(player);if(cooldowns==null)return Collections.emptyList();if(managerType!=cooldowns.getClass())bindManager(cooldowns.getClass());
            HashSet<String> observed=new HashSet<>();for(Method hand:hands){Object stack=hand.invoke(player);if(stack==null)continue;Object key=group(cooldowns,stack);if(key==null)continue;String id=key.toString(),name=label(stack);if(!name.isEmpty()&&observed.add(id))labels.put(id,name);}
            int tick=tickField.getInt(cooldowns);Object raw=mapField.get(cooldowns);if(!(raw instanceof Map))return Collections.emptyList();Map<?,?> map=(Map<?,?>)raw;List<Entry> result=new ArrayList<>();HashSet<String> alive=new HashSet<>();int scanned=0;
            for(Map.Entry<?,?> entry:map.entrySet()){
                if(scanned++>=64)break;Object value=entry.getValue();if(value==null)continue;List<Field> pair=times.get(value.getClass());
                if(pair==null){pair=new ArrayList<>();for(Field field:fields(value.getClass()))if(field.getType()==int.class){field.setAccessible(true);pair.add(field);}times.put(value.getClass(),pair);}
                if(pair.size()!=2)continue;int a=pair.get(0).getInt(value),b=pair.get(1).getInt(value),start=Math.min(a,b),end=Math.max(a,b),remaining=end-tick,total=end-start;
                if(total<=0||remaining<=0||remaining>total||remaining>20*86400)continue;String id=String.valueOf(entry.getKey());alive.add(id);Active known=active.get(id);
                if(known==null||known.start!=start){known=new Active(start,labels.getOrDefault(id,fallback(id)));active.put(id,known);}result.add(new Entry(id,known.label,remaining,total));
            }
            active.keySet().retainAll(alive);result.sort(Comparator.comparingInt((Entry entry)->entry.remaining).thenComparing(entry->entry.group));if(result.size()>MAX_ROWS)result=new ArrayList<>(result.subList(0,MAX_ROWS));return Collections.unmodifiableList(result);
        }catch(Throwable ignored){return Collections.emptyList();}
    }
}
