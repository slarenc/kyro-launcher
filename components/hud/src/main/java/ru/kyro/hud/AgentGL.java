package ru.kyro.hud;

import java.awt.image.BufferedImage;
import java.lang.reflect.*;
import java.nio.*;
import java.util.*;

/** LWJGL 3 OpenGL bridge. Every changed game state is restored before swapping. */
final class AgentGL {
    final ClassLoader loader;final Map<String,Method> methods=new HashMap<>();final Map<String,Class<?>> classes=new HashMap<>();
    final int[] textures=new int[4];final int[] textureWidths=new int[4],textureHeights=new int[4];
    ByteBuffer pixels=ByteBuffer.allocateDirect(4);final FloatBuffer vertices=ByteBuffer.allocateDirect(64).order(ByteOrder.nativeOrder()).asFloatBuffer();
    final IntBuffer viewport=ByteBuffer.allocateDirect(16).order(ByteOrder.nativeOrder()).asIntBuffer();
    final ByteBuffer colorMask=ByteBuffer.allocateDirect(4);
    final int[] toggles={3042,2929,2884,3089,2960,3058,36281,35977};final boolean[] enabled=new boolean[toggles.length];
    boolean modern,samplers;int program,vao,vbo,width,height;
    int previousProgram,previousVao,previousBuffer,previousFramebuffer,previousTexture,previousActive,previousSampler,srcRgb,dstRgb,srcAlpha,dstAlpha,equationRgb,equationAlpha,matrixMode;
    boolean depthMask;
    AgentGL(ClassLoader loader)throws Exception{
        this.loader=loader;String version=(String)call("GL11","glGetString",new Class<?>[]{int.class},7938);
        String[] pieces=version.split("[. ]");int major=Integer.parseInt(pieces[0]),minor=Integer.parseInt(pieces[1]);
        modern=major>=3;samplers=major>3||major==3&&minor>=3;
        if(modern){
            String directive=major>3||minor>=2?"#version 150\n":"#version 130\n";
            int vs=shader(35633,directive+"in vec2 position;in vec2 texcoord;out vec2 uv;void main(){uv=texcoord;gl_Position=vec4(position,0,1);}");
            int fs=shader(35632,directive+"uniform sampler2D image;in vec2 uv;out vec4 outColor;void main(){outColor=texture(image,uv);}");
            program=integer("GL20","glCreateProgram");ints("GL20","glAttachShader",program,vs);ints("GL20","glAttachShader",program,fs);
            call("GL20","glBindAttribLocation",new Class<?>[]{int.class,int.class,CharSequence.class},program,0,"position");call("GL20","glBindAttribLocation",new Class<?>[]{int.class,int.class,CharSequence.class},program,1,"texcoord");
            call("GL30","glBindFragDataLocation",new Class<?>[]{int.class,int.class,CharSequence.class},program,0,"outColor");ints("GL20","glLinkProgram",program);
            if(integer("GL20","glGetProgrami",program,35714)==0)throw new IllegalStateException("HUD shader link failed");
            ints("GL20","glDeleteShader",vs);ints("GL20","glDeleteShader",fs);
            previousVao=get(34229);previousBuffer=get(34964);vao=integer("GL30","glGenVertexArrays");vbo=integer("GL15","glGenBuffers");
            ints("GL30","glBindVertexArray",vao);ints("GL15","glBindBuffer",34962,vbo);
            call("GL15","glBufferData",new Class<?>[]{int.class,long.class,int.class},34962,64L,35040);
            for(int i=0;i<2;i++){ints("GL20","glEnableVertexAttribArray",i);call("GL20","glVertexAttribPointer",new Class<?>[]{int.class,int.class,int.class,boolean.class,int.class,long.class},i,2,5126,false,16,(long)i*8);}
            ints("GL30","glBindVertexArray",previousVao);ints("GL15","glBindBuffer",34962,previousBuffer);
        }
        System.out.println("[KYRO HUD] Renderer: OpenGL "+version+(modern?" / programmable":" / compatibility"));
    }
    Object call(String owner,String name,Class<?>[] types,Object... args)throws Exception{
        String key=owner+'.'+name+Arrays.toString(types);Method m=methods.get(key);
        if(m==null){Class<?> cls=classes.get(owner);if(cls==null){cls=Class.forName("org.lwjgl.opengl."+owner,true,loader);classes.put(owner,cls);}m=cls.getMethod(name,types);methods.put(key,m);}
        return m.invoke(null,args);
    }
    Object ints(String owner,String name,int... args)throws Exception{Class<?>[] types=new Class<?>[args.length];Arrays.fill(types,int.class);Object[] values=new Object[args.length];for(int i=0;i<args.length;i++)values[i]=args[i];return call(owner,name,types,values);}
    int integer(String owner,String name,int...args)throws Exception{return ((Number)ints(owner,name,args)).intValue();}
    int get(int key)throws Exception{return integer("GL11","glGetInteger",key);}
    int shader(int type,String source)throws Exception{int id=integer("GL20","glCreateShader",type);call("GL20","glShaderSource",new Class<?>[]{int.class,CharSequence.class},id,source);ints("GL20","glCompileShader",id);if(integer("GL20","glGetShaderi",id,35713)==0)throw new IllegalStateException("HUD shader compilation failed");return id;}
    void begin(int w,int h)throws Exception{
        width=w;height=h;previousProgram=get(35725);previousActive=get(34016);ints("GL13","glActiveTexture",33984);previousTexture=get(32873);
        if(!modern){
            matrixMode=get(2976);ints("GL11","glPushAttrib",1048575);ints("GL20","glUseProgram",0);ints("GL11","glViewport",0,0,w,h);
            ints("GL11","glMatrixMode",5889);ints("GL11","glPushMatrix");ints("GL11","glLoadIdentity");call("GL11","glOrtho",new Class<?>[]{double.class,double.class,double.class,double.class,double.class,double.class},0d,(double)w,(double)h,0d,-1d,1d);
            ints("GL11","glMatrixMode",5888);ints("GL11","glPushMatrix");ints("GL11","glLoadIdentity");ints("GL11","glEnable",3553);
            for(int flag:new int[]{2929,2884,3089,2960,3058})ints("GL11","glDisable",flag);
            ints("GL11","glEnable",3042);ints("GL14","glBlendFuncSeparate",770,771,1,771);ints("GL11","glTexEnvi",8960,8704,7681);return;
        }
        previousVao=get(34229);previousBuffer=get(34964);previousFramebuffer=get(36006);
        viewport.clear();call("GL11","glGetIntegerv",new Class<?>[]{int.class,IntBuffer.class},2978,viewport);
        for(int i=0;i<toggles.length;i++)enabled[i]=(Boolean)call("GL11","glIsEnabled",new Class<?>[]{int.class},toggles[i]);
        srcRgb=get(32969);dstRgb=get(32968);srcAlpha=get(32971);dstAlpha=get(32970);equationRgb=get(32777);equationAlpha=get(34877);
        depthMask=(Boolean)call("GL11","glGetBoolean",new Class<?>[]{int.class},2930);colorMask.clear();call("GL11","glGetBooleanv",new Class<?>[]{int.class,ByteBuffer.class},3107,colorMask);
        if(samplers){previousSampler=get(35097);ints("GL33","glBindSampler",0,0);}
        ints("GL30","glBindFramebuffer",36009,0);ints("GL11","glViewport",0,0,w,h);
        for(int flag:toggles)ints("GL11","glDisable",flag);ints("GL11","glEnable",3042);
        call("GL11","glDepthMask",new Class<?>[]{boolean.class},false);call("GL11","glColorMask",new Class<?>[]{boolean.class,boolean.class,boolean.class,boolean.class},true,true,true,true);
        ints("GL14","glBlendFuncSeparate",770,771,1,771);ints("GL20","glBlendEquationSeparate",32774,32774);
        ints("GL20","glUseProgram",program);int location=((Number)call("GL20","glGetUniformLocation",new Class<?>[]{int.class,CharSequence.class},program,"image")).intValue();ints("GL20","glUniform1i",location,0);
        ints("GL30","glBindVertexArray",vao);ints("GL15","glBindBuffer",34962,vbo);
    }
    void draw(int slot,BufferedImage image,boolean dirty,float x,float y,float w,float h)throws Exception{
        if(image==null)return;
        if(textures[slot]==0){textures[slot]=integer("GL11","glGenTextures");dirty=true;}
        ints("GL11","glBindTexture",3553,textures[slot]);
        if(dirty){
            int bytes=image.getWidth()*image.getHeight()*4;if(pixels.capacity()<bytes)pixels=ByteBuffer.allocateDirect(bytes).order(ByteOrder.nativeOrder());pixels.clear();pixels.limit(bytes);pixels.asIntBuffer().put(image.getRGB(0,0,image.getWidth(),image.getHeight(),null,0,image.getWidth()));
            int pbo=get(35055),alignment=get(3317),row=get(3314),skipRows=get(3315),skipPixels=get(3316);
            ints("GL15","glBindBuffer",35052,0);ints("GL11","glPixelStorei",3317,1);ints("GL11","glPixelStorei",3314,0);ints("GL11","glPixelStorei",3315,0);ints("GL11","glPixelStorei",3316,0);
            try{
                ints("GL11","glTexParameteri",3553,10241,9729);ints("GL11","glTexParameteri",3553,10240,9729);ints("GL11","glTexParameteri",3553,10242,33071);ints("GL11","glTexParameteri",3553,10243,33071);
                if(textureWidths[slot]!=image.getWidth()||textureHeights[slot]!=image.getHeight()){
                    call("GL11","glTexImage2D",new Class<?>[]{int.class,int.class,int.class,int.class,int.class,int.class,int.class,int.class,ByteBuffer.class},3553,0,32856,image.getWidth(),image.getHeight(),0,32993,5121,pixels);textureWidths[slot]=image.getWidth();textureHeights[slot]=image.getHeight();
                }else call("GL11","glTexSubImage2D",new Class<?>[]{int.class,int.class,int.class,int.class,int.class,int.class,int.class,int.class,ByteBuffer.class},3553,0,0,0,image.getWidth(),image.getHeight(),32993,5121,pixels);
            }finally{ints("GL15","glBindBuffer",35052,pbo);ints("GL11","glPixelStorei",3317,alignment);ints("GL11","glPixelStorei",3314,row);ints("GL11","glPixelStorei",3315,skipRows);ints("GL11","glPixelStorei",3316,skipPixels);}
        }
        if(modern){
            float l=x/width*2-1,r=(x+w)/width*2-1,t=1-y/height*2,b=1-(y+h)/height*2;
            vertices.clear();vertices.put(new float[]{l,t,0,0,l,b,0,1,r,t,1,0,r,b,1,1}).flip();
            call("GL15","glBufferSubData",new Class<?>[]{int.class,long.class,FloatBuffer.class},34962,0L,vertices);ints("GL11","glDrawArrays",5,0,4);
        }else{
            ints("GL11","glBegin",7);legacyVertex(x,y,0,0);legacyVertex(x+w,y,1,0);legacyVertex(x+w,y+h,1,1);legacyVertex(x,y+h,0,1);ints("GL11","glEnd");
        }
    }
    void legacyVertex(float x,float y,float u,float v)throws Exception{call("GL11","glTexCoord2f",new Class<?>[]{float.class,float.class},u,v);call("GL11","glVertex2f",new Class<?>[]{float.class,float.class},x,y);}
    void end()throws Exception{
        if(modern){
            ints("GL30","glBindVertexArray",previousVao);ints("GL15","glBindBuffer",34962,previousBuffer);ints("GL30","glBindFramebuffer",36009,previousFramebuffer);ints("GL11","glViewport",viewport.get(0),viewport.get(1),viewport.get(2),viewport.get(3));
            for(int i=0;i<toggles.length;i++)ints("GL11",enabled[i]?"glEnable":"glDisable",toggles[i]);
            call("GL11","glDepthMask",new Class<?>[]{boolean.class},depthMask);call("GL11","glColorMask",new Class<?>[]{boolean.class,boolean.class,boolean.class,boolean.class},colorMask.get(0)!=0,colorMask.get(1)!=0,colorMask.get(2)!=0,colorMask.get(3)!=0);
            ints("GL14","glBlendFuncSeparate",srcRgb,dstRgb,srcAlpha,dstAlpha);ints("GL20","glBlendEquationSeparate",equationRgb,equationAlpha);if(samplers)ints("GL33","glBindSampler",0,previousSampler);
        }else{ints("GL11","glMatrixMode",5888);ints("GL11","glPopMatrix");ints("GL11","glMatrixMode",5889);ints("GL11","glPopMatrix");ints("GL11","glMatrixMode",matrixMode);ints("GL11","glPopAttrib");}
        ints("GL20","glUseProgram",previousProgram);ints("GL11","glBindTexture",3553,previousTexture);ints("GL13","glActiveTexture",previousActive);
    }
}
