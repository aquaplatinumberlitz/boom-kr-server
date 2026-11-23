# Optional developer helper; players can use Settings > Create shortcut.
$ErrorActionPreference = 'Stop'
$target = Join-Path $PSScriptRoot 'LANBoom.exe'
if (-not (Test-Path -LiteralPath $target -PathType Leaf)) { throw 'LANBoom.exe is missing' }
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut((Join-Path $shell.SpecialFolders.Item('Desktop') 'LANBoom.lnk'))
$shortcut.TargetPath = $target
$shortcut.WorkingDirectory = $PSScriptRoot
$shortcut.IconLocation = "$target,0"
$shortcut.Save()
