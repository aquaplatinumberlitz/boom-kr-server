# Run from the extracted Windows player bundle. No permanent system service is installed.
$ErrorActionPreference = 'Stop'
$Root = $PSScriptRoot
$Principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $Principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Host 'Run this setup as administrator to add the LAN rules. See Guide > Windows Firewall if you do not have permission.'
    exit 0
}
$Applications = @('kr-player.exe', 'kr-prelogin.exe', 'friends-server\bin\kr-service.exe')
foreach ($Application in $Applications) {
    $Path = Join-Path $Root $Application
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Missing program: $Application"
    }
}
$Hasher = [Security.Cryptography.SHA256]::Create()
try {
    foreach ($Application in $Applications) {
        $Path = Join-Path $Root $Application
        $Id = 'BoomKR-O1-' + [BitConverter]::ToString($Hasher.ComputeHash([Text.Encoding]::UTF8.GetBytes($Path.ToUpperInvariant()))).Replace('-', '')
        if (-not (Get-NetFirewallRule -Name $Id -ErrorAction SilentlyContinue)) {
            New-NetFirewallRule -Name $Id -DisplayName "Boom Play LAN ($Application)" `
                -Direction Inbound -Action Allow -Program $Path -Protocol UDP -RemoteAddress LocalSubnet -Profile Any | Out-Null
        }
    }
} finally {
    $Hasher.Dispose()
}
Write-Host 'Boom Play LAN rules are installed.'
