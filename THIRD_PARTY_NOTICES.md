# Third-party notices

## Inter Variable

Inter Variable 4.1 by Rasmus Andersson is distributed under the SIL Open Font License 1.1.

- Source: https://github.com/rsms/inter/tree/v4.1
- Bundled font: `ui/fonts/InterVariable.woff2`
- License: [ui/fonts/OFL.txt](ui/fonts/OFL.txt)

Rust dependencies and their versions are recorded in `Cargo.lock`; their source packages include their respective license information. Minecraft game files, mod loaders and third-party mods are downloaded separately from their upstream services and are not included in the Windows release.


Интерфейс использует Tauri 2 (MIT / Apache-2.0) и Microsoft Edge WebView2 Runtime. Inter Variable встроен из `ui/fonts/InterVariable.woff2`, лицензия SIL OFL находится в `ui/fonts/OFL.txt`.

## Internal HUD runtime

The built-in HUD uses ASM 9.10.1 and Gson 2.10.1. Their upstream notices are retained in the runtime archive. Both dependencies are relocated into private namespaces. The old Fabric-only HUD and bundled Fabric API have been removed from the production resources.

## HUD visual reference and font assets

The built-in KYRO HUD uses the Paradox Client strip/card and island layout as a visual reference, adapted to a native Java2D renderer with KYRO branding. The two bundled HUD font files originate from the regular and semibold faces in Paradox Client 1.0.263 (build 363); their original internal font metadata is retained.

Reference: https://paradoxclient.com/download/ . No Paradox launcher, game classes, Lumen engine or native Paradox DLL is bundled with KYRO. Windows media integration is implemented in `components/media` using the Windows media-session and Core Audio APIs.
