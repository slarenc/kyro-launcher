package ru.kyro.hud;

import java.lang.instrument.ClassFileTransformer;
import java.security.ProtectionDomain;
import org.objectweb.asm.*;

/** Stable GLFW hooks plus narrowly scoped Minecraft text decoration. */
final class HudTransform implements ClassFileTransformer {
    static final String HOOK="ru/kyro/builtin/AgentHooks";
    final String textClass;String gameModeClass="",useItemMethod="",useItemOnMethod="",componentClass="",guiClass="",skyProbeClass="",skyValueMethod="";final java.util.Set<String> scopes=new java.util.HashSet<>();
    HudTransform(){String mapped="";try{Class<?> hooks=Class.forName("ru.kyro.builtin.BadgeHooks",true,null);mapped=(String)hooks.getMethod("textClass").invoke(null);java.util.Collections.addAll(scopes,(String[])hooks.getMethod("scopeClasses").invoke(null));}catch(Exception ignored){}textClass=mapped;
        java.util.Collections.addAll(scopes,"net/minecraft/client/gui/components/ChatComponent","net/minecraft/client/gui/components/PlayerTabOverlay","net/minecraft/client/gui/hud/ChatHud","net/minecraft/client/gui/hud/PlayerListHud","net/minecraft/class_338","net/minecraft/class_355","net/minecraft/class_897","net/minecraft/class_11689","net/minecraft/client/renderer/entity/EntityRenderer","net/minecraft/client/render/entity/EntityRenderer","net/minecraft/client/renderer/feature/NameTagFeatureRenderer","net/minecraft/client/render/command/LabelCommandRenderer");
        try{Class<?> hooks=Class.forName("ru.kyro.builtin.CooldownHooks",true,null);gameModeClass=(String)hooks.getMethod("gameModeClass").invoke(null);useItemMethod=(String)hooks.getMethod("useMethod").invoke(null);useItemOnMethod=(String)hooks.getMethod("useOnMethod").invoke(null);componentClass=(String)hooks.getMethod("componentClass").invoke(null);guiClass=(String)hooks.getMethod("guiClass").invoke(null);}catch(Exception ignored){}
        try{Class<?> hooks=Class.forName("ru.kyro.builtin.SkyHooks",true,null);skyProbeClass=(String)hooks.getMethod("probeClass").invoke(null);skyValueMethod=(String)hooks.getMethod("valueMethod").invoke(null);}catch(Exception ignored){}
    }
    boolean text(String name){return name.equals(textClass)||name.equals("net/minecraft/util/StringDecomposer")||name.equals("net/minecraft/text/TextVisitFactory")||name.equals("net/minecraft/class_5223")||name.equals("net/minecraft/util/text/TextProcessing");}
    public byte[] transform(ClassLoader loader,String name,Class<?> type,ProtectionDomain domain,byte[] bytes){
        if(name.equals("org/lwjgl/sdl/SDLVideo")||name.equals("org/lwjgl/sdl/SDLEvents"))return sdl(name,bytes);
        if(name.equals(skyProbeClass)||name.equals("net/minecraft/world/attribute/EnvironmentAttributeProbe")||name.equals("net/minecraft/class_12202"))return sky(bytes);
        if(name.equals(gameModeClass)||name.equals("net/minecraft/client/multiplayer/MultiPlayerGameMode")||name.equals("net/minecraft/client/network/ClientPlayerInteractionManager")||name.equals("net/minecraft/class_636"))return cooldown(bytes);
        if(text(name))return badge(name,bytes);
        boolean messages=scopes.contains(name)||name.equals(guiClass)||name.equals("net/minecraft/client/gui/Gui")||name.equals("net/minecraft/client/gui/hud/InGameHud")||name.equals("net/minecraft/class_329");if(messages)bytes=feedback(bytes);
        if(scopes.contains(name))return scope(name,bytes);
        if(!"org/lwjgl/glfw/GLFW".equals(name))return messages?bytes:null;
        try{
            ClassReader reader=new ClassReader(bytes);ClassWriter writer=new ClassWriter(reader,ClassWriter.COMPUTE_MAXS);
            reader.accept(new ClassVisitor(Opcodes.ASM9,writer){
                public MethodVisitor visitMethod(int access,String method,String desc,String signature,String[] exceptions){
                    MethodVisitor delegate=super.visitMethod(access,method,desc,signature,exceptions);
                    if((access&(Opcodes.ACC_NATIVE|Opcodes.ACC_ABSTRACT))!=0)return delegate;
                    if(method.equals("glfwSwapBuffers")&&desc.equals("(J)V"))return new MethodVisitor(Opcodes.ASM9,delegate){
                        public void visitCode(){super.visitCode();mv.visitVarInsn(Opcodes.LLOAD,0);mv.visitLdcInsn(Type.getObjectType(name));mv.visitMethodInsn(Opcodes.INVOKESTATIC,HOOK,"frame","(JLjava/lang/Class;)V",false);}
                    };
                    String callback=null;
                    if(method.equals("glfwSetMouseButtonCallback"))callback="mouse";
                    if(method.equals("glfwSetCursorPosCallback"))callback="cursor";
                    if(method.equals("glfwSetScrollCallback"))callback="scroll";
                    if(method.equals("glfwSetKeyCallback"))callback="key";
                    if(method.equals("glfwSetCharCallback"))callback="char";
                    if(method.equals("glfwSetCharModsCallback"))callback="charMods";
                    if(callback==null||!desc.startsWith("(JLorg/lwjgl/glfw/"))return delegate;
                    final String kind=callback;final String iface=Type.getArgumentTypes(desc)[1].getInternalName();
                    return new MethodVisitor(Opcodes.ASM9,delegate){
                        public void visitCode(){super.visitCode();mv.visitVarInsn(Opcodes.LLOAD,0);mv.visitVarInsn(Opcodes.ALOAD,2);mv.visitLdcInsn(kind);mv.visitMethodInsn(Opcodes.INVOKESTATIC,HOOK,"wrap","(JLjava/lang/Object;Ljava/lang/String;)Ljava/lang/Object;",false);mv.visitTypeInsn(Opcodes.CHECKCAST,iface);mv.visitVarInsn(Opcodes.ASTORE,2);}
                    };
                }
            },0);
            System.out.println("[KYRO HUD] Attached to GLFW in "+loader);
            return writer.toByteArray();
        }catch(Throwable error){System.err.println("[KYRO HUD] GLFW hook failed: "+error);return null;}
    }
    byte[] sdl(String name,byte[] bytes){try{ClassReader reader=new ClassReader(bytes);ClassWriter writer=new ClassWriter(reader,ClassWriter.COMPUTE_MAXS);reader.accept(new ClassVisitor(Opcodes.ASM9,writer){public MethodVisitor visitMethod(int access,String method,String desc,String signature,String[] exceptions){MethodVisitor delegate=super.visitMethod(access,method,desc,signature,exceptions);if((access&(Opcodes.ACC_NATIVE|Opcodes.ACC_ABSTRACT))!=0)return delegate;if(method.equals("SDL_GL_SwapWindow")&&desc.equals("(J)Z"))return new MethodVisitor(Opcodes.ASM9,delegate){public void visitCode(){super.visitCode();mv.visitVarInsn(Opcodes.LLOAD,0);mv.visitLdcInsn(Type.getObjectType(name));mv.visitMethodInsn(Opcodes.INVOKESTATIC,HOOK,"frame","(JLjava/lang/Class;)V",false);}};if(method.equals("SDL_PollEvent")&&desc.equals("(Lorg/lwjgl/sdl/SDL_Event;)Z"))return new MethodVisitor(Opcodes.ASM9,delegate){public void visitInsn(int opcode){if(opcode==Opcodes.IRETURN){mv.visitVarInsn(Opcodes.ALOAD,0);mv.visitMethodInsn(Opcodes.INVOKESTATIC,"ru/kyro/builtin/AgentSDL","event","(ZLjava/lang/Object;)Z",false);}super.visitInsn(opcode);}};return delegate;}},0);System.out.println("[KYRO HUD] Attached to SDL3: "+name);return writer.toByteArray();}catch(Throwable error){System.err.println("[KYRO HUD] SDL hook failed: "+error);return null;}}
    byte[] sky(byte[] bytes){
        try{ClassReader reader=new ClassReader(bytes);ClassWriter writer=new ClassWriter(reader,ClassWriter.COMPUTE_MAXS);reader.accept(new ClassVisitor(Opcodes.ASM9,writer){public MethodVisitor visitMethod(int access,String method,String desc,String signature,String[] exceptions){
            MethodVisitor original=super.visitMethod(access,method,desc,signature,exceptions);Type[] args=Type.getArgumentTypes(desc);
            if((access&(Opcodes.ACC_STATIC|Opcodes.ACC_NATIVE|Opcodes.ACC_ABSTRACT))!=0||args.length!=2||args[0].getSort()!=Type.OBJECT||!args[1].equals(Type.FLOAT_TYPE)||!Type.getReturnType(desc).equals(Type.getType(Object.class))||!(method.equals("getValue")||method.equals("method_75689")||method.equals(skyValueMethod)))return original;
            return new MethodVisitor(Opcodes.ASM9,original){public void visitInsn(int opcode){if(opcode==Opcodes.ARETURN){mv.visitVarInsn(Opcodes.ALOAD,1);mv.visitMethodInsn(Opcodes.INVOKESTATIC,"ru/kyro/builtin/SkyHooks","apply","(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",false);}super.visitInsn(opcode);}};
        }},0);System.out.println("[KYRO HUD] Custom Sky environment hook attached");return writer.toByteArray();}catch(Throwable error){System.err.println("[KYRO HUD] Custom Sky hook failed: "+error);return null;}
    }
    byte[] feedback(byte[] bytes){
        try{ClassReader reader=new ClassReader(bytes);ClassWriter writer=new ClassWriter(reader,ClassWriter.COMPUTE_MAXS);reader.accept(new ClassVisitor(Opcodes.ASM9,writer){public MethodVisitor visitMethod(int access,String method,String desc,String signature,String[] exceptions){
            MethodVisitor original=super.visitMethod(access,method,desc,signature,exceptions);Type[] args=Type.getArgumentTypes(desc);if(method.startsWith("<")||(access&(Opcodes.ACC_ABSTRACT|Opcodes.ACC_NATIVE))!=0||args.length==0||args[0].getSort()!=Type.OBJECT)return original;
            String first=args[0].getInternalName();if(!(first.equals(componentClass)||first.equals("net/minecraft/network/chat/Component")||first.equals("net/minecraft/text/Text")||first.equals("net/minecraft/class_2561")))return original;final int slot=(access&Opcodes.ACC_STATIC)!=0?0:1;
            return new MethodVisitor(Opcodes.ASM9,original){public void visitCode(){super.visitCode();mv.visitVarInsn(Opcodes.ALOAD,slot);mv.visitMethodInsn(Opcodes.INVOKESTATIC,"ru/kyro/builtin/CooldownHooks","feedback","(Ljava/lang/Object;)V",false);}};
        }},0);return writer.toByteArray();}catch(Throwable ignored){return bytes;}
    }
    byte[] cooldown(byte[] bytes){
        try{ClassReader reader=new ClassReader(bytes);ClassWriter writer=new ClassWriter(reader,ClassWriter.COMPUTE_MAXS);
            reader.accept(new ClassVisitor(Opcodes.ASM9,writer){public MethodVisitor visitMethod(int access,String method,String desc,String signature,String[] exceptions){
                MethodVisitor original=super.visitMethod(access,method,desc,signature,exceptions);Type[] args=Type.getArgumentTypes(desc);
                boolean use=args.length==2&&(method.equals("useItem")||method.equals("interactItem")||method.equals("method_2919")||method.equals(useItemMethod));boolean on=args.length==3&&(method.equals("useItemOn")||method.equals("interactBlock")||method.equals("method_2896")||method.equals(useItemOnMethod));
                if((access&(Opcodes.ACC_STATIC|Opcodes.ACC_NATIVE|Opcodes.ACC_ABSTRACT))!=0||(!use&&!on)||args[0].getSort()!=Type.OBJECT||args[1].getSort()!=Type.OBJECT||Type.getReturnType(desc).getSort()!=Type.OBJECT)return original;
                return new MethodVisitor(Opcodes.ASM9,original){
                    public void visitCode(){super.visitCode();mv.visitVarInsn(Opcodes.ALOAD,1);mv.visitVarInsn(Opcodes.ALOAD,2);mv.visitMethodInsn(Opcodes.INVOKESTATIC,"ru/kyro/builtin/CooldownHooks","beforeUse","(Ljava/lang/Object;Ljava/lang/Object;)V",false);}
                    public void visitInsn(int opcode){if(opcode==Opcodes.ARETURN){mv.visitInsn(Opcodes.DUP);mv.visitMethodInsn(Opcodes.INVOKESTATIC,"ru/kyro/builtin/CooldownHooks","afterUse","(Ljava/lang/Object;)V",false);}super.visitInsn(opcode);}
                };
            }},0);return writer.toByteArray();
        }catch(Throwable error){System.err.println("[KYRO HUD] Cooldown hook failed: "+error);return null;}
    }
    byte[] scope(String name,byte[] bytes){
        try{ClassReader reader=new ClassReader(bytes);ClassWriter writer=new ClassWriter(reader,ClassWriter.COMPUTE_MAXS);
            reader.accept(new ClassVisitor(Opcodes.ASM9,writer){public MethodVisitor visitMethod(int access,String method,String desc,String signature,String[] exceptions){
                MethodVisitor original=super.visitMethod(access,method,desc,signature,exceptions);
                if(method.startsWith("<")||(access&(Opcodes.ACC_NATIVE|Opcodes.ACC_ABSTRACT))!=0)return original;
                return new MethodVisitor(Opcodes.ASM9,original){final Label start=new Label(),end=new Label(),handler=new Label();
                    public void visitCode(){super.visitCode();mv.visitMethodInsn(Opcodes.INVOKESTATIC,"ru/kyro/builtin/BadgeHooks","enter","()V",false);mv.visitLabel(start);}
                    public void visitInsn(int opcode){if(opcode>=Opcodes.IRETURN&&opcode<=Opcodes.RETURN)mv.visitMethodInsn(Opcodes.INVOKESTATIC,"ru/kyro/builtin/BadgeHooks","exit","()V",false);super.visitInsn(opcode);}
                    public void visitMaxs(int stack,int locals){mv.visitLabel(end);mv.visitTryCatchBlock(start,end,handler,"java/lang/Throwable");mv.visitLabel(handler);mv.visitFrame(Opcodes.F_FULL,0,new Object[0],1,new Object[]{"java/lang/Throwable"});mv.visitMethodInsn(Opcodes.INVOKESTATIC,"ru/kyro/builtin/BadgeHooks","exit","()V",false);mv.visitInsn(Opcodes.ATHROW);super.visitMaxs(stack,locals);}
                };
            }},0);System.out.println("[KYRO HUD] Badge render scope: "+name);return writer.toByteArray();
        }catch(Throwable error){System.err.println("[KYRO HUD] Badge scope failed: "+error);return null;}
    }
    byte[] badge(String name,byte[] bytes){
        try{ClassReader reader=new ClassReader(bytes);ClassWriter writer=new ClassWriter(reader,ClassWriter.COMPUTE_MAXS);
            reader.accept(new ClassVisitor(Opcodes.ASM9,writer){public MethodVisitor visitMethod(int access,String method,String desc,String signature,String[] exceptions){
                MethodVisitor original=super.visitMethod(access,method,desc,signature,exceptions);Type[] args=Type.getArgumentTypes(desc);
                // Decorate formatted source text only. Plain forward/backward visits
                // replay already styled glyph runs after bidi ordering; decorating
                // them again inserts a second badge when fonts split those runs.
                if((access&Opcodes.ACC_STATIC)==0||(access&(Opcodes.ACC_NATIVE|Opcodes.ACC_ABSTRACT))!=0||!Type.getReturnType(desc).equals(Type.BOOLEAN_TYPE)||args.length<4||!args[0].getDescriptor().equals("Ljava/lang/String;")||!args[1].equals(Type.INT_TYPE)||args[args.length-1].getSort()!=Type.OBJECT)return original;
                int last=0;for(int i=0;i<args.length-1;i++)last+=args[i].getSize();final int slot=last;final Type sink=args[args.length-1];
                return new MethodVisitor(Opcodes.ASM9,original){public void visitCode(){super.visitCode();mv.visitVarInsn(Opcodes.ALOAD,0);mv.visitVarInsn(Opcodes.ALOAD,slot);mv.visitLdcInsn(sink);mv.visitMethodInsn(Opcodes.INVOKESTATIC,"ru/kyro/builtin/BadgeHooks","wrap","(Ljava/lang/String;Ljava/lang/Object;Ljava/lang/Class;)Ljava/lang/Object;",false);mv.visitTypeInsn(Opcodes.CHECKCAST,sink.getInternalName());mv.visitVarInsn(Opcodes.ASTORE,slot);}};
            }},0);System.out.println("[KYRO HUD] Name badge attached to "+name);return writer.toByteArray();
        }catch(Throwable error){System.err.println("[KYRO HUD] Name badge hook failed: "+error);return null;}
    }
}
