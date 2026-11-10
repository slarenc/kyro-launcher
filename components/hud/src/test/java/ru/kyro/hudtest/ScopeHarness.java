package ru.kyro.hudtest;
import net.minecraft.client.gui.hud.ChatHud;
import java.lang.reflect.*;
public final class ScopeHarness {
    public static void main(String[] args)throws Exception{
        Object p=new Object(),result=new Object();if(new net.minecraft.client.network.ClientPlayerInteractionManager().interactItem(p,result)!=result)throw new AssertionError("Cooldown hook changed use-item result");
        Class<?> hooks=Class.forName("ru.kyro.builtin.BadgeHooks",true,null);Field field=hooks.getDeclaredField("SCOPE");field.setAccessible(true);ThreadLocal<?> depth=(ThreadLocal<?>)field.get(null);
        Runnable active=()->{if(((Number)depth.get()).intValue()<1)throw new AssertionError("Chat scope missing");};
        ChatHud.paint(active);if(((Number)depth.get()).intValue()!=0)throw new AssertionError("Scope leaked to scoreboard");
        try{ChatHud.paintAndFail(active);throw new AssertionError();}catch(IllegalStateException expected){}
        if(((Number)depth.get()).intValue()!=0)throw new AssertionError("Exception leaked chat scope");
        if(ChatHud.catchesInternally(active)!=7L||((Number)depth.get()).intValue()!=0)throw new AssertionError("Original exception handler changed");
        ChatHud.paint(()->ChatHud.paint(active));if(((Number)depth.get()).intValue()!=0)throw new AssertionError("Nested scopes leaked");
        System.out.println("PASS: scoped chat/TAB/name rendering, original exception handlers, nested calls, no scope leaks into sidebar");
    }
}
