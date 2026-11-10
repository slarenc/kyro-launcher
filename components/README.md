# Built-in KYRO components

- `hud`: launcher-owned HUD runtime. It is attached automatically to the game JVM and has no Fabric/Forge dependency or `mods` installation step. Its private runtime and dependencies are isolated from the game's class namespaces.
- `media`: Windows media-session and per-application audio bridge, started and stopped with the game.

Build both with `node tools/build-hud.cjs`. See [HUD behaviour and checks](../docs/kyro-panels.md).
