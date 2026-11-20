package ru.kyro.hud;
import java.lang.reflect.*;
import java.util.*;
public class PingTest {
 public static class Player { public UUID getUUID(){return new UUID(0,1);} }
 public static class Entry { public int getLatency(){return 83;} }
 public static class Connection {
  boolean missing;
  public Object getEntity(UUID id){return new Object();}
  public Entry getPlayerInfo(UUID id){return missing?null:new Entry();}
 }
 public static class Client {
  public Player player=new Player();public Connection connection=new Connection();
  public Connection getConnection(){return connection;}
 }
 static void set(Object target,String name,Object value)throws Exception {Field f=AgentGame.class.getDeclaredField(name);f.setAccessible(true);f.set(target,value);}
 static void check(boolean ok,String message){if(!ok)throw new AssertionError(message);}
 public static void main(String[] args)throws Exception {
  AgentGame game=new AgentGame();Client client=new Client();set(game,"client",client);set(game,"players",Arrays.asList(Client.class.getField("player")));
  game.update(PingTest.class.getClassLoader());check(game.ping.equals("83 ms"),"Player info latency");
  client.connection.missing=true;game.update(PingTest.class.getClassLoader());check(game.ping.equals("83 ms"),"Temporary missing info retains ping");
  set(game,"pingAt",System.nanoTime()-6_000_000_000L);game.update(PingTest.class.getClassLoader());check(game.ping.equals("— ms"),"Stale ping expires");
  client.connection=new Connection();game.update(PingTest.class.getClassLoader());client.connection=new Connection();client.connection.missing=true;game.update(PingTest.class.getClassLoader());check(game.ping.equals("— ms"),"New connection clears old ping");
  client.player=null;game.update(PingTest.class.getClassLoader());check(!game.world&&game.ping.equals("— ms"),"Disconnect clears ping");
  System.out.println("PASS: ping lookup, temporary gaps, expiry, reconnect, disconnect");
 }
}
