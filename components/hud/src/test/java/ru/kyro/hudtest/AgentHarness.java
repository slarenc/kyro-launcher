package ru.kyro.hudtest;

import org.lwjgl.glfw.*;
import org.lwjgl.opengl.*;
import java.nio.*;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.lang.reflect.*;
import java.util.*;

/** Real OpenGL/callback smoke test, without a game world or user profile. */
public final class AgentHarness {
    static double pointerX,pointerY;
    static String clipboardText="#123456";public static String clipboard(long window){return clipboardText;}
    public static void pointer(long window,DoubleBuffer x,DoubleBuffer y){x.put(0,pointerX);y.put(0,pointerY);}
    static void check(boolean value,String message){if(!value)throw new AssertionError(message);}
    public static class Session {public UUID profileId(){return UUID.randomUUID();}}
    public static class Entity {public double x,y,z;public UUID id(){return UUID.randomUUID();}}
    public static class Span {public int start=0,end=600;}
    public static class Stack {public String getHoverName(){return "КД · проверка";}}
    public static class Cooldowns {public Map<String,Span> groups=new HashMap<>();public int tick;Cooldowns(){groups.put("kyro:qa",new Span());}public String getCooldownGroup(Stack stack){return "kyro:qa";}}
    public static class Player extends Entity {final Cooldowns cooldowns=new Cooldowns();public Cooldowns getCooldowns(){return cooldowns;}public Stack getMainHandItem(){return new Stack();}}
    public static class Options {public boolean hideGui;}
    public static class Client {public static final Client INSTANCE=new Client();public final Session session=new Session();public Player player;public Options options=new Options();}
    static Field field(Class<?> type,String name)throws Exception{Field f=type.getDeclaredField(name);f.setAccessible(true);return f;}
    static void scaledInput(Class<?> hooks,long window,Method mouse,Map<String,Method> windowMethods)throws Exception{
        Object layout=field(hooks,"layout").get(null),media=field(hooks,"media").get(null);Class<?> layoutType=layout.getClass(),mediaType=media.getClass();
        Method close=mediaType.getDeclaredMethod("close");close.setAccessible(true);close.invoke(media);
        Field stateField=field(mediaType,"state");Object oldState=stateField.get(media);Class<?> stateType=oldState.getClass();Constructor<?> constructor=stateType.getDeclaredConstructor();constructor.setAccessible(true);Object state=constructor.newInstance();
        field(stateType,"available").setBoolean(state,true);field(stateType,"canToggle").setBoolean(state,true);field(stateType,"canSeek").setBoolean(state,true);field(stateType,"source").set(state,"scaling-qa");field(stateType,"title").set(state,"Scaling QA");field(stateType,"duration").setLong(state,240000);stateField.set(media,state);
        @SuppressWarnings("unchecked") Queue<Object> queue=(Queue<Object>)field(mediaType,"actions").get(media);
        Field size=field(layoutType,"musicScale"),positionY=field(layoutType,"musicY"),hover=field(layoutType,"expandOnHover"),announce=field(layoutType,"announceTracks");double oldSize=size.getDouble(layout),oldY=positionY.getDouble(layout);boolean oldHover=hover.getBoolean(layout),oldAnnounce=announce.getBoolean(layout);hover.setBoolean(layout,false);announce.setBoolean(layout,false);positionY.setDouble(layout,0);
        windowMethods.put("glfwGetCursorPos",AgentHarness.class.getMethod("pointer",long.class,DoubleBuffer.class,DoubleBuffer.class));
        try{for(double scale:new double[]{.4,.65,1}){
            size.setDouble(layout,scale);field(hooks,"expand").setDouble(null,0);field(hooks,"seek").setDouble(null,-1);field(hooks,"previewUntil").setLong(null,0);GLFW.glfwSwapBuffers(window);
            double x=field(hooks,"musicX").getDouble(null),y=field(hooks,"musicY").getDouble(null),guiScale=field(hooks,"scale").getFloat(null);
            pointerX=(x+222*scale)*guiScale;pointerY=(y+24*scale)*guiScale;queue.clear();mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);Object action=queue.poll();check(action!=null&&field(action.getClass(),"action").get(action).equals("toggle"),"Play hit target follows music scale "+scale);
            pointerX=(x+74*scale)*guiScale;pointerY=(y+45*scale)*guiScale;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);action=queue.poll();check(action!=null&&field(action.getClass(),"action").get(action).equals("seek")&&Math.abs(field(action.getClass(),"value").getDouble(action)-60000)<1,"Seek hit target follows music scale "+scale);
            field(hooks,"seek").setDouble(null,-1);field(hooks,"previewUntil").setLong(null,0);pointerX=(x+90*scale)*guiScale;pointerY=(y+38*scale)*guiScale;mouse.invoke(null,window,0,1);check(field(hooks,"dragging").getInt(null)==1,"Dragging follows scaled panel bounds");mouse.invoke(null,window,0,0);
        }}finally{queue.clear();stateField.set(media,oldState);size.setDouble(layout,oldSize);positionY.setDouble(layout,oldY);hover.setBoolean(layout,oldHover);announce.setBoolean(layout,oldAnnounce);field(hooks,"seek").setDouble(null,-1);field(hooks,"previewUntil").setLong(null,0);windowMethods.remove("glfwGetCursorPos");Method save=hooks.getDeclaredMethod("save");save.setAccessible(true);save.invoke(null);}
    }
    static void gameState()throws Exception{
        Class<?> game=Class.forName("ru.kyro.builtin.AgentGame",true,null);Constructor<?> ctor=game.getDeclaredConstructor();ctor.setAccessible(true);Object instance=ctor.newInstance();
        Method bind=game.getDeclaredMethod("bind",Class.class);bind.setAccessible(true);check((Boolean)bind.invoke(instance,Client.class),"Game client binding");
        Method update=game.getDeclaredMethod("update",ClassLoader.class);update.setAccessible(true);Field world=game.getDeclaredField("world");world.setAccessible(true);
        update.invoke(instance,Client.class.getClassLoader());check(!world.getBoolean(instance),"Login-session UUID does not count as a loaded world");
        Client.INSTANCE.player=new Player();update.invoke(instance,Client.class.getClassLoader());check(world.getBoolean(instance),"A local player enables the HUD");
        Method hidden=game.getDeclaredMethod("hidden");hidden.setAccessible(true);check(!(Boolean)hidden.invoke(instance),"HUD visible by default");Client.INSTANCE.options.hideGui=true;check((Boolean)hidden.invoke(instance),"F1 hide-GUI flag respected");
        Client.INSTANCE.player=null;update.invoke(instance,Client.class.getClassLoader());check(!world.getBoolean(instance),"Leaving a world hides the HUD");
    }
    public static void main(String[] args)throws Exception{
        gameState();
        GLFWErrorCallback error=GLFWErrorCallback.createPrint(System.err);error.set();check(GLFW.glfwInit(),"GLFW initialisation");
        GLFW.glfwWindowHint(GLFW.GLFW_VISIBLE,GLFW.GLFW_FALSE);boolean core=args.length==0||args[0].equals("core");
        GLFW.glfwWindowHint(GLFW.GLFW_CONTEXT_VERSION_MAJOR,core?3:2);GLFW.glfwWindowHint(GLFW.GLFW_CONTEXT_VERSION_MINOR,core?2:1);
        if(core)GLFW.glfwWindowHint(GLFW.GLFW_OPENGL_PROFILE,GLFW.GLFW_OPENGL_CORE_PROFILE);
        long window=GLFW.glfwCreateWindow(1280,720,"KYRO built-in HUD QA",0,0);check(window!=0,"OpenGL window");
        try{
            GLFW.glfwMakeContextCurrent(window);GL.createCapabilities();GLFW.glfwSwapInterval(0);
            final int[] gameKeyPresses={0};
            final int[] gameChars={0};GLFW.glfwSetCharCallback(window,(w,c)->gameChars[0]++);GLFWCharCallback chars=GLFW.glfwSetCharCallback(window,null);GLFW.glfwSetCharCallback(window,chars);
            GLFW.glfwSetKeyCallback(window,(w,key,scan,action,mods)->{if(action==1)gameKeyPresses[0]++;});GLFW.glfwSetMouseButtonCallback(window,(w,button,action,mods)->{});GLFW.glfwSetCursorPosCallback(window,(w,x,y)->{});GLFW.glfwSetScrollCallback(window,(w,x,y)->{});
            Class<?> hooks=Class.forName("ru.kyro.builtin.AgentHooks",true,null);Method key=hooks.getMethod("key",long.class,int.class,int.class),mouse=hooks.getMethod("mouse",long.class,int.class,int.class);
            Object game=field(hooks,"game").get(null);Method bind=game.getClass().getDeclaredMethod("bind",Class.class);bind.setAccessible(true);bind.invoke(game,Client.class);Client.INSTANCE.options.hideGui=false;Client.INSTANCE.player=new Player();
            Class<?> painter=Class.forName("ru.kyro.builtin.PanelPainter",true,null);Field logo=painter.getDeclaredField("LOGO");logo.setAccessible(true);check(logo.get(null)!=null,"KYRO logo loads from the bootstrap runtime");
            Field methods=hooks.getDeclaredField("windowMethods");methods.setAccessible(true);
            @SuppressWarnings("unchecked") Map<String,Method> windowMethods=(Map<String,Method>)methods.get(null);
            int gameBuffer=GL15.glGenBuffers();long start=System.nanoTime();boolean toggled=false;
            while(!GLFW.glfwWindowShouldClose(window)&&System.nanoTime()-start<25_000_000_000L){
                Client.INSTANCE.player.cooldowns.tick=(int)((System.nanoTime()-start)/50_000_000L);
                GLFW.glfwPollEvents();GL11.glViewport(0,0,1280,720);GL11.glClearColor(.08f,.17f,.24f,1);GL11.glClear(GL11.GL_COLOR_BUFFER_BIT);
                GL13.glActiveTexture(GL13.GL_TEXTURE3);GL15.glBindBuffer(GL15.GL_ARRAY_BUFFER,gameBuffer);GL11.glDisable(GL11.GL_BLEND);GL11.glEnable(GL11.GL_SCISSOR_TEST);GL11.glScissor(0,0,1280,720);
                GLFW.glfwSwapBuffers(window);
                check(GL11.glGetInteger(GL13.GL_ACTIVE_TEXTURE)==GL13.GL_TEXTURE3,"Active texture state restored");check(GL11.glGetInteger(GL15.GL_ARRAY_BUFFER_BINDING)==gameBuffer,"Vertex buffer state restored");check(!GL11.glIsEnabled(GL11.GL_BLEND)&&GL11.glIsEnabled(GL11.GL_SCISSOR_TEST),"Blend/scissor state restored");
                check(GL11.glGetError()==GL11.GL_NO_ERROR,"No OpenGL errors after the overlay");
                if(!toggled&&System.nanoTime()-start>2_000_000_000L){
                    GLFW.glfwSetInputMode(window,GLFW.GLFW_CURSOR,GLFW.GLFW_CURSOR_DISABLED);
                    key.invoke(null,window,344,1);check(hooks.getField("editor").getBoolean(null),"Right Shift opens editor");
                    check(gameKeyPresses[0]==0,"Opening HUD must not press inventory, chat, or other game keys");
                    check(GLFW.glfwGetInputMode(window,GLFW.GLFW_CURSOR)==GLFW.GLFW_CURSOR_NORMAL,"HUD releases the cursor directly");
                    windowMethods.put("glfwGetCursorPos",AgentHarness.class.getMethod("pointer",long.class,DoubleBuffer.class,DoubleBuffer.class));
                    pointerX=488;pointerY=216;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);
                    pointerX=792;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);
                    Path settings=Paths.get(System.getProperty("kyro.hud.config"),"kyro-panels.json");String saved=new String(Files.readAllBytes(settings),StandardCharsets.UTF_8);check(saved.contains("\"info\": false")&&saved.contains("\"music\": false"),"Both panels can be disabled independently in Right Shift");
                    pointerX=488;pointerY=438;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);saved=new String(Files.readAllBytes(settings),StandardCharsets.UTF_8);check(saved.contains("\"cooldowns\": false"),"Cooldown toggle in shared Right Shift menu");mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);pointerY=216;
                    pointerX=488;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);pointerX=792;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);windowMethods.remove("glfwGetCursorPos");
                    windowMethods.put("glfwGetCursorPos",AgentHarness.class.getMethod("pointer",long.class,DoubleBuffer.class,DoubleBuffer.class));
                    pointerX=816;pointerY=330;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);
                    saved=new String(Files.readAllBytes(settings),StandardCharsets.UTF_8);check(saved.contains("\"musicScale\": 0.6"),"Music scale minus button is saved");
                    pointerX=892;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);windowMethods.remove("glfwGetCursorPos");
                    saved=new String(Files.readAllBytes(settings),StandardCharsets.UTF_8);check(saved.contains("\"musicScale\": 0.65"),"Music scale plus button restores the compact default");
                    windowMethods.put("glfwGetCursorPos",AgentHarness.class.getMethod("pointer",long.class,DoubleBuffer.class,DoubleBuffer.class));pointerX=816;pointerY=386;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);
                    saved=new String(Files.readAllBytes(settings),StandardCharsets.UTF_8);check(saved.contains("\"infoScale\": 0.75"),"Island minus button is saved");pointerX=892;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);windowMethods.remove("glfwGetCursorPos");
                    saved=new String(Files.readAllBytes(settings),StandardCharsets.UTF_8);check(saved.contains("\"infoScale\": 0.8"),"Island plus button restores compact size");
                    windowMethods.put("glfwGetCursorPos",AgentHarness.class.getMethod("pointer",long.class,DoubleBuffer.class,DoubleBuffer.class));pointerX=816;pointerY=500;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);saved=new String(Files.readAllBytes(settings),StandardCharsets.UTF_8);check(saved.contains("\"cooldownScale\": 0.75"),"Cooldown minus button saves smaller size");pointerX=892;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);saved=new String(Files.readAllBytes(settings),StandardCharsets.UTF_8);check(saved.contains("\"cooldownScale\": 0.8"),"Cooldown plus button restores compact default");pointerX=792;pointerY=556;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);check(field(hooks,"skyEditor").getBoolean(null),"Custom Sky settings open from shared menu");
                    pointerX=570;pointerY=268;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);windowMethods.put("glfwGetClipboardString",AgentHarness.class.getMethod("clipboard",long.class));Method keyMods=hooks.getMethod("key",long.class,int.class,int.class,int.class);keyMods.invoke(null,window,86,1,2);
                    saved=new String(Files.readAllBytes(settings),StandardCharsets.UTF_8);check(saved.contains("\"skyColor\": 1193046")&&saved.contains("\"customSky\": true"),"Ctrl+V applies and saves HEX colour");
                    for(char c:"ABCDEF".toCharArray())key.invoke(null,window,(int)c,1);saved=new String(Files.readAllBytes(settings),StandardCharsets.UTF_8);check(saved.contains("\"skyColor\": 11259375"),"Typing replaces selected HEX without sending game keys");chars.invoke(window,'a');check(gameChars[0]==0,"HEX input does not type into underlying game screens");key.invoke(null,window,257,1);check(hooks.getField("editor").getBoolean(null),"Enter confirms HEX without closing settings");
                    pointerX=488;pointerY=604;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);check(!field(hooks,"skyEditor").getBoolean(null),"Back returns to shared menu");pointerY=556;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);saved=new String(Files.readAllBytes(settings),StandardCharsets.UTF_8);check(saved.contains("\"customSky\": false"),"Custom Sky can be disabled independently");windowMethods.remove("glfwGetClipboardString");windowMethods.remove("glfwGetCursorPos");
                    windowMethods.put("glfwGetCursorPos",AgentHarness.class.getMethod("pointer",long.class,DoubleBuffer.class,DoubleBuffer.class));pointerX=792;pointerY=438;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);
                    Class<?> cooldownMenu=Class.forName("ru.kyro.builtin.CooldownMenu",true,null);check(field(cooldownMenu,"open").getBoolean(null),"Cooldown editor opens from shared menu");
                    pointerX=488;pointerY=336;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);check(!((String)field(cooldownMenu,"key").get(null)).isEmpty(),"Concrete held item is captured");
                    pointerX=488;pointerY=396;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);for(char c:"Донат".toCharArray())chars.invoke(window,c);check(field(cooldownMenu,"name").get(null).equals("Донат")&&gameChars[0]==0,"Cyrillic item label is entered without typing into Minecraft");
                    pointerY=456;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);chars.invoke(window,'1');pointerX=700;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);chars.invoke(window,'2');chars.invoke(window,'0');pointerX=860;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);
                    saved=new String(Files.readAllBytes(settings),StandardCharsets.UTF_8);check(saved.contains("Донат")&&saved.contains("\"seconds\": 80.0"),"Manual minutes and seconds saved through real GLFW callbacks");
                    pointerX=488;pointerY=652;mouse.invoke(null,window,0,1);mouse.invoke(null,window,0,0);check(!field(cooldownMenu,"open").getBoolean(null),"Cooldown back button returns to shared menu");windowMethods.remove("glfwGetCursorPos");
                    scaledInput(hooks,window,mouse,windowMethods);
                    key.invoke(null,window,344,1);check(!hooks.getField("editor").getBoolean(null),"Right Shift closes editor");
                    check(GLFW.glfwGetInputMode(window,GLFW.GLFW_CURSOR)==GLFW.GLFW_CURSOR_DISABLED,"Closing HUD restores the captured cursor");
                    check(gameKeyPresses[0]==0,"Closing HUD does not send game key presses");toggled=true;
                    chars.invoke(window,'b');check(gameChars[0]==1,"Character callback passes through after editor closes");
                }
                Thread.sleep(10);
            }
            check(toggled,"Editor controls exercised");Field failed=hooks.getDeclaredField("failed");failed.setAccessible(true);check(!failed.getBoolean(null),"Renderer stayed healthy");
            Object gl=field(hooks,"gl").get(null);check(((int[])field(gl.getClass(),"textures").get(gl))[3]!=0,"Cooldown rows uploaded to fourth real GL texture");
            Field errors=hooks.getDeclaredField("errors");errors.setAccessible(true);check(((Set<?>)errors.get(null)).isEmpty(),"No callback or rendering fallbacks");
            Path reports=Paths.get(System.getProperty("kyro.hud.capture"));check(Files.exists(reports.resolve("agent-2.png")),"All real OpenGL previews were captured");
            System.out.println("PASS: built-in agent, real GLFW callbacks, isolated dependencies, OpenGL state, Right Shift toggles, saved settings, captures; Java "+System.getProperty("java.version")+" / "+args[0]);
        }finally{GLFW.glfwDestroyWindow(window);GLFW.glfwTerminate();error.free();}
    }
}
