# Run only on a disposable Windows CI runner; this exercises real user registry entries.
param([Parameter(Mandatory)] [string] $Setup, [Parameter(Mandatory)] [string] $Payload)
$ErrorActionPreference = 'Stop'
$Setup = (Resolve-Path $Setup).Path
$Payload = (Resolve-Path $Payload).Path
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$dir = Join-Path $env:RUNNER_TEMP 'Hatch Installer App'
$menu = Join-Path ([Environment]::GetFolderPath('Programs')) '7-Zip Hatch'
$desktopLink = Join-Path ([Environment]::GetFolderPath('Desktop')) '7-Zip Hatch.lnk'
$uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\io.github.chai-in.7zip-hatch_is1'
$shellKey = 'HKCU:\Software\Classes\SystemFileAssociations\.zip\shell\7zipHatch'
$openWith = 'HKCU:\Software\Classes\.zip\OpenWithProgids'
$quiet = '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/SP-'

function Run($file, $arguments) {
    $p = Start-Process -FilePath $file -ArgumentList $arguments -PassThru
    $null = $p.Handle
    if (-not $p.WaitForExit(180000)) { $p.Kill(); throw "Timed out: $file" }
    if ($p.ExitCode) { throw "$file exited with $($p.ExitCode)" }
}

function CheckWindow($file) {
    $p = Start-Process -FilePath $file -PassThru
    try {
        foreach ($i in 1..60) {
            Start-Sleep -Milliseconds 500
            $p.Refresh()
            if ($p.HasExited) { throw "$file exited before showing a window: $($p.ExitCode)" }
            if ($p.MainWindowHandle -ne 0) { break }
        }
        if ($p.MainWindowHandle -eq 0) { throw "No desktop window for $file" }
        Write-Host "Desktop window: $($p.MainWindowTitle)"
        $null = $p.CloseMainWindow()
        if (-not $p.WaitForExit(15000)) { throw "Window did not close: $file" }
        if ($p.ExitCode) { throw "Window closed with error: $($p.ExitCode)" }
    } finally {
        if (-not $p.HasExited) { $p.Kill() }
    }
}

if ((Test-Path $dir) -or (Test-Path $uninstallKey) -or (Test-Path $shellKey)) { throw 'Requires a clean CI user' }
$env:SLINT_BACKEND = 'winit-software'
Run $Setup ($quiet + @("/DIR=`"$dir`"", '/TASKS=desktopicon,shellmenu', "/LOG=`"$env:RUNNER_TEMP/install.log`""))
Get-ChildItem $Payload -File -Recurse | ForEach-Object {
    $relative = [IO.Path]::GetRelativePath($Payload, $_.FullName)
    $installed = Join-Path $dir $relative
    if (-not (Test-Path $installed)) { throw "Missing installed file: $relative" }
    if ((Get-FileHash $_.FullName).Hash -ne (Get-FileHash $installed).Hash) { throw "Changed bytes: $relative" }
}
foreach ($name in '7-Zip Hatch.lnk', '7-Zip File Manager.lnk', 'Uninstall 7-Zip Hatch.lnk') {
    if (-not (Test-Path (Join-Path $menu $name))) { throw "Missing Start menu link: $name" }
}
if (-not (Test-Path $desktopLink)) { throw 'Missing optional desktop shortcut' }
$entry = Get-ItemProperty $uninstallKey
if ($entry.DisplayVersion -ne '0.2.0') { throw 'Wrong installed version' }
if (-not (Test-Path $shellKey)) { throw 'Missing context menu' }
if ((Get-ItemProperty $openWith).PSObject.Properties.Name -notcontains '7zipHatch.Archive') { throw 'Missing Open with handler' }
$command = (Get-Item "$shellKey\shell\01open\command").GetValue('')
if (-not $command.StartsWith("`"$dir\7zip-hatch.exe`" --open")) { throw "Wrong menu command: $command" }
python (Join-Path $root 'Hatch/test_integration.py') (Join-Path $dir '7zz.exe')
if ($LASTEXITCODE) { throw 'Installed native engine integration failed' }
CheckWindow (Join-Path $dir '7zip-hatch.exe')
CheckWindow (Join-Path $dir '7zFM.exe')

# Reinstall at the same path, then verify only installed files are removed.
$userFile = Join-Path $dir 'my archive notes.txt'
Set-Content $userFile 'User data must survive uninstall.'
Run $Setup ($quiet + @("/DIR=`"$dir`"", '/TASKS=desktopicon,shellmenu'))
Run (Join-Path $dir 'unins000.exe') $quiet
foreach ($i in 1..30) {
    if (-not (Test-Path (Join-Path $dir '7zip-hatch.exe'))) { break }
    Start-Sleep -Seconds 1
}
foreach ($path in $menu, $desktopLink, $uninstallKey, $shellKey, (Join-Path $dir '7zz.exe'), (Join-Path $dir '7zFM.exe')) {
    if (Test-Path $path) { throw "Uninstall left owned entry: $path" }
}
if ((Test-Path $openWith) -and ((Get-ItemProperty $openWith).PSObject.Properties.Name -contains '7zipHatch.Archive')) { throw 'Open with entry survived uninstall' }
if ((Get-Content $userFile) -ne 'User data must survive uninstall.') { throw 'Uninstall removed user data' }
Remove-Item $userFile
Write-Host 'Installer, upgrade, both desktop windows, engine, shortcuts and uninstall passed.'
