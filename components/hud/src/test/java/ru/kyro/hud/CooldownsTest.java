package ru.kyro.hud;
import java.util.*;
import java.nio.file.*;
import com.google.gson.Gson;
public final class CooldownsTest {
    public static final class Lore {public List<String> lines(){return Collections.singletonList("КД: 30 сек");}}
    public static final class DonationStack {int count=1;public Object get(net.minecraft.core.component.DataComponents.Token token){return token==net.minecraft.core.component.DataComponents.LORE?new Lore():null;}public String getHoverName(){return "Телепорт";}public Object getItem(){return "minecraft:stick";}public int getCount(){return count;}}
    public enum Hand {MAIN}
    public static final class Donor {final DonationStack stack=new DonationStack();public DonationStack getItemInHand(Hand hand){return stack;}}
    public static final class Result {final boolean accepted;Result(boolean accepted){this.accepted=accepted;}public boolean consumesAction(){return accepted;}}
    public static final class Span {int start,end;Span(int a,int b){start=a;end=b;}}
    public static final class Stack {final String key,name;Stack(String key,String name){this.key=key;this.name=name;}public String getHoverName(){return name;}}
    public static final class Manager {Map<String,Span> groups=new LinkedHashMap<>();int tick;public String getCooldownGroup(Stack stack){return stack.key;}public float getCooldownPercent(Stack stack,float partial){return 0;}}
    public static final class Player {final Manager manager=new Manager();Stack stack=new Stack("plugin:teleport","§d\uE100 Телепорт");public Manager getCooldowns(){return manager;}public Stack getMainHandItem(){return stack;}}
    static void check(boolean ok,String message){if(!ok)throw new AssertionError(message);}
    public static void main(String[] args)throws Exception{
        check(CooldownHooks.duration("§7КД: 30 сек.")==30_000_000_000L,"Russian lore seconds");
        check(CooldownHooks.duration("Задержка использования: 1,5 минуты")==90_000_000_000L,"decimal minute lore");
        check(CooldownHooks.duration("Cooldown: 2.5 s")==2_500_000_000L,"English lore");
        check(CooldownHooks.duration("Перезарядка: 1 мин. 30 сек.")==90_000_000_000L,"Combined minutes and seconds");
        check(CooldownHooks.duration("Урон: 500") == 0&&CooldownHooks.duration("КД: 999999 с")==0,"No guessed damage or invalid duration");
        Donor donor=new Donor();CooldownHooks.reset();CooldownHooks.merge(donor,Collections.emptyList());CooldownHooks.beforeUse(donor,Hand.MAIN);CooldownHooks.afterUse(new Result(false));check(CooldownHooks.merge(donor,Collections.emptyList()).isEmpty(),"PASS or rejected use alone does not start a lore timer");
        CooldownHooks.afterUse(new Result(true));check(CooldownHooks.merge(donor,Collections.emptyList()).size()==1,"Accepted use starts real lore fixture timer");
        CooldownHooks.feedback("Подождите, осталось 20 секунд КД");check(CooldownHooks.merge(donor,Collections.emptyList()).isEmpty(),"Server rejection cancels just-started local timer");
        CooldownHooks.reset();CooldownHooks.merge(donor,Collections.emptyList());CooldownHooks.beforeUse(donor,Hand.MAIN);donor.stack.count=0;check(CooldownHooks.merge(donor,Collections.emptyList()).size()==1,"Server inventory consumption confirms delayed custom item use");
        AgentCooldowns reader=new AgentCooldowns();Player player=new Player();player.manager.tick=110;player.manager.groups.put("plugin:teleport",new Span(100,160));
        List<AgentCooldowns.Entry> rows=reader.sample(player);check(rows.size()==1&&rows.get(0).remaining==50&&rows.get(0).total==60,"Actual remaining ticks");
        check(rows.get(0).label.equals("Телепорт"),"Custom name without server font glyphs");player.stack=new Stack("other","Другое");
        check(reader.sample(player).get(0).label.equals("Телепорт"),"Name survives hand change");check(reader.sample(player).get(0).remaining==50,"Paused ticks stay paused");
        player.manager.tick=160;check(reader.sample(player).isEmpty(),"Expired native cooldown hidden");
        CooldownHooks.reset();CooldownHooks.merge(player,Collections.emptyList());
        CooldownHooks.Candidate c=new CooldownHooks.Candidate(player,null,null,"custom","Телепорт",1,30_000_000_000L);
        CooldownHooks.pending=c;CooldownHooks.start();List<AgentCooldowns.Entry> estimated=CooldownHooks.merge(player,Collections.emptyList());check(estimated.size()==1&&estimated.get(0).estimated,"Lore timer marked as estimate");
        AgentCooldowns.Entry exact=new AgentCooldowns.Entry("native","Телепорт",30,60);check(CooldownHooks.merge(player,Collections.singletonList(exact)).size()==1,"Native timer takes precedence");
        check(CooldownHooks.merge(new Player(),Collections.emptyList()).isEmpty(),"World/player switch clears lore timers");
        check(PanelPainter.cooldowns(estimated).getHeight()==42&&PanelPainter.cooldowns(Collections.emptyList())==null,"Compact painter and empty state");
        Path dir=Files.createTempDirectory("kyro-cooldowns"),file=dir.resolve("panels.json");PanelLayout layout=new PanelLayout();check(layout.cooldowns,"Enabled default");layout.cooldowns=false;layout.save(file,new Gson());check(!PanelLayout.load(file,new Gson()).cooldowns,"Toggle persisted");Files.delete(file);Files.delete(dir);
        System.out.println("PASS: native cooldown ticks, custom groups/names, lore units, bounds, estimates, precedence, reset, painter and saved toggle");
    }
}
