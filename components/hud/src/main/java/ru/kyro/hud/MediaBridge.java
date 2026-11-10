package ru.kyro.hud;

import com.google.gson.*;
import javax.imageio.ImageIO;
import java.awt.image.BufferedImage;
import java.io.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.Base64;
import java.util.concurrent.*;

/** Line-delimited JSON over a private child process; never block the render thread. */
final class MediaBridge implements AutoCloseable {
    static final class Action {
        final String action,source;final double value;
        Action(String action,double value,String source){this.action=action;this.value=value;this.source=source;}
        String action(){return action;}String source(){return source;}double value(){return value;}
    }
    static final class State {
        boolean available,playing,canToggle,canNext,canPrevious,canSeek;
        String title="",artist="",source="",error="",artwork;
        long position,duration,artRevision; double volume=-1; int protocol;
    }
    volatile State state=new State();
    volatile BufferedImage cover;
    volatile long receivedAt=System.nanoTime(),changedAt=System.nanoTime();
    volatile boolean enabled=true,closed;
    volatile Process process;
    volatile long requestAt;
    private volatile double desiredVolume=Double.NaN;
    private volatile long volumeUntil;
    private volatile String volumeSource="";
    private final ConcurrentLinkedQueue<Action> actions=new ConcurrentLinkedQueue<>();
    private final ScheduledExecutorService watchdog=Executors.newSingleThreadScheduledExecutor(r->{Thread t=new Thread(r,"KYRO media watchdog");t.setDaemon(true);return t;});
    private final Path helper;
    MediaBridge(Path helper){this.helper=helper;}
    void start(){
        watchdog.scheduleWithFixedDelay(()->{Process p=process;if(p!=null&&requestAt!=0&&System.nanoTime()-requestAt>8_000_000_000L)p.destroyForcibly();},1,1,TimeUnit.SECONDS);
        Thread thread=new Thread(this::run,"KYRO media bridge");thread.setDaemon(true);thread.start();
    }
    private void run(){
        while(!closed){
            try{
                if(!Files.isRegularFile(helper))throw new IOException("Не найден медиамост KYRO");
                process=new ProcessBuilder(helper.toAbsolutePath().toString()).redirectError(new File(System.getProperty("os.name","").toLowerCase(java.util.Locale.ROOT).contains("win")?"NUL":"/dev/null")).start();
                long revision=0;String lastKey="";
                try(BufferedReader in=new BufferedReader(new InputStreamReader(process.getInputStream(),StandardCharsets.UTF_8));
                    BufferedWriter out=new BufferedWriter(new OutputStreamWriter(process.getOutputStream(),StandardCharsets.UTF_8))){
                    while(!closed){
                        if(!enabled){actions.clear();Thread.sleep(500);continue;}
                        Action action=actions.poll();JsonObject request=new JsonObject();request.addProperty("action",action==null?"poll":action.action());
                        request.addProperty("artRevision",revision);
                        if(action!=null){request.addProperty("session",action.source());if(action.action().equals("seek"))request.addProperty("value",(long)action.value());else request.addProperty("value",action.value());}
                        requestAt=System.nanoTime();out.write(new Gson().toJson(request));out.newLine();out.flush();
                        String line=in.readLine();requestAt=0;if(line==null)throw new IOException("Медиапанель перезапускается");
                        State next=new Gson().fromJson(line,State.class);
                        if(next==null||next.protocol!=2)throw new IOException("Несовместимый медиамост KYRO");
                        next.title=safe(next.title);next.artist=safe(next.artist);next.source=safe(next.source);next.error=safe(next.error);
                        String key=next.available?next.source+"\0"+next.title+"\0"+next.artist:"";
                        if(!key.equals(lastKey)){cover=null;changedAt=System.nanoTime();lastKey=key;}
                        if(next.artwork!=null){cover=decode(next.artwork);revision=next.artRevision;next.artwork=null;}
                        if(!next.available){cover=null;revision=0;}
                        receivedAt=System.nanoTime();state=next;
                        // Controls queued while reading are dispatched without another polling delay.
                        for(int i=0;i<10&&actions.isEmpty()&&!closed;i++)Thread.sleep(25);
                    }
                }
            }catch(Exception e){if(!closed){State failure=new State();failure.error=e.getMessage()==null?"Медиапанель недоступна":e.getMessage();state=failure;cover=null;actions.clear();}}
            finally{requestAt=0;Process p=process;if(p!=null)p.destroyForcibly();process=null;}
            if(!closed)try{Thread.sleep(2000);}catch(InterruptedException e){Thread.currentThread().interrupt();return;}
        }
    }
    static String safe(String value){return value==null?"":value;}
    static BufferedImage decode(String encoded){
        if(encoded.isEmpty()||encoded.length()>6*1024*1024)return null;
        try(javax.imageio.stream.ImageInputStream stream=ImageIO.createImageInputStream(new ByteArrayInputStream(Base64.getDecoder().decode(encoded)))){
            java.util.Iterator<javax.imageio.ImageReader> readers=ImageIO.getImageReaders(stream);if(!readers.hasNext())return null;
            javax.imageio.ImageReader reader=readers.next();
            try{reader.setInput(stream);int w=reader.getWidth(0),h=reader.getHeight(0);if(w<=0||h<=0||w>8192||h>8192)return null;
                javax.imageio.ImageReadParam params=reader.getDefaultReadParam();int subsample=Math.max(1,Math.max(w,h)/256);params.setSourceSubsampling(subsample,subsample,0,0);return reader.read(0,params);
            }finally{reader.dispose();}
        }catch(Exception e){return null;}
    }
    PanelPainter.Track snapshot(){
        State s=state;double position=s.position/1000.0;
        if(s.playing)position+=Math.max(0,System.nanoTime()-receivedAt)/1_000_000_000.0;
        if(s.duration>0)position=Math.min(position,s.duration/1000.0);
        double volume=s.source.equals(volumeSource)&&System.nanoTime()<volumeUntil?desiredVolume:s.volume;
        return new PanelPainter.Track(s.available,s.playing,s.canToggle,s.canPrevious,s.canNext,s.canSeek,s.title,s.artist,s.source,s.error,position,s.duration/1000.0,volume,cover);
    }
    synchronized void command(String action,double value){
        State current=state;if(!current.available||actions.size()>=16)return;
        if(action.equals("toggle")&&!current.canToggle||action.equals("next")&&!current.canNext||action.equals("previous")&&!current.canPrevious||action.equals("seek")&&!current.canSeek)return;
        if(action.equals("volume")){
            if(current.volume<0)return;
            value=PanelPainter.clamp(value);desiredVolume=value;volumeSource=current.source;volumeUntil=System.nanoTime()+1_000_000_000L;
            actions.removeIf(queued->queued.action().equals("volume"));
        }
        actions.offer(new Action(action,value,current.source));
    }
    double trackSeconds(){return Math.max(0,System.nanoTime()-changedAt)/1_000_000_000.0;}
    public void close(){closed=true;watchdog.shutdownNow();Process p=process;if(p!=null)p.destroyForcibly();}
}
