package ru.kyro.hud;
import java.nio.file.*;
import org.objectweb.asm.*;
public final class HandScaleTest {
    public enum Context{NONE,THIRD_LEFT,THIRD_RIGHT,FIRST_LEFT,FIRST_RIGHT,HEAD,GUI,GROUND,FIXED}
    public static class Pose{float value=1;double x=.56,y=-.52,z=-.72;java.util.Deque<Float> saved=new java.util.ArrayDeque<>();public void pushPose(){saved.push(value);}public void popPose(){value=saved.pop();}public void scale(float x,float y,float z){if(x!=y||y!=z)throw new AssertionError();value*=x;}}
    static void check(boolean ok){if(!ok)throw new AssertionError();}
    public static void main(String[] args)throws Exception{
        AgentHooks.layout=new PanelLayout();Pose pose=new Pose();
        AgentHooks.layout.rightHandScale=.5;AgentHooks.layout.leftHandScale=.8;
        HandHooks.beginContext(pose,Context.FIRST_RIGHT);check(pose.value==.5f&&pose.x==.56&&pose.y==-.52&&pose.z==-.72);HandHooks.end();check(pose.value==1);
        HandHooks.beginContext(pose,Context.FIRST_LEFT);check(pose.value==.8f);HandHooks.end();check(pose.value==1);
        for(Context context:new Context[]{Context.GUI,Context.THIRD_RIGHT,Context.THIRD_LEFT,Context.GROUND}){HandHooks.beginContext(pose,context);check(pose.value==1);HandHooks.end();}
        check(PanelLayout.handScale(Double.NaN)==1&&PanelLayout.handScale(-1)==.2&&PanelLayout.handScale(2)==1);
        Path file=Files.createTempDirectory("kyro-hands").resolve("panels.json");try{AgentHooks.layout.save(file,AgentHooks.JSON);PanelLayout loaded=PanelLayout.load(file,AgentHooks.JSON);check(loaded.rightHandScale==.5&&loaded.leftHandScale==.8);}finally{Files.deleteIfExists(file);Files.deleteIfExists(file.getParent());}
        ClassWriter w=new ClassWriter(0);w.visit(Opcodes.V1_8,Opcodes.ACC_PUBLIC,"Fixture",null,"java/lang/Object",null);
        MethodVisitor m=w.visitMethod(Opcodes.ACC_PUBLIC,"renderItem","(Ljava/lang/Object;Ljava/lang/Object;Ljava/lang/Object;Lcom/mojang/blaze3d/vertex/PoseStack;Ljava/lang/Object;I)V",null,null);
        m.visitCode();m.visitInsn(Opcodes.RETURN);m.visitMaxs(0,7);m.visitEnd();w.visitEnd();
        final int[] calls={0};new ClassReader(new HudTransform().hands(w.toByteArray())).accept(new ClassVisitor(Opcodes.ASM9){public MethodVisitor visitMethod(int a,String n,String d,String sig,String[] e){return new MethodVisitor(Opcodes.ASM9){public void visitMethodInsn(int op,String owner,String name,String desc,boolean itf){if(owner.endsWith("/HandHooks")&&name.equals("beginContext"))calls[0]++;}};}},0);check(calls[0]==1);
        check(new HudTransform().transform(null,"net/minecraft/client/renderer/entity/player/PlayerRenderer",null,null,w.toByteArray())==null);
        check(new HudTransform().transform(null,"net/minecraft/class_1007",null,null,w.toByteArray())==null);
        check(pose.saved.isEmpty());
        System.out.println("PASS: item scales, player arm renderers untouched, physical sides, isolated GUI/third person, balanced stack, persistence, bounds, fixed position and render injection");
    }
}
