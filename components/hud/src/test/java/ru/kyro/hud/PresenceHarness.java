package ru.kyro.hud;
import java.util.*;
import java.net.*;
public final class PresenceHarness {
    static final String A="11111111111111111111111111111111",B="22222222222222222222222222222222",C="33333333333333333333333333333333";
    static void check(boolean value,String message){if(!value)throw new AssertionError(message);}
    public static class Profile {final UUID id;final String name;Profile(UUID id,String name){this.id=id;this.name=name;}public UUID getId(){return id;}public String getName(){return name;}}
    public static class Entry {final Profile profile;Entry(Profile profile){this.profile=profile;}public Profile getProfile(){return profile;}}
    public static class Connection {public Collection<Entry> getPlayerList(){return Arrays.asList(new Entry(new Profile(UUID.fromString("22222222-2222-2222-2222-222222222222"),"SecondKyro")));}}
    public static void main(String[] args)throws Exception{
        System.setProperty("kyro.hud.player","WannaForget");Map<String,String> roster=new LinkedHashMap<>();roster.put(A,"WannaForget");roster.put(B,"SecondKyro");roster.put(C,"NoKyroHere");
        check(BadgePresence.players(new Connection()).containsValue("SecondKyro"),"Game profile roster extraction");
        try(BadgePresence a=new BadgePresence(args[0],A,"WannaForget");BadgePresence b=new BadgePresence(args[0],B,"SecondKyro")){
            a.update(true,roster);b.update(true,roster);a.syncNow();b.syncNow();a.syncNow();
            check(a.eligible().contains("SecondKyro")&&b.eligible().contains("WannaForget"),"Two clients discover each other");check(!a.eligible().contains("NoKyroHere"),"Unknown client remains unmarked");
            BadgeHooks.enter();try{check(BadgeHooksTest.visit("SecondKyro").equals("\uE100 SecondKyro"),"Discovered peer receives a badge");check(BadgeHooksTest.visit("NoKyroHere").equals("NoKyroHere"),"No badge for ordinary players");}finally{BadgeHooks.exit();}
            check(BadgeHooksTest.visit("SecondKyro").equals("SecondKyro"),"Peer badge excluded from scoreboard");
            b.close();a.syncNow();check(!a.eligible().contains("SecondKyro"),"Closing peer removes badge");
            Map<String,String> impostor=new LinkedHashMap<>();impostor.put(B,"DifferentName");a.update(true,impostor);a.syncNow();check(a.eligible().isEmpty(),"UUID/name mismatch is ignored");
            a.update(false,Collections.emptyMap());check(a.eligible().isEmpty(),"Leaving world clears peers");
        }
        try(BadgePresence mapped=new BadgePresence(args[0],A,"WannaForget");BadgePresence observer=new BadgePresence(args[0],B,"SecondKyro")){
            Map<String,String> serverRoster=new LinkedHashMap<>();serverRoster.put(C,"ServerAlias");mapped.update(true,serverRoster);mapped.game(true,null,UUID.fromString("33333333-3333-3333-3333-333333333333"));mapped.syncNow();observer.update(true,serverRoster);observer.syncNow();check(observer.eligible().contains("ServerAlias"),"Server-assigned UUID/name recognized on offline/proxy servers");
        }
        try(BadgePresence slow=new BadgePresence(args[0]+"/slow",A,"WannaForget")){
            slow.start();long before=System.nanoTime();slow.update(true,roster);check(System.nanoTime()-before<250_000_000L,"Game thread does not wait for network");Thread.sleep(100);
        }
        System.out.println("PASS: production Java clients + real Worker/SQLite, mutual badges, unknown player exclusion, UUID/name match, leaving, scoreboard exclusion, nonblocking network");
    }
}
