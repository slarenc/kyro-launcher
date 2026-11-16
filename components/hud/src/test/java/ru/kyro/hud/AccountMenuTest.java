package ru.kyro.hud;
import com.google.gson.*;
import java.nio.file.*;
import java.util.*;
public final class AccountMenuTest {
 public static void main(String[] args)throws Exception{
  Path dir=Files.createTempDirectory("kyro-accounts-test");System.setProperty("kyro.accounts.root",dir.toString());
  JsonObject catalog=new JsonObject();catalog.addProperty("time",System.currentTimeMillis());JsonArray accounts=new JsonArray();
  for(int i=0;i<3;i++){JsonObject a=new JsonObject();a.addProperty("name","Player"+i);a.addProperty("uuid","uuid"+i);a.addProperty("key","offline:uuid"+i);accounts.add(a);}catalog.add("accounts",accounts);AccountMenu.write(dir.resolve("catalog.json"),catalog);
  JsonObject live=new JsonObject();live.addProperty("uuid","uuid0");live.addProperty("time",System.currentTimeMillis());live.addProperty("online",true);live.addProperty("health",17);live.addProperty("armor",12);live.addProperty("world","minecraft:overworld");live.addProperty("mode","Survival");live.add("inventory",new JsonArray());AccountMenu.write(dir.resolve("client-0/state.json"),live);
  JsonObject stale=live.deepCopy();stale.addProperty("uuid","uuid1");stale.addProperty("time",System.currentTimeMillis()-60000);AccountMenu.write(dir.resolve("client-1/state.json"),stale);
  AccountMenu.refresh();check(AccountMenu.rows.size()==3,"All saved accounts");check(AccountMenu.rows.get(0).online,"Live session");check(!AccountMenu.rows.get(1).alive,"Stale session not online");
  AccountMenu.send(AccountMenu.rows.get(0),"control");check(AccountMenu.str(AccountMenu.read(dir.resolve("client-0/command.json")),"action").equals("control"),"Control targets correct client");check(!Files.exists(dir.resolve("client-1/command.json")),"Other client untouched");
  AccountMenu.server="localhost:25565";AccountMenu.request("launch",AccountMenu.rows.get(2));JsonObject request=AccountMenu.read(dir.resolve("requests").resolve(AccountMenu.request));check(AccountMenu.str(request,"key").equals("offline:uuid2"),"Play as chooses account");check(AccountMenu.str(request,"server").equals("localhost:25565"),"Server preserved");
  check(!AccountMenu.validId("../outside"),"Traversal rejected");AccountMenu.page=99;AccountMenu.paint();check(AccountMenu.page==1,"Page bounded after account list changes");
  System.out.println("PASS: account session expiry, routing, launch identity/server, pagination");
 }
 static void check(boolean condition,String message){if(!condition)throw new AssertionError(message);}
}
