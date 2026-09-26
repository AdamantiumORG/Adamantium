#define ProductName "Adamantium"
#define ProductVersion GetEnv("ADAMANTIUM_VERSION")
#ifndef SourceRoot
  #define SourceRoot "..\dist\adamantium-windows-x86_64"
#endif

[Setup]
AppId={{8D79EE6E-53D0-47C8-A173-C5836473C57A}
AppName={#ProductName}
AppVersion={#ProductVersion}
DefaultDirName={autopf}\Adamantium
DefaultGroupName=Adamantium
OutputDir=..\dist
OutputBaseFilename=adamantium-windows-x86_64-installer
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
ChangesEnvironment=yes
LicenseFile=..\LICENSE.md

[Files]
Source: "{#SourceRoot}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Registry]
Root: HKA; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Check: NeedsAddPath(ExpandConstant('{app}'))

[Icons]
Name: "{group}\Adamantium terminal"; Filename: "{cmd}"; Parameters: "/K cd /D {app}"

[Code]
function NeedsAddPath(Path: string): Boolean;
var
  CurrentPath: string;
begin
  if not RegQueryStringValue(HKEY_AUTO, 'Environment', 'Path', CurrentPath) then
    CurrentPath := '';
  Result := Pos(';' + Uppercase(Path) + ';', ';' + Uppercase(CurrentPath) + ';') = 0;
end;

