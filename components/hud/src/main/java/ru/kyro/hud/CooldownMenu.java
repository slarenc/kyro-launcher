package ru.kyro.hud;

import java.awt.*;
import java.awt.image.BufferedImage;
import java.util.*;
import java.util.List;

/** Per-item manual donation cooldowns, edited inside the existing Right Shift menu. */
final class CooldownMenu {
    static boolean open,selected;static int field=-1,page;static String key="",name="",minutes="0",seconds="30",status="";
    static void fresh(){field=-1;key="";name="";minutes="0";seconds="30";status="";}
    static void load(PanelLayout.CooldownRule rule){key=rule.key;name=rule.label;minutes=String.valueOf((int)(rule.seconds/60));double rest=rule.seconds%60;seconds=rest==Math.rint(rest)?String.valueOf((int)rest):String.format(Locale.ROOT,"%.2f",rest).replaceAll("0+$","");field=-1;status="Предмет выбран. Укажи время и сохрани.";}
    static void input(Graphics2D g,int x,int y,int width,String value,String hint,int index){
        boolean focus=field==index;g.setColor(PanelPainter.ink(focus?30:18));g.fillRoundRect(x,y,width,24,7,7);if(focus){g.setColor(PanelPainter.ACCENT);g.drawRoundRect(x,y,width,24,7,7);}PanelPainter.text(g,value.isEmpty()?hint:value,x+7,y+6,width-14,9,value.isEmpty()?PanelPainter.QUIET:PanelPainter.WHITE,0,false);
    }
    static BufferedImage paint(){
        PanelLayout s=AgentHooks.layout;BufferedImage image=PanelPainter.canvas(320,300);Graphics2D g=PanelPainter.graphics(image);PanelPainter.glass(g,320,300,14,false);
        PanelPainter.text(g,"KYRO / КД предметов",14,10,292,13,PanelPainter.ACCENT,0,true);PanelPainter.text(g,status.isEmpty()?"Возьми донатный предмет и нажми «Из руки».":status,14,30,292,8,PanelPainter.QUIET,0,false);
        AgentHooks.button(g,12,50,"КД предметов",s.cooldowns);AgentHooks.button(g,164,50,"Название",s.cooldownNames);
        AgentHooks.button(g,12,80,"Секунды",!s.cooldownMinutes);AgentHooks.button(g,164,80,"Мин + секунды",s.cooldownMinutes);
        AgentHooks.action(g,12,110,"Из руки");AgentHooks.action(g,164,110,"Новый таймер");input(g,12,140,296,name,"Название предмета",0);
        PanelPainter.text(g,"Мин",14,177,32,9,PanelPainter.QUIET,0,false);input(g,48,170,66,minutes,"0",1);PanelPainter.text(g,"Сек",126,177,32,9,PanelPainter.QUIET,0,false);input(g,160,170,64,seconds,"30",2);
        g.setColor(PanelPainter.ink(24));g.fillRoundRect(238,170,70,24,8,8);PanelPainter.text(g,"Сохранить",243,177,60,8.5f,PanelPainter.ACCENT,0,true);
        List<PanelLayout.CooldownRule> rules=s.cooldownRules;int pages=Math.max(1,(rules.size()+1)/2);page=Math.max(0,Math.min(page,pages-1));
        if(rules.isEmpty())PanelPainter.text(g,"Своих таймеров пока нет",20,217,280,9,PanelPainter.QUIET,0,false);
        for(int row=0;row<2;row++){int index=page*2+row;if(index>=rules.size())break;PanelLayout.CooldownRule r=rules.get(index);int y=204+row*24;g.setColor(PanelPainter.ink(13));g.fillRoundRect(12,y,296,22,7,7);PanelPainter.text(g,r.label,20,y+5,169,9,PanelPainter.WHITE,0,false);PanelPainter.text(g,PanelPainter.cooldownTime((int)Math.ceil(r.seconds*20),true),192,y+5,81,8,PanelPainter.ACCENT,0,true);PanelPainter.text(g,"×",285,y+1,18,14,PanelPainter.QUIET,0,false);}
        PanelPainter.text(g,"‹ Назад",18,274,100,9,PanelPainter.WHITE,0,true);PanelPainter.text(g,"‹",133,271,18,14,PanelPainter.ACCENT,0,true);PanelPainter.text(g,(page+1)+" / "+pages,155,275,44,8,PanelPainter.QUIET,0,false);PanelPainter.text(g,"›",202,271,18,14,PanelPainter.ACCENT,0,true);PanelPainter.text(g,"Готово",252,274,55,9,PanelPainter.ACCENT,0,true);g.dispose();return image;
    }
    static boolean inside(double x,double y,int a,int b,int w,int h){return AgentHooks.inside(x,y,a,b,w,h);}
    static void click(double x,double y)throws Exception{
        PanelLayout s=AgentHooks.layout;
        if(inside(x,y,12,140,296,24)){field=0;selected=true;return;}if(inside(x,y,48,170,66,24)){field=1;selected=true;return;}if(inside(x,y,160,170,64,24)){field=2;selected=true;return;}field=-1;
        if(inside(x,y,12,50,144,24))s.cooldowns=!s.cooldowns;else if(inside(x,y,164,50,144,24))s.cooldownNames=!s.cooldownNames;
        else if(inside(x,y,12,80,144,24))s.cooldownMinutes=false;else if(inside(x,y,164,80,144,24))s.cooldownMinutes=true;
        else if(inside(x,y,12,110,144,24)){try{PanelLayout.CooldownRule item=CooldownHooks.fromMainHand(AgentHooks.game.localPlayer);if(item==null)status="Возьми нужный предмет в основную руку.";else load(item);}catch(Throwable ignored){status="Не удалось прочитать предмет из руки.";}}
        else if(inside(x,y,164,110,144,24))fresh();else if(inside(x,y,238,170,70,24))saveRule();
        else if(inside(x,y,12,268,104,24))open=false;else if(inside(x,y,240,268,70,24))AgentHooks.setEditor(false);
        else if(inside(x,y,124,268,27,24))page=Math.max(0,page-1);else if(inside(x,y,195,268,27,24))page=Math.min(Math.max(0,(s.cooldownRules.size()-1)/2),page+1);
        else for(int row=0;row<2;row++){int index=page*2+row;if(index>=s.cooldownRules.size())break;if(inside(x,y,12,204+24*row,296,22)){PanelLayout.CooldownRule item=s.cooldownRules.get(index);if(x>=278){List<PanelLayout.CooldownRule> list=new ArrayList<>(s.cooldownRules);list.remove(index);s.cooldownRules=list;CooldownHooks.forget(item.key);if(key.equals(item.key))fresh();status="Таймер удалён.";}else load(item);break;}}
        AgentHooks.save();
    }
    static void saveRule(){
        PanelLayout s=AgentHooks.layout;String label=AgentCooldowns.clean(name);double duration;
        try{duration=(minutes.isEmpty()?0:Integer.parseInt(minutes))*60.0+(seconds.isEmpty()?0:Double.parseDouble(seconds.replace(',','.')));}catch(Exception ignored){status="Укажи минуты и секунды числами.";return;}
        if(label.isEmpty()){status="Укажи название предмета.";return;}if(!Double.isFinite(duration)||duration<=0||duration>86400){status="Время: больше нуля, не больше 24 часов.";return;}
        if(key.isEmpty()){status="Сначала выбери конкретный предмет «Из руки».";return;}
        List<PanelLayout.CooldownRule> list=new ArrayList<>(s.cooldownRules);int at=-1;for(int i=0;i<list.size();i++)if(list.get(i).key.equals(key)){at=i;break;}
        if(at<0&&list.size()>=24){status="До 24 таймеров. Удали лишний.";return;}PanelLayout.CooldownRule rule=new PanelLayout.CooldownRule(key,label,duration);if(at<0){at=list.size();list.add(rule);}else list.set(at,rule);s.cooldownRules=list;CooldownHooks.forget(key);load(rule);page=at/2;status="Сохранено. Отсчёт начнётся при использовании.";AgentHooks.save();
    }
    static String value(){return field==0?name:field==1?minutes:seconds;}
    static void value(String value){if(field==0)name=value;else if(field==1)minutes=value;else if(field==2)seconds=value;}
    static boolean validInput(String value){return field==0?value.length()<=80:field==1?value.matches("[0-9]{0,4}"):value.matches("[0-9]{0,5}([.,][0-9]{0,2})?");}
    static void character(int point){
        if(field<0||!Character.isValidCodePoint(point)||Character.isISOControl(point)||Character.getType(point)==Character.PRIVATE_USE)return;String next=(selected?"":value())+new String(Character.toChars(point));if(validInput(next)){value(next);selected=false;status="";}
    }
    static void key(int code,int action,int mods){
        if(field<0||action!=1&&action!=2)return;
        if((mods&2)!=0){if(code==65)selected=true;else if(code==86)try{Object clipboard=AgentHooks.window("glfwGetClipboardString",new Class<?>[]{long.class},AgentHooks.activeWindow);if(clipboard instanceof String){String next=field==0?AgentCooldowns.clean((String)clipboard):((String)clipboard).trim();if(validInput(next)){value(next);selected=true;status="";}else status="Проверь длину названия или формат времени.";}}catch(Exception ignored){status="Не удалось вставить из буфера.";}return;}
        if(code==257||code==335){saveRule();field=-1;}else if(code==259||code==261){String old=value();value(selected?"":old.isEmpty()?"":old.substring(0,old.offsetByCodePoints(old.length(),-1)));selected=false;status="";}
    }
}
