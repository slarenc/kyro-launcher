package ru.kyro.hud;

import com.google.gson.*;
import java.io.*;
import java.net.*;
import java.nio.charset.StandardCharsets;
import java.lang.reflect.*;
import java.util.*;
import java.util.concurrent.*;

/** Bounded background presence; no game credentials or server addresses leave the JVM. */
final class BadgePresence implements AutoCloseable {
    final String endpoint,id,name;volatile String desiredId;String currentId,currentName;volatile boolean active,closed;volatile Map<String,String> roster=Collections.emptyMap();volatile Set<String> rosterNames=Collections.emptySet();volatile Object gameConnection;
    final Map<String,Long> confirmed=new ConcurrentHashMap<>();Set<String> published=Collections.emptySet();String lease="";Thread worker;
    BadgePresence(){this(System.getProperty("kyro.hud.presence",""),System.getProperty("kyro.hud.uuid",""),System.getProperty("kyro.hud.player",""));}
    BadgePresence(String endpoint,String id,String name){
        String safe="";try{URI uri=new URI(endpoint);if(uri.getUserInfo()==null&&("https".equals(uri.getScheme())||"http".equals(uri.getScheme())&&Arrays.asList("127.0.0.1","localhost").contains(uri.getHost())))safe=endpoint.replaceAll("/+$","");}catch(Exception ignored){}
        this.endpoint=safe;this.id=uuid(id);this.name=name;desiredId=this.id;currentId=this.id;currentName=name;
    }
    static String uuid(String value){String id=value==null?"":value.replace("-","").toLowerCase(Locale.ROOT);return id.matches("[0-9a-f]{32}")?id:"";}
    synchronized void start(){if(worker!=null||endpoint.isEmpty()||id.isEmpty()||!name.matches("[A-Za-z0-9_]{3,16}"))return;worker=new Thread(()->{
        long next=0;try{while(!closed){long now=System.nanoTime();if(active&&now>=next){try{Object connection=gameConnection;if(connection!=null)setRoster(players(connection));if(active)syncNow();}catch(Exception ignored){}next=System.nanoTime()+20_000_000_000L;}else if(!active&&!lease.isEmpty()){leave();next=0;}publish();Thread.sleep(500);}}
        catch(InterruptedException stopped){}finally{leave();}
    },"KYRO player status");worker.setDaemon(true);worker.start();}
    void update(boolean playing,Map<String,String> players){
        setRoster(players);active=playing;if(!playing)confirmed.clear();publish();
    }
    void setRoster(Map<String,String> players){
        LinkedHashMap<String,String> safe=new LinkedHashMap<>();for(Map.Entry<String,String> player:players.entrySet()){String key=uuid(player.getKey()),value=player.getValue();if(!key.isEmpty()&&value!=null&&value.matches("[A-Za-z0-9_]{3,16}")){safe.put(key,value);if(safe.size()==256)break;}}
        roster=Collections.unmodifiableMap(safe);rosterNames=Collections.unmodifiableSet(new HashSet<>(safe.values()));
    }
    void game(boolean playing,Object connection,Object playerId){gameConnection=connection;String actual=uuid(String.valueOf(playerId));desiredId=playing&&!actual.isEmpty()?actual:id;active=playing;if(!playing){roster=Collections.emptyMap();rosterNames=Collections.emptySet();confirmed.clear();publish();}}
    static Map<String,String> players(Object connection){
        if(connection==null)return Collections.emptyMap();try{
            Object entries=null;for(String alias:new String[]{"getOnlinePlayers","getListedOnlinePlayers","getPlayerList","method_2880",BadgeHooks.mapped("connectionPlayersMethod")})if(!alias.isEmpty())try{entries=connection.getClass().getMethod(alias).invoke(connection);if(entries instanceof Collection)break;}catch(Exception ignored){}
            if(!(entries instanceof Collection))return Collections.emptyMap();LinkedHashMap<String,String> result=new LinkedHashMap<>();
            for(Object entry:(Collection<?>)entries){if(entry==null)continue;Object profile=AgentGame.call(entry,new String[]{"getProfile","method_2966"});if(profile==null)for(Method method:entry.getClass().getMethods())if(method.getParameterTypes().length==0&&method.getReturnType().getName().equals("com.mojang.authlib.GameProfile")){profile=method.invoke(entry);break;}
                if(profile==null)continue;Object id=AgentGame.call(profile,new String[]{"getId","id"}),name=AgentGame.call(profile,new String[]{"getName","name"});if(id instanceof UUID&&name instanceof String)result.put(id.toString(),(String)name);if(result.size()==256)break;
            }return result;
        }catch(Exception ignored){return Collections.emptyMap();}
    }
    synchronized Set<String> eligible(){long now=System.nanoTime();HashSet<String> names=new HashSet<>();for(Map.Entry<String,Long> player:confirmed.entrySet())if(player.getValue()>now&&rosterNames.contains(player.getKey()))names.add(player.getKey());return names;}
    synchronized void publish(){Set<String> names=active?eligible():Collections.emptySet();if(!names.equals(published)){published=names;BadgeHooks.setPeers(names);}}
    void syncNow()throws Exception{
        Map<String,String> requested=roster;String actualId=desiredId,actualName=requested.getOrDefault(actualId,name);if(!actualId.equals(currentId)||!actualName.equals(currentName)){leave();currentId=actualId;currentName=actualName;}
        JsonObject body=new JsonObject();body.addProperty("uuid",currentId);body.addProperty("name",currentName);if(!lease.isEmpty())body.addProperty("lease",lease);JsonArray peers=new JsonArray();for(String peer:requested.keySet())peers.add(peer);body.add("peers",peers);
        JsonObject response=post("/v1/sync",body);if(response==null){lease="";return;}if(!response.has("protocol")||response.get("protocol").getAsInt()!=1)return;
        String token=response.get("lease").getAsString();if(!token.matches("[0-9a-f]{64}"))return;lease=token;Map<String,Long> next=new HashMap<>();long now=System.nanoTime();JsonArray players=response.getAsJsonArray("players");if(players==null||players.size()>512)return;
        for(JsonElement element:players){JsonObject player=element.getAsJsonObject();String key=uuid(player.get("uuid").getAsString()),value=player.get("name").getAsString();int ttl=player.get("ttl").getAsInt();if(value.equals(requested.get(key))&&ttl>0&&ttl<=75)next.put(value,now+ttl*1_000_000_000L);}
        confirmed.clear();confirmed.putAll(next);publish();
    }
    JsonObject post(String path,JsonObject body)throws Exception{
        HttpURLConnection connection=(HttpURLConnection)new URL(endpoint+path).openConnection();try{
            connection.setInstanceFollowRedirects(false);connection.setConnectTimeout(2500);connection.setReadTimeout(2500);connection.setRequestMethod("POST");connection.setRequestProperty("Content-Type","application/json");connection.setRequestProperty("User-Agent","KYRO-Presence/1");connection.setDoOutput(true);
            byte[] bytes=new Gson().toJson(body).getBytes(StandardCharsets.UTF_8);connection.setFixedLengthStreamingMode(bytes.length);try(OutputStream out=connection.getOutputStream()){out.write(bytes);}
            int status=connection.getResponseCode();if(status==401)return null;if(status!=200)throw new IOException("Presence HTTP "+status);
            try(InputStream in=connection.getInputStream();ByteArrayOutputStream out=new ByteArrayOutputStream()){byte[] buffer=new byte[4096];int count;while((count=in.read(buffer))>=0){if(out.size()+count>65536)throw new IOException("Presence response too large");out.write(buffer,0,count);}return new Gson().fromJson(new String(out.toByteArray(),StandardCharsets.UTF_8),JsonObject.class);}
        }finally{connection.disconnect();}
    }
    void leave(){if(lease.isEmpty())return;String token=lease;lease="";try{JsonObject body=new JsonObject();body.addProperty("lease",token);post("/v1/leave",body);}catch(Exception ignored){}}
    public void close(){active=false;closed=true;confirmed.clear();publish();if(worker!=null){worker.interrupt();try{worker.join(3000);}catch(InterruptedException interrupted){Thread.currentThread().interrupt();}}else leave();}
}
