# Run after unpacking the Windows player package to its final directory.
$ErrorActionPreference = 'Stop'
$target = Join-Path $PSScriptRoot 'kr-player.exe'
if (-not (Test-Path -LiteralPath $target -PathType Leaf)) { throw 'kr-player.exe is missing' }
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut((Join-Path $PSScriptRoot 'Boom Play.lnk'))
$shortcut.TargetPath = $target
$shortcut.Arguments = ''
$shortcut.WorkingDirectory = $PSScriptRoot
$shortcut.Save()
