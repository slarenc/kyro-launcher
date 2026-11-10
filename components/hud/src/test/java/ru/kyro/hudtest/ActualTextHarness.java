package ru.kyro.hudtest;
import java.lang.reflect.*;
import java.util.*;
/** Exercises the real cached game's text visitor with the production JVM agent. */
public final class ActualTextHarness {
    public static void main(String[] args)throws Exception{
        System.out.println("Loading actual text visitor "+args[0]);
        Class.forName("ru.kyro.builtin.BadgeHooks",true,null).getMethod("enter").invoke(null);
        Class<?> factory=Class.forName(args[0]);Method visit=null;
        for(Method method:factory.getDeclaredMethods()){
            Class<?>[] p=method.getParameterTypes();if(Modifier.isStatic(method.getModifiers())&&method.getReturnType()==boolean.class&&p.length==4&&p[0]==String.class&&p[1]==int.class&&p[3].isInterface()){visit=method;break;}
        }
        if(visit==null)throw new AssertionError("Real text visitor missing");
        Class<?> styleClass=visit.getParameterTypes()[2],sinkClass=visit.getParameterTypes()[3];Object style=null;
        for(Field field:styleClass.getFields())if(Modifier.isStatic(field.getModifiers())&&field.getType()==styleClass){style=field.get(null);if(style!=null)break;}
        if(style==null)throw new AssertionError("Default game text style missing");
        String original="<WannaForget> hello";StringBuilder rendered=new StringBuilder();List<Integer> indices=new ArrayList<>();
        final Object originalStyle=style;final boolean[] isolated={false};
        Object sink=Proxy.newProxyInstance(sinkClass.getClassLoader(),new Class<?>[]{sinkClass},(proxy,method,values)->{int code=((Number)values[2]).intValue();if(code==0xE100){if(!values[1].toString().contains("kyro:badge"))throw new AssertionError("Badge inherited server/default font: "+values[1]);isolated[0]=true;}else if(code!=32&&values[1]!=originalStyle)throw new AssertionError("Nickname style changed");rendered.appendCodePoint(code);indices.add(((Number)values[0]).intValue());return true;});
        visit.invoke(null,original,0,style,sink);
        if(!isolated[0]||!rendered.toString().equals("<\uE100 WannaForget> hello"))throw new AssertionError("Actual game did not render exactly one badge: "+rendered);
        if(!original.equals("<WannaForget> hello")||indices.size()!=original.length()+2||indices.get(1)!=1||indices.get(2)!=1||indices.get(3)!=1)throw new AssertionError("Source text indices changed");
        // Bidi text replay visits the nickname as its own font run. It must not
        // add a second badge when the first glyph has a dedicated font/style.
        Method forward=null;for(Method method:factory.getDeclaredMethods()){
            Class<?>[] p=method.getParameterTypes();if(Modifier.isStatic(method.getModifiers())&&method.getReturnType()==boolean.class&&Arrays.equals(p,new Class<?>[]{String.class,styleClass,sinkClass})&&Arrays.asList("a","visitForwards","iterate","method_27474").contains(method.getName())){forward=method;break;}
        }
        if(forward==null)throw new AssertionError("Plain glyph replay missing");rendered.setLength(0);forward.invoke(null,"WannaForget",style,sink);if(!rendered.toString().equals("WannaForget"))throw new AssertionError("Badge injected again during styled glyph replay");
        System.out.println("PASS: actual Minecraft text visitor "+args[0]+", exactly one icon plus space, original text/style/indices retained");
    }
}
