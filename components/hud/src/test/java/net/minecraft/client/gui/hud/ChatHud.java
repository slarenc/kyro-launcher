package net.minecraft.client.gui.hud;
/** A bytecode fixture: the production agent recognizes this renderer class name. */
public final class ChatHud {
    public static void paint(Runnable body){body.run();}
    public static int paintAndFail(Runnable body){body.run();throw new IllegalStateException("expected");}
    public static long catchesInternally(Runnable body){try{body.run();throw new IllegalArgumentException();}catch(IllegalArgumentException expected){return 7L;}}
}
