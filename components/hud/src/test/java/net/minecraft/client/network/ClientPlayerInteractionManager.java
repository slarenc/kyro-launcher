package net.minecraft.client.network;
/** Exercises the production transformer and its return-value stack preservation. */
public final class ClientPlayerInteractionManager {
    public Object interactItem(Object player,Object hand){return hand;}
    public Object interactBlock(Object player,Object hand,Object hit){return hit;}
}
