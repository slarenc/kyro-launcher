; Build with ../build-installer.ps1. The desktop shortcut is mandatory.
#ifndef AppVersion
  #error AppVersion must be provided by build-installer.ps1
#endif
#ifndef SourceExe
  #error SourceExe must be provided by build-installer.ps1
#endif

[Setup]
#ifdef TestRoot
AppId=KyroLauncher.InstallerSmokeTest
AppName=KyroLauncher Installer Smoke Test
DefaultDirName={#TestRoot}\App
OutputBaseFilename=KyroLauncherSetup-Test
#else
AppId={{9B095ABB-AD05-4CF4-8905-00AE189F699B}
AppName=KyroLauncher
DefaultDirName={localappdata}\Programs\KyroLauncher
OutputBaseFilename=KyroLauncherSetup
#endif
AppVersion={#AppVersion}
AppPublisher=KyroLauncher
AppPublisherURL=https://github.com/slarenc/kyro-launcher
AppSupportURL=https://github.com/slarenc/kyro-launcher/issues
AppUpdatesURL=https://github.com/slarenc/kyro-launcher/releases
VersionInfoVersion={#AppVersion}.0
VersionInfoDescription=KyroLauncher Setup
DefaultGroupName=KyroLauncher
DisableProgramGroupPage=yes
DisableWelcomePage=no
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
WizardStyle=modern
SetupIconFile=..\src-tauri\icons\icon.ico
UninstallDisplayIcon={app}\KyroLauncher.exe
Compression=lzma2
SolidCompression=yes
CloseApplications=no
RestartApplications=no
SetupLogging=yes
OutputDir=..\dist

[Languages]
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; DestName: "KyroLauncher.exe"; Flags: ignoreversion
Source: "..\src-tauri\assets\Inter-LICENSE.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\THIRD_PARTY_NOTICES.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "README.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
#ifdef TestRoot
Name: "{#TestRoot}\Desktop\KyroLauncher"; Filename: "{app}\KyroLauncher.exe"; WorkingDir: "{app}"; IconFilename: "{app}\KyroLauncher.exe"; Comment: "Minecraft Launcher"; AppUserModelID: "KyroLauncher.Desktop"
Name: "{#TestRoot}\StartMenu\KyroLauncher"; Filename: "{app}\KyroLauncher.exe"; WorkingDir: "{app}"; IconFilename: "{app}\KyroLauncher.exe"; AppUserModelID: "KyroLauncher.Desktop"
#else
Name: "{autodesktop}\KyroLauncher"; Filename: "{app}\KyroLauncher.exe"; WorkingDir: "{app}"; IconFilename: "{app}\KyroLauncher.exe"; Comment: "Minecraft Launcher"; AppUserModelID: "KyroLauncher.Desktop"
Name: "{autoprograms}\KyroLauncher"; Filename: "{app}\KyroLauncher.exe"; WorkingDir: "{app}"; IconFilename: "{app}\KyroLauncher.exe"; AppUserModelID: "KyroLauncher.Desktop"
#endif

[Run]
Filename: "{app}\KyroLauncher.exe"; Description: "{cm:LaunchProgram,KyroLauncher}"; Flags: nowait postinstall skipifsilent

; No wildcard deletion: worlds, accounts and user-created files must survive uninstall.
