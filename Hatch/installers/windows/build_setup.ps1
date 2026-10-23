param([Parameter(Mandatory)] [string] $Payload)
$ErrorActionPreference = 'Stop'
$Payload = (Resolve-Path $Payload).Path
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$output = Join-Path $root 'build/packages'
$iscc = Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6/ISCC.exe'
if (-not (Test-Path $iscc)) { throw 'Install Inno Setup 6 from https://jrsoftware.org/isdl.php first' }
foreach ($file in '7zip-hatch.exe', '7zz.exe', '7zFM.exe', 'Licenses/GPL-3.0.txt') {
    if (-not (Test-Path (Join-Path $Payload $file))) { throw "Missing payload file: $file" }
}
New-Item -ItemType Directory -Force $output | Out-Null
Copy-Item (Join-Path (Split-Path $iscc) 'license.txt') (Join-Path $Payload 'Licenses/Inno-Setup.txt')
(Get-Item $iscc).VersionInfo | Format-List ProductVersion, FileVersion
& $iscc /Qp "/DPayload=$Payload" "/DOutputDirectory=$output" (Join-Path $PSScriptRoot 'setup.iss')
if ($LASTEXITCODE) { throw "Inno Setup failed: $LASTEXITCODE" }
