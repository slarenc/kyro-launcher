package ru.kyro.hud;

import java.lang.instrument.Instrumentation;
import java.nio.file.*;
import java.util.jar.JarFile;

/** Launcher-owned game integration; no Fabric/Forge dependency or mods-folder entry. */
public final class HudAgent {
    public static void premain(String options,Instrumentation instrumentation){
        try{
            Path own=Paths.get(HudAgent.class.getProtectionDomain().getCodeSource().getLocation().toURI());
            Path runtime=own.resolveSibling("kyro-hud-runtime.jar");
            System.setProperty("kyro.hud.runtime",runtime.toString());
            instrumentation.appendToBootstrapClassLoaderSearch(new JarFile(runtime.toFile()));
            Class.forName("ru.kyro.builtin.AgentHooks",true,null).getMethod("boot").invoke(null);
            instrumentation.addTransformer(new HudTransform(),false);
            System.out.println("[KYRO HUD] Built-in overlay enabled (Java "+System.getProperty("java.version")+")");
        }catch(Throwable error){System.err.println("[KYRO HUD] Unable to initialise overlay: "+error);}
    }
}
