; 7-Zip Hatch's separate per-user installation. Does not replace upstream 7-Zip.
#ifndef Payload
  #error Payload must be the unpacked desktop folder
#endif
#ifndef OutputDirectory
  #define OutputDirectory "..\..\..\build\packages"
#endif
#define AppVersion "0.2.0"

[Setup]
AppId=io.github.chai-in.7zip-hatch
AppName=7-Zip Hatch
AppVersion={#AppVersion}
AppPublisher=7-Zip Hatch contributors
AppPublisherURL=https://github.com/chai-in/7zip-hatch
AppSupportURL=https://github.com/chai-in/7zip-hatch/issues
AppUpdatesURL=https://github.com/chai-in/7zip-hatch/releases
DefaultDirName={localappdata}\Programs\7-Zip Hatch
DefaultGroupName=7-Zip Hatch
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
OutputDir={#OutputDirectory}
OutputBaseFilename=7zip-hatch-{#AppVersion}-windows-x64-setup
SetupIconFile=..\..\..\third_party\hatchzip\assets\icon\hatchzip.ico
WizardStyle=modern
UninstallDisplayIcon={app}\7zip-hatch.exe
UninstallDisplayName=7-Zip Hatch
InfoBeforeFile={#Payload}\Licenses\Desktop-license.txt
Compression=lzma2/max
SolidCompression=yes
ChangesAssociations=yes
CloseApplications=yes
RestartApplications=no

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; GroupDescription: "Shortcuts:"; Flags: unchecked
Name: "shellmenu"; Description: "Add 7-Zip Hatch to the archive &right-click menu"; GroupDescription: "Explorer integration:"

[Files]
Source: "{#Payload}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\7-Zip Hatch"; Filename: "{app}\7zip-hatch.exe"; WorkingDir: "{userdocs}"
Name: "{group}\7-Zip File Manager"; Filename: "{app}\7zFM.exe"; WorkingDir: "{userdocs}"
Name: "{group}\Uninstall 7-Zip Hatch"; Filename: "{uninstallexe}"
Name: "{autodesktop}\7-Zip Hatch"; Filename: "{app}\7zip-hatch.exe"; Tasks: desktopicon

[Registry]
; Alternate Open With handler, leaving the user's default archive apps alone.
Root: HKCU; Subkey: "Software\Classes\7zipHatch.Archive"; ValueType: string; ValueData: "Archive"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\7zipHatch.Archive\DefaultIcon"; ValueType: string; ValueData: """{app}\7zip-hatch.exe"",0"
Root: HKCU; Subkey: "Software\Classes\7zipHatch.Archive\shell\open\command"; ValueType: string; ValueData: """{app}\7zip-hatch.exe"" ""%1"""
Root: HKCU; Subkey: "Software\Classes\Applications\7zip-hatch.exe"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "7-Zip Hatch"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\Applications\7zip-hatch.exe\shell\open\command"; ValueType: string; ValueData: """{app}\7zip-hatch.exe"" ""%1"""
Root: HKCU; Subkey: "Software\Classes\.zip\OpenWithProgids"; ValueType: string; ValueName: "7zipHatch.Archive"; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.7z\OpenWithProgids"; ValueType: string; ValueName: "7zipHatch.Archive"; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.tar\OpenWithProgids"; ValueType: string; ValueName: "7zipHatch.Archive"; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.gz\OpenWithProgids"; ValueType: string; ValueName: "7zipHatch.Archive"; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.tgz\OpenWithProgids"; ValueType: string; ValueName: "7zipHatch.Archive"; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.001\OpenWithProgids"; ValueType: string; ValueName: "7zipHatch.Archive"; ValueData: ""; Flags: uninsdeletevalue

[Run]
Filename: "{app}\7zip-hatch.exe"; Parameters: "--register-shell"; Tasks: shellmenu; Flags: runhidden waituntilterminated
Filename: "{app}\7zip-hatch.exe"; Parameters: "--unregister-shell"; Check: RemoveDeselectedMenu; Flags: runhidden waituntilterminated
Filename: "{app}\7zip-hatch.exe"; Description: "Open 7-Zip Hatch"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{app}\7zip-hatch.exe"; Parameters: "--unregister-shell"; RunOnceId: "UnregisterShell"; Check: OwnsShellMenu; Flags: runhidden waituntilterminated

[Code]
function OwnsShellMenu: Boolean;
var
  Command: String;
begin
  Result := RegQueryStringValue(HKCU,
    'Software\Classes\SystemFileAssociations\.zip\shell\7zipHatch\shell\01open\command',
    '', Command) and (Pos('"' + ExpandConstant('{app}\7zip-hatch.exe') + '"', Command) = 1);
end;

function RemoveDeselectedMenu: Boolean;
begin
  Result := not WizardIsTaskSelected('shellmenu') and OwnsShellMenu;
end;
