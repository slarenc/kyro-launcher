package ru.kyro.hud;
import java.util.*;
public final class BadgeHooksTest {
    public interface Sink {boolean accept(int index,Object style,int code);}
    public static final class Id {final String value;public Id(String namespace,String path){value=namespace+":"+path;}public String toString(){return value;}}
    public interface Font {Resource DEFAULT=new Resource(new Id("minecraft","default"));}
    public static final class Resource implements Font {final Id id;public Resource(Id id){this.id=id;}}
    public static final class Style {
        public static final Style EMPTY=new Style(Font.DEFAULT,false,0xFFFFFF);final Font font;final boolean bold;final int color;
        Style(Font font,boolean bold,int color){this.font=font;this.bold=bold;this.color=color;}
        public Style withFont(Font value){return new Style(value,bold,color);}public Style withColor(int value){return new Style(font,bold,value);}
    }
    static void check(boolean condition){if(!condition)throw new AssertionError();}
    static String visit(String text)throws Exception{
        StringBuilder output=new StringBuilder();List<Integer> indices=new ArrayList<>();Style style=new Style(new Resource(new Id("server","menu")),true,0xFF0000);
        Sink raw=(index,s,code)->{if(s!=style){Style badge=(Style)s;check(((Resource)badge.font).id.toString().equals("kyro:badge"));check(!badge.bold&&badge.color==0xFFFFFF);check(code==BadgeHooks.GLYPH||code==32);}indices.add(index);output.appendCodePoint(code);return true;};
        Sink sink=(Sink)BadgeHooks.wrap(text,raw,Sink.class);check(BadgeHooks.wrap(text,sink,Sink.class)==sink);
        for(int i=0;i<text.length();i++)sink.accept(i,style,text.charAt(i));
        for(int index:indices)check(index>=0&&index<text.length());return output.toString();
    }
    public static void main(String[] args)throws Exception{
        System.setProperty("kyro.hud.player","WannaForget");
        check(visit("WannaForget").equals("WannaForget")); // Scoreboards and unrelated screens.
        BadgeHooks.enter();
        check(visit("<WannaForget> hello").equals("<\uE100 WannaForget> hello"));
        check(visit("WannaForget").equals("\uE100 WannaForget"));
        check(visit("OtherWannaForget WannaForget123").equals("OtherWannaForget WannaForget123"));
        check(visit("\uE100 WannaForget").equals("\uE100 \uE100 WannaForget")); // Same code point in a server font is a different glyph.
        String source="[Team] WannaForget";visit(source);check(source.equals("[Team] WannaForget"));
        Sink stop=(index,style,code)->false;Sink wrapped=(Sink)BadgeHooks.wrap("WannaForget",stop,Sink.class);check(!wrapped.accept(0,Style.EMPTY,'W'));
        check(visit("\uE100 other server glyph").equals("\uE100 other server glyph"));
        BadgeHooks.exit();check(visit("WannaForget").equals("WannaForget"));
        System.out.println("PASS: name badge, exact-name boundaries, preserved source indices/style/text, no duplicate wrapping, sink cancellation");
    }
}
