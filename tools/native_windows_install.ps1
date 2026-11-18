param([ValidateRange(30,600)][int]$WaitSeconds=300,[switch]$Resume,[switch]$DiagnosticLauncher,[switch]$SocketProbe,[switch]$NetworkControl,[switch]$WarmCompanion)
# Private native candidate installer; no native success inferred from readiness.
$ErrorActionPreference='Stop'
$report=[ordered]@{schema='kre-m8-native-windows-install-v2';start_utc=[DateTime]::UtcNow.ToString('o');phase='starting';limits='Installation/process observations only. KR login/gameplay needs native window/wire/account/journal evidence.'}
$reportPath=$null
function Save-Report {
 if($reportPath){[IO.File]::WriteAllText($reportPath,($report | ConvertTo-Json -Depth 10))}
}
try {
 $output=Get-Volume | Where-Object FileSystemLabel -eq 'M8SETUP' | Select-Object -First 1
 if(-not $output){throw 'Writable output media missing'}
 $reportPath=$output.DriveLetter+':\native-windows-install.json'
 $media=Get-Volume | Where-Object FileSystemLabel -eq 'M8PLAYER' | Select-Object -First 1
 if(-not $media){throw 'Player input media missing'}
 $seed=$media.DriveLetter+':\'
 $root='C:\KRM8Player'
 if((Test-Path -LiteralPath $root) -and -not $Resume){throw 'Owned installation destination already exists'}
 if($Resume -and -not (Test-Path -LiteralPath $root)){throw 'Resume installation missing'}
 $identity=[Security.Principal.WindowsIdentity]::GetCurrent()
 $principal=New-Object Security.Principal.WindowsPrincipal($identity)
 if(-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'Installer requires an elevated console for the owned hosts mapping'}
 if(-not $Resume){
  $report.phase='copying';Save-Report
  New-Item -ItemType Directory -Path $root | Out-Null
  Copy-Item -LiteralPath (Join-Path $seed 'game') -Destination (Join-Path $root 'game') -Recurse
  foreach($name in @('kr-launch.exe','kr-companion.exe','certificate.der','player-config.json','game-manifest.json','credentials.json')){
   Copy-Item -LiteralPath (Join-Path $seed ('scripts\'+$name)) -Destination (Join-Path $root $name)
  }
 } else {
  # Resume only the installed input from the same media. Endpoint/IP relocation
  # is allowed for player-config; executable, trust and credential bytes are not.
  foreach($name in @('kr-launch.exe','kr-companion.exe','certificate.der','game-manifest.json','credentials.json')){
   $installed=(Get-FileHash -LiteralPath (Join-Path $root $name) -Algorithm SHA256).Hash
   $inputHash=(Get-FileHash -LiteralPath (Join-Path $seed ('scripts\'+$name)) -Algorithm SHA256).Hash
   if($installed -ne $inputHash){throw 'Resume input identity mismatch'}
  }
  $report.resumed_verified_install=$true
 }
 # Copying from optical media preserves ReadOnly. This is the owned mutable
 # installation, including native metadata caches and relocated config.
 Get-ChildItem -LiteralPath $root -Recurse -File | ForEach-Object {$_.IsReadOnly=$false}
 $report.phase='checking';Save-Report
 $manifest=Get-Content -LiteralPath (Join-Path $root 'game-manifest.json') -Raw | ConvertFrom-Json
 $checked=0
 foreach($entry in $manifest.files){
  $path=Join-Path (Join-Path $root 'game') $entry.path.Replace('/','\')
  $file=Get-Item -LiteralPath $path
  if($file.Length -ne $entry.bytes -or (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.sha256){throw 'Installed game data does not match the declared input'}
  $checked++
 }
 $deploy=Get-Content -LiteralPath (Join-Path $seed 'scripts\deployment-private.json') -Raw | ConvertFrom-Json
 $hosts=Join-Path $env:SystemRoot 'System32\drivers\etc\hosts'
 [IO.File]::AppendAllText($hosts,"`r`n"+$deploy.server+" lslist.crazyarcade.com dream-login-ca.nexon.com happy-login-ca.nexon.com`r`n")
 $report.install=[ordered]@{files_checked=$checked;exe_sha256=(Get-FileHash -LiteralPath (Join-Path $root 'game\CA.exe') -Algorithm SHA256).Hash.ToLowerInvariant();launcher_sha256=(Get-FileHash -LiteralPath (Join-Path $root 'kr-launch.exe') -Algorithm SHA256).Hash.ToLowerInvariant()}
 $address=Get-NetIPAddress -AddressFamily IPv4 | Where-Object {$_.IPAddress -ne '127.0.0.1' -and $_.IPAddress -notlike '169.254.*'} | Select-Object -First 1
 if(-not $address){throw 'Guest IPv4 missing'}
 $configPath=Join-Path $root 'player-config.json'
 # Derive every resume from the pinned media config. Diagnostic wrappers must
 # not accumulate in an installed config left by an earlier measurement.
 $config=Get-Content -LiteralPath (Join-Path $seed 'scripts\player-config.json') -Raw | ConvertFrom-Json
 # Login metadata describes the guest NIC; companion receives the native
 # packets on loopback. These addresses have different ownership and purpose.
 $config | Add-Member -NotePropertyName reported_login_ipv4 -NotePropertyValue $address.IPAddress -Force
 $config.native_ip='127.0.0.1'
 if($DiagnosticLauncher){
  $controlRoot=$output.DriveLetter+':\'
  $control=Get-Content -LiteralPath (Join-Path $controlRoot 'launcher-control.json') -Raw | ConvertFrom-Json
  if($control.PSObject.Properties.Name -contains 'companion_sha256'){
   $companionPath=Join-Path $controlRoot 'kr-companion-control.exe'
   $companionHash=(Get-FileHash -LiteralPath $companionPath -Algorithm SHA256).Hash.ToLowerInvariant()
   if($control.companion_sha256 -notmatch '^[a-f0-9]{64}$' -or $companionHash -ne $control.companion_sha256){throw 'Diagnostic companion identity mismatch'}
   $config.companion=$companionPath
   $report.diagnostic_companion=[ordered]@{sha256=$companionHash;installed_companion_unchanged=$true}
  }
 }
 if($SocketProbe){
  $controlRoot=$output.DriveLetter+':\'
  $probe=Get-Content -LiteralPath (Join-Path $controlRoot 'socket-probe-control.json') -Raw | ConvertFrom-Json
  $probePath=Join-Path $controlRoot 'socket-probe.exe'
  $probeHash=(Get-FileHash -LiteralPath $probePath -Algorithm SHA256).Hash.ToLowerInvariant()
  if($probe.sha256 -notmatch '^[a-f0-9]{64}$' -or $probeHash -ne $probe.sha256 -or $probe.native_sha256 -ne $report.install.exe_sha256){throw 'Socket probe input identity mismatch'}
  # The debugger owns the native child and kills it if the launcher stops the
  # probe. No installed binary or game manifest is changed by this control.
  $nativePath=$config.native_program
  if(-not [IO.Path]::IsPathRooted($nativePath)){$nativePath=Join-Path $root $nativePath}
  $nativePath=(Resolve-Path -LiteralPath $nativePath).Path
  $config.native_arguments=@((Join-Path $controlRoot 'native-socket-probe.jsonl'),$nativePath)+@($config.native_arguments)
  $config.native_program=$probePath
  $report.socket_probe=[ordered]@{sha256=$probeHash;limits='Owned debug child; software breakpoints may perturb behavior. Not installed launcher/native acceptance.'}
 }
 [IO.File]::WriteAllText($configPath,($config | ConvertTo-Json -Depth 5))
 if($WarmCompanion){
  $warm=New-Object Diagnostics.ProcessStartInfo
  $warm.FileName=$config.companion
  if(-not [IO.Path]::IsPathRooted($warm.FileName)){$warm.FileName=Join-Path $root $warm.FileName}
  $warm.WorkingDirectory=$root;$warm.UseShellExecute=$false
  $warm.RedirectStandardInput=$true;$warm.RedirectStandardOutput=$true;$warm.RedirectStandardError=$true
  $child=New-Object Diagnostics.Process;$child.StartInfo=$warm
  $watch=[Diagnostics.Stopwatch]::StartNew()
  if(-not $child.Start()){throw 'Companion startup control did not start'}
  $child.StandardInput.Write('{}');$child.StandardInput.Close()
  if(-not $child.WaitForExit(45000)){$child.Kill();$child.WaitForExit();throw 'Companion startup control deadline'}
  $watch.Stop();$stdout=$child.StandardOutput.ReadToEnd();$stderr=$child.StandardError.ReadToEnd()
  $report.companion_startup_control=[ordered]@{elapsed_ms=$watch.ElapsedMilliseconds;exit=$child.ExitCode;stdout=$stdout;stderr=$stderr;limits='Malformed-input startup only; no network authentication or native acceptance.'}
  if($child.ExitCode -ne 1 -or $stdout -ne '' -or ($stderr | ConvertFrom-Json).stage -ne 'installation'){throw 'Companion startup control unexpected result'}
 }
 $report.phase='install_ready';Save-Report
 Write-Output ('M8_INSTALL_READY files='+$checked)
 $deadline=[DateTime]::UtcNow.AddSeconds($WaitSeconds)
 $online=$false
 while([DateTime]::UtcNow -lt $deadline){
  $socket=New-Object Net.Sockets.TcpClient
  try {
   $pending=$socket.BeginConnect($deploy.server,[int]$deploy.tcp_ports[-1],$null,$null)
   if($pending.AsyncWaitHandle.WaitOne(400)){$socket.EndConnect($pending);$online=$true}
  } catch { $online=$false } finally { $socket.Close() }
  if($online){break}
  Start-Sleep -Milliseconds 600
 }
 if(-not $online){throw 'Bounded launcher service wait expired before native start'}
 if($NetworkControl){
  # One plaintext reachability marker, never an account capability or native
  # packet. The peer service should reject it; captures establish delivery only.
  $udp=New-Object Net.Sockets.UdpClient
  try {
   $bytes=[Text.Encoding]::ASCII.GetBytes('KRM8-UDP-CONTROL1')
   $sent=$udp.Send($bytes,$bytes.Length,$deploy.server,[int]$deploy.udp_ports[-1])
   $report.network_control=[ordered]@{udp_bytes_sent=$sent;limits='UDP API return is not packet delivery or QUIC authentication.'}
  } finally {$udp.Close()}
 }
 $start=New-Object Diagnostics.ProcessStartInfo
 $start.FileName=Join-Path $root 'kr-launch.exe';$start.Arguments='player-config.json';$start.WorkingDirectory=$root
 if($DiagnosticLauncher){
  # Explicit measurement binary from writable seed; installed launcher stays
  # unchanged. This diagnostic cannot certify the installed player package.
  $controlRoot=$output.DriveLetter+':\'
  $control=Get-Content -LiteralPath (Join-Path $controlRoot 'launcher-control.json') -Raw | ConvertFrom-Json
  $controlPath=Join-Path $controlRoot 'kr-launch-control.exe'
  $controlHash=(Get-FileHash -LiteralPath $controlPath -Algorithm SHA256).Hash.ToLowerInvariant()
  if($control.sha256 -notmatch '^[a-f0-9]{64}$' -or $controlHash -ne $control.sha256){throw 'Diagnostic launcher identity mismatch'}
  $start.FileName=$controlPath
  $start.Arguments='C:\KRM8Player\player-config.json'
  $report.diagnostic_launcher=[ordered]@{sha256=$controlHash;installed_launcher_unchanged=$true;limits='Measurement binary from seed; not installed package acceptance'}
 }
 $start.UseShellExecute=$false;$start.RedirectStandardInput=$true;$start.RedirectStandardOutput=$true;$start.RedirectStandardError=$true
 $process=New-Object Diagnostics.Process;$process.StartInfo=$start
 if(-not $process.Start()){throw 'Player launcher did not start'}
 $process.StandardInput.Write([IO.File]::ReadAllText((Join-Path $root 'credentials.json')));$process.StandardInput.Close()
 $report.launcher_pid=$process.Id;$report.native_attempt_utc=[DateTime]::UtcNow.ToString('o');$report.phase='launcher_running';Save-Report
 $deadline=[DateTime]::UtcNow.AddSeconds(240)
 while(-not $process.HasExited -and [DateTime]::UtcNow -lt $deadline){Start-Sleep -Milliseconds 250}
 if(-not $process.HasExited){
  $report.observation_deadline=$true
  $owned=@(Get-CimInstance Win32_Process | Where-Object {$_.ExecutablePath -eq (Join-Path $root 'game\CA.exe') -and $_.ParentProcessId -eq $process.Id})
  $report.owned_native=@($owned | Select-Object ProcessId,ParentProcessId,ExecutablePath)
  foreach($row in $owned){$native=Get-Process -Id $row.ProcessId -ErrorAction SilentlyContinue;if($native){$native.CloseMainWindow() | Out-Null}}
  if(-not $process.WaitForExit(15000)){$report.forced_launcher_stop=$true;$process.Kill();$process.WaitForExit()}
 }
 $report.launcher_exit=$process.ExitCode;$report.stdout=$process.StandardOutput.ReadToEnd();$report.stderr=$process.StandardError.ReadToEnd()
 $report.phase='finished';$report.ended_utc=[DateTime]::UtcNow.ToString('o');Save-Report
 Write-Output ('M8_LAUNCHER_EXIT '+$process.ExitCode)
} catch {
 $report.error=($_ | Out-String);$report.phase='failed';Save-Report
 Write-Output ('M8_CANDIDATE_FAILED '+$report.error)
}
