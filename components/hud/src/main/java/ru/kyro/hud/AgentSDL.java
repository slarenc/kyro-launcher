package ru.kyro.hud;

import java.lang.reflect.*;
import java.nio.*;
import java.util.*;

/** SDL3 window/input adapter used by the named 26.3+ client. */
public final class AgentSDL {
    static final Map<String,Method> METHODS=new HashMap<>();
    static final FloatBuffer X=ByteBuffer.allocateDirect(4).order(ByteOrder.nativeOrder()).asFloatBuffer(),Y=ByteBuffer.allocateDirect(4).order(ByteOrder.nativeOrder()).asFloatBuffer();
    static boolean active(){return AgentHooks.glfw!=null&&AgentHooks.glfw.getName().startsWith("org.lwjgl.sdl.");}
    static Object call(String type,String name,Class<?>[] signature,Object...args)throws Exception{String key=type+"#"+name;Method method=METHODS.get(key);if(method==null){method=Class.forName("org.lwjgl.sdl."+type,false,AgentHooks.glfw.getClassLoader()).getMethod(name,signature);METHODS.put(key,method);}return method.invoke(null,args);}
    static Object window(String name,Object...args)throws Exception{
        switch(name){
            case "glfwGetCurrentContext":return call("SDLVideo","SDL_GL_GetCurrentWindow",new Class<?>[0]);
            case "glfwGetFramebufferSize":return call("SDLVideo","SDL_GetWindowSizeInPixels",new Class<?>[]{long.class,IntBuffer.class,IntBuffer.class},args);
            case "glfwGetWindowSize":return call("SDLVideo","SDL_GetWindowSize",new Class<?>[]{long.class,IntBuffer.class,IntBuffer.class},args);
            case "glfwHideWindow":return call("SDLVideo","SDL_HideWindow",new Class<?>[]{long.class},args);
            case "glfwGetInputMode":return Boolean.TRUE.equals(call("SDLMouse","SDL_GetWindowRelativeMouseMode",new Class<?>[]{long.class},args[0]))?212995:212993;
            case "glfwSetInputMode":boolean locked=((Number)args[2]).intValue()==212995;call("SDLMouse","SDL_SetWindowRelativeMouseMode",new Class<?>[]{long.class,boolean.class},args[0],locked);call("SDLMouse",locked?"SDL_HideCursor":"SDL_ShowCursor",new Class<?>[0]);call("SDLKeyboard",locked?"SDL_StopTextInput":"SDL_StartTextInput",new Class<?>[]{long.class},args[0]);return null;
            case "glfwGetCursorPos":call("SDLMouse","SDL_GetMouseState",new Class<?>[]{FloatBuffer.class,FloatBuffer.class},X,Y);((DoubleBuffer)args[1]).put(0,X.get(0));((DoubleBuffer)args[2]).put(0,Y.get(0));return null;
            case "glfwGetClipboardString":return call("SDLClipboard","SDL_GetClipboardText",new Class<?>[0]);
            case "glfwSetWindowShouldClose":if(Boolean.TRUE.equals(args[1]))GameAccess.call(AgentHooks.game.clientObject(),"stop");return null;
            default:throw new NoSuchMethodException(name);
        }
    }
    static int key(int scan){if(scan>=4&&scan<=29)return 65+scan-4;if(scan>=30&&scan<=38)return 49+scan-30;if(scan==39)return 48;switch(scan){case 40:return 257;case 41:return 256;case 42:return 259;case 43:return 258;case 44:return 32;case 74:return 268;case 76:return 261;case 77:return 269;case 79:return 262;case 80:return 263;case 81:return 264;case 82:return 265;case 224:return 341;case 225:return 340;case 228:return 345;case 229:return 344;default:return -1;}}
    public static boolean event(boolean available,Object event){if(!available||!active()||!AgentHooks.ready)return available;try{int type=((Number)GameAccess.call(event,"type")).intValue();boolean consumed=false;long window=AgentHooks.activeWindow;
        if(type==0x300||type==0x301){Object keyboard=GameAccess.call(event,"key");int scan=((Number)GameAccess.call(keyboard,"scancode")).intValue(),mod=((Number)GameAccess.call(keyboard,"mod")).intValue(),mods=((mod&3)!=0?1:0)|((mod&0xc0)!=0?2:0)|((mod&0x300)!=0?4:0);int action=type==0x301?0:Boolean.TRUE.equals(GameAccess.call(keyboard,"repeat"))?2:1;consumed=AgentHooks.key(window,key(scan),action,mods);}
        else if(type==0x303){String text=(String)GameAccess.call(GameAccess.call(event,"text"),"textString");if(text!=null)for(int point:text.codePoints().toArray())consumed=AgentHooks.character(window,point)||consumed;}
        else if(type==0x401||type==0x402){Object button=GameAccess.call(event,"button");int nativeButton=((Number)GameAccess.call(button,"button")).intValue(),mapped=nativeButton==1?0:nativeButton==3?1:nativeButton==2?2:nativeButton-1;consumed=AgentHooks.mouse(window,mapped,type==0x401?1:0);}
        else if(type==0x403)consumed=AgentHooks.scroll(window,((Number)GameAccess.call(GameAccess.call(event,"wheel"),"y")).doubleValue());
        else if(type==0x400)consumed=AgentHooks.editor;
        if(consumed)GameAccess.call(event,"type",0);
    }catch(Throwable error){AgentHooks.log("SDL input dispatch failed",error);}return available;}
}
