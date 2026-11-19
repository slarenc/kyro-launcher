package ru.kyro.hud;
import java.nio.file.*;
import org.objectweb.asm.*;
public final class HandScaleTest {
    public enum Arm{LEFT,RIGHT} public enum Hand{MAIN_HAND,OFF_HAND}
    public static class Player{Arm arm=Arm.RIGHT;public Arm getMainArm(){return arm;}}
    public static class Pose{float value=1;double depth;public void translate(double x,double y,double z){depth=z;}public void scale(float x,float y,float z){if(x!=y||y!=z)throw new AssertionError();value=x;}}
    static void check(boolean ok){if(!ok)throw new AssertionError();}
    public static void main(String[] args)throws Exception{
        AgentHooks.layout=new PanelLayout();Player p=new Player();Pose pose=new Pose();
        AgentHooks.layout.rightHandScale=.5;AgentHooks.layout.leftHandScale=.8;
        HandHooks.apply(pose,p,Hand.MAIN_HAND);check(pose.value==.5f);
        double originalWidth=.3/.6,projectedWidth=(.3*pose.value)/(.6*pose.value-pose.depth);check(Math.abs(projectedWidth/originalWidth-.5)<.00001);HandHooks.apply(pose,p,Hand.OFF_HAND);check(pose.value==.8f);
        p.arm=Arm.LEFT;HandHooks.apply(pose,p,Hand.MAIN_HAND);check(pose.value==.8f);HandHooks.apply(pose,p,Hand.OFF_HAND);check(pose.value==.5f);
        check(PanelLayout.handScale(Double.NaN)==1&&PanelLayout.handScale(-1)==.2&&PanelLayout.handScale(2)==1);
        Path file=Files.createTempDirectory("kyro-hands").resolve("panels.json");try{AgentHooks.layout.save(file,AgentHooks.JSON);PanelLayout loaded=PanelLayout.load(file,AgentHooks.JSON);check(loaded.rightHandScale==.5&&loaded.leftHandScale==.8);}finally{Files.deleteIfExists(file);Files.deleteIfExists(file.getParent());}
        ClassWriter w=new ClassWriter(0);w.visit(Opcodes.V1_8,Opcodes.ACC_PUBLIC,"Fixture",null,"java/lang/Object",null);
        MethodVisitor m=w.visitMethod(Opcodes.ACC_PUBLIC,"renderArmWithItem","(Ljava/lang/Object;FFLjava/lang/Object;FLjava/lang/Object;FLjava/lang/Object;)V",null,null);
        // Real signature has PoseStack at argument 7 (index 7), after equipped progress.
        m.visitCode();m.visitVarInsn(Opcodes.ALOAD,8);m.visitMethodInsn(Opcodes.INVOKEVIRTUAL,"java/lang/Object","pushPose","()V",false);m.visitInsn(Opcodes.RETURN);m.visitMaxs(1,9);m.visitEnd();w.visitEnd();
        final int[] calls={0};new ClassReader(new HudTransform().hands(w.toByteArray())).accept(new ClassVisitor(Opcodes.ASM9){public MethodVisitor visitMethod(int a,String n,String d,String s,String[] e){return new MethodVisitor(Opcodes.ASM9){public void visitMethodInsn(int op,String owner,String name,String desc,boolean itf){if(owner.endsWith("/HandHooks")&&name.equals("apply"))calls[0]++;}};}},0);check(calls[0]==1);
        System.out.println("PASS: hand scales, left-handed player, persistence, bounds, perspective reduction and render injection");
    }
}
