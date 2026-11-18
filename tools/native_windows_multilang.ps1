# Parameterized M8 control for an already verified vi/en/ko installation.
# Inputs, credentials and endpoints are private seed files; source installation is retained.
$ErrorActionPreference='Stop'
$output=Get-Volume|Where-Object FileSystemLabel -eq 'M8SETUP'|Select-Object -First 1
$seed=$output.DriveLetter+':\';$reportPath=Join-Path $seed 'native-windows-install.json'
$control=Get-Content -LiteralPath (Join-Path $seed 'multilang-run.json') -Raw|ConvertFrom-Json
if($control.language -notin @('vi','en','ko') -or $control.runtime_name -notmatch '^M8Native[A-Za-z0-9_-]{1,40}$' -or $control.observe_seconds -lt 30 -or $control.observe_seconds -gt 240){throw 'Multilingual control bounds'}
$report=[ordered]@{schema='kre-m8-multilang-windows-native-v2';start_utc=[DateTime]::UtcNow.ToString('o');phase='starting';language=$control.language;limits='Existing verified language inputs with declared loader-control exe; original L1/installation unchanged. Runtime/wire/UI evidence required.'}
function Save-Report {[IO.File]::WriteAllText($reportPath,($report|ConvertTo-Json -Depth 12))}
$werKey='HKLM:\SOFTWARE\Microsoft\Windows\Windows Error Reporting\LocalDumps\CA.exe'
$ownedWer=$false
$localCapture=$null
try{
 $game=Join-Path 'C:\' $control.runtime_name;if(Test-Path -LiteralPath $game){throw 'Owned native runtime already exists'}
 New-Item -ItemType Directory -Path $game|Out-Null
 $inputs=Get-Content -LiteralPath (Join-Path $seed 'language-inputs.json') -Raw|ConvertFrom-Json
 $count=0
 foreach($entry in $inputs){
  $source=Join-Path (Join-Path 'C:\BoomPlayM8' $control.language) $entry.path.Replace('/','\')
  $file=Get-Item -LiteralPath $source
  if($file.Length -ne $entry.bytes -or (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.sha256){throw 'New-language input identity mismatch'}
  $target=Join-Path $game $entry.path.Replace('/','\');New-Item -ItemType Directory -Path (Split-Path $target) -Force|Out-Null
  if($file.Extension -eq '.idd'){New-Item -ItemType HardLink -Path $target -Target $source|Out-Null}else{Copy-Item -LiteralPath $source -Destination $target;(Get-Item -LiteralPath $target).IsReadOnly=$false}
  $count++
 }
 $pins=Get-Content -LiteralPath (Join-Path $seed 'control-pins.json') -Raw|ConvertFrom-Json
 foreach($name in @('kr-launch.exe','kr-companion.exe','CA-control.exe','certificate.der')){
  $source=Join-Path $seed $name
  if((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -ne $pins.$name){throw 'Runtime control identity mismatch'}
  $target=Join-Path $game $name;if($name -eq 'CA-control.exe'){$target=Join-Path $game 'CA.exe'}
  Copy-Item -LiteralPath $source -Destination $target;(Get-Item -LiteralPath $target).IsReadOnly=$false
 }
 $deploy=Get-Content -LiteralPath (Join-Path $seed 'deployment-private.json') -Raw|ConvertFrom-Json
 $hosts=Join-Path $env:SystemRoot 'System32\drivers\etc\hosts'
 $nativeHost=$deploy.server
 if($control.ingame_local_capture){
  if(-not $control.ingame_login_probe){throw 'Local capture requires no-token mode'}
  $nativeHost='127.0.0.1'
  foreach($name in @('login-bootstrap.bin','login-wcp.bin')){
   $expected=$control.($name+'_sha256')
   if((Get-FileHash -LiteralPath (Join-Path $seed $name) -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expected){throw 'Local login fixture mismatch'}
  }
  $source=Join-Path $seed 'native-login-capture.cs'
  if((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -ne $control.capture_source_sha256){throw 'Local login capture source mismatch'}
  Add-Type -Path $source
  $localCapture=New-Object M8LoginCapture((Join-Path $seed 'native-login-capture.jsonl'),[IO.File]::ReadAllBytes((Join-Path $seed 'login-bootstrap.bin')),[IO.File]::ReadAllBytes((Join-Path $seed 'login-wcp.bin')))
  $report.local_capture='Loopback only; pinned bootstrap/WCP, no auth replies, bounded raw synthetic input. No production auth change.'
 }
 if($control.ingame_local_capture){
  $ownedNames=@('lslist.crazyarcade.com','dream-login-ca.nexon.com','happy-login-ca.nexon.com')
  $kept=New-Object Collections.Generic.List[string]
  foreach($line in [IO.File]::ReadAllLines($hosts)){
   $parts=($line -split '#',2)[0].Trim() -split '\s+'
   if($parts.Count -gt 1){
    $aliases=@($parts|Select-Object -Skip 1|Where-Object {$_ -notin $ownedNames})
    if($aliases.Count -ne ($parts.Count-1)){
     if($aliases.Count -gt 0){$kept.Add($parts[0]+' '+($aliases -join ' '))}
     continue
    }
   }
   $kept.Add($line)
  }
  [IO.File]::WriteAllLines($hosts,$kept)
 }
 [IO.File]::AppendAllText($hosts,"`r`n"+$nativeHost+" lslist.crazyarcade.com dream-login-ca.nexon.com happy-login-ca.nexon.com`r`n")
 if($control.ingame_local_capture){
  $null=& ipconfig.exe /flushdns
  $report.local_host_resolution=@($ownedNames|ForEach-Object {
   $name=$_;$values=@([Net.Dns]::GetHostAddresses($name)|ForEach-Object {$_.IPAddressToString})
   if($values.Count -eq 0 -or @($values|Where-Object {$_ -ne '127.0.0.1'}).Count -ne 0){throw 'Feasibility hostname is not loopback only'}
   [ordered]@{name=$name;addresses=$values}
  })
 }
 $address=Get-NetIPAddress -AddressFamily IPv4|Where-Object {$_.IPAddress -ne '127.0.0.1' -and $_.IPAddress -notlike '169.254.*'}|Select-Object -First 1
 $cancel=Join-Path $game 'harness-cancel.request'
 $config=[ordered]@{server=($deploy.server+':'+$deploy.tls_port);certificate=(Join-Path $game 'certificate.der');reported_login_ipv4=$address.IPAddress;native_ip='127.0.0.1';companion=(Join-Path $game 'kr-companion.exe');native_program=(Join-Path $game 'CA.exe');native_arguments=@('multiserveridx:0');native_directory=$game;port_budget=100;peer_diagnostics=$true;udp_backend='portable';cancel_file=$cancel}
 if($control.PSObject.Properties.Name -contains 'socket_probe_sha256'){
  $probe=Join-Path $seed 'socket-probe.exe'
  if($control.socket_probe_sha256 -notmatch '^[a-f0-9]{64}$' -or (Get-FileHash -LiteralPath $probe -Algorithm SHA256).Hash.ToLowerInvariant() -ne $control.socket_probe_sha256){throw 'Socket probe identity mismatch'}
  $config.native_arguments=@((Join-Path $seed 'native-socket-probe.jsonl'),$config.native_program)+@($config.native_arguments)
  $config.native_program=$probe
  $report.socket_probe=[ordered]@{sha256=$control.socket_probe_sha256;limits='Owned debug child; software breakpoints can perturb startup and concurrency; not native acceptance.'}
 }
 $path=Join-Path $game 'launcher.json';[IO.File]::WriteAllText($path,($config|ConvertTo-Json -Depth 8))
 if($control.crash_dump){
  if(Test-Path -LiteralPath $werKey){throw 'WER destination already exists'}
  $dump=Join-Path $seed 'dumps';if(Test-Path -LiteralPath $dump){throw 'Crash output already exists'}
  New-Item -ItemType Directory -Path $dump|Out-Null
  New-Item -Path $werKey -Force|Out-Null;$ownedWer=$true
  New-ItemProperty -Path $werKey -Name DumpFolder -PropertyType ExpandString -Value $dump|Out-Null
  New-ItemProperty -Path $werKey -Name DumpType -PropertyType DWord -Value 1|Out-Null
  New-ItemProperty -Path $werKey -Name DumpCount -PropertyType DWord -Value 1|Out-Null
  $report.crash_capture='Owned WER minidump; no debugger or runtime exception handler added'
 }
 $report.install=[ordered]@{files_checked=($count+1);exe_sha256=$pins.'CA-control.exe';launcher_sha256=$pins.'kr-launch.exe';companion_sha256=$pins.'kr-companion.exe'}
 $report.phase='install_ready';Save-Report;Write-Output ('M8_INSTALL_READY files='+($count+1))
 $online=$false;$deadline=[DateTime]::UtcNow.AddSeconds(90)
 while([DateTime]::UtcNow -lt $deadline){$s=New-Object Net.Sockets.TcpClient;try{$a=$s.BeginConnect($deploy.server,[int]$deploy.tls_port,$null,$null);if($a.AsyncWaitHandle.WaitOne(400)){$s.EndConnect($a);$online=$true}}catch{$online=$false}finally{$s.Close()};if($online){break};Start-Sleep -Milliseconds 500}
 if(-not $online){throw 'Service readiness deadline'}
 # Bare TCP readiness probes are not TLS requests. Let their bounded server
 # handlers drain before the single declared login (server timeout <=2s).
 # This is a harness timing control, not an auth retry or relaxed deadline.
 $report.prelogin_probe_drain_ms=2200;Save-Report;Start-Sleep -Milliseconds 2200
 $start=New-Object Diagnostics.ProcessStartInfo;$start.FileName=Join-Path $game 'kr-launch.exe';$start.Arguments=$path;$start.WorkingDirectory=$game;$start.UseShellExecute=$false;$start.RedirectStandardInput=$true;$start.RedirectStandardOutput=$true;$start.RedirectStandardError=$true
 if($control.ingame_login_probe){
  $start.FileName=Join-Path $game 'CA.exe';$start.Arguments='multiserveridx:0'
  $report.execution_mode='direct_native_no_token_no_launcher_credentials'
  $report.client_arguments=@('multiserveridx:0')
  if($control.ingame_empty_arguments){$start.Arguments='';$report.client_arguments=@()}
  if($control.ingame_marker_argument){
   if($control.ingame_empty_arguments){throw 'Conflicting no-token argument controls'}
   $start.Arguments='m8probe:1';$report.client_arguments=@('m8probe:1')
  }
  $report.feasibility_limit='Observe existing LoginStage only. No launcher, companion session or credential submission; not account/gameplay acceptance.'
  if($control.PSObject.Properties.Name -contains 'ingame_input_probe_sha256'){
   if(-not $control.ingame_local_capture){throw 'Input observer needs no-token loopback control'}
   $probe=Join-Path $seed 'socket-probe.exe'
   if((Get-FileHash -LiteralPath $probe -Algorithm SHA256).Hash.ToLowerInvariant() -ne $control.ingame_input_probe_sha256){throw 'Input observer identity mismatch'}
   $nativeArguments=$start.Arguments
   $start.FileName=$probe
   $start.Arguments='"'+(Join-Path $seed 'native-socket-probe.jsonl')+'" "'+(Join-Path $game 'CA.exe')+'"'
   if($nativeArguments){$start.Arguments+=' '+$nativeArguments}
   $start.EnvironmentVariables['KR_M8_LOGIN_INPUT_PROBE']='1'
   $report.execution_mode='no_token_read_only_input_observer'
   $report.feasibility_limit='Declared debugger control of existing handlers. Software breakpoints perturb timing; no secret reads, forced results or authentication acceptance.'
  }
 }
 if($control.PSObject.Properties.Name -contains 'wait_timeout_control_ms'){
  if(-not ($control.PSObject.Properties.Name -contains 'socket_probe_sha256') -or $control.wait_timeout_control_ms -lt 1 -or $control.wait_timeout_control_ms -gt 60000){throw 'Wait timing control bounds'}
  $start.EnvironmentVariables['KR_M8_WAIT_TIMEOUT_MS']=[string]$control.wait_timeout_control_ms
  $report.wait_timeout_control=[ordered]@{milliseconds=$control.wait_timeout_control_ms;limits='Debugger-only finite control applied only if measured dialog limit is zero. No source file, server deadline or success result is changed.'}
 }
 if($control.ui_exit_probe){
  if(-not ($control.PSObject.Properties.Name -contains 'socket_probe_sha256')){throw 'UI exit observer requires debugger control'}
  $start.EnvironmentVariables['KR_M8_UI_EXIT_PROBE']='1'
  $report.ui_exit_probe='Read-only numeric exit-button/modal/predicate observation; no forced result or native acceptance'
 }
 $p=New-Object Diagnostics.Process;$p.StartInfo=$start;if(-not $p.Start()){throw 'Native launcher did not start'}
 # Drain both pipes while the process runs. A native/debug child can inherit
 # their handles, so EOF and process exit are separate bounded observations.
 $stdoutTask=$p.StandardOutput.ReadToEndAsync();$stderrTask=$p.StandardError.ReadToEndAsync()
 if(-not $control.ingame_login_probe){$p.StandardInput.Write([IO.File]::ReadAllText((Join-Path $seed 'credentials.json')))}
 $p.StandardInput.Close()
 $report.launcher_pid=$p.Id;$report.phase='launcher_running';$report.native_attempt_utc=[DateTime]::UtcNow.ToString('o');Save-Report
 $deadline=[DateTime]::UtcNow.AddSeconds([int]$control.observe_seconds)
 while(-not $p.HasExited -and [DateTime]::UtcNow -lt $deadline){
  if($control.ingame_login_probe){
   $report.native_windows=@(Get-Process -Name CA,CALauncher -ErrorAction SilentlyContinue|Where-Object {$_.Path -and $_.Path.StartsWith($game+'\')}|Select-Object Id,ProcessName,MainWindowTitle,MainWindowHandle,Responding)
   Save-Report
  }
  [void]$p.WaitForExit(1000)
 }
 if(-not $p.HasExited){
  $report.observation_deadline=$true;$report.termination='harness_cancellation';Save-Report
  # Use the launcher's owned-child cancellation path, including debug children.
  # This is a harness cancellation, never a native UI logout acceptance.
  if($control.ingame_login_probe){
   $report.termination='direct_native_harness_cancellation'
   $null=& taskkill.exe /PID $p.Id /T /F 2>&1
  }else{[IO.File]::WriteAllText($cancel,'observation_deadline')}
  $report.cancel_requested_utc=[DateTime]::UtcNow.ToString('o');Save-Report
  if(-not $p.WaitForExit(15000)){
   $report.forced_launcher_stop=$true;Save-Report
   $p.Kill();if(-not $p.WaitForExit(5000)){throw 'Launcher stop deadline'}
  }
 }else{$report.termination='launcher_exit'}
 $report.launcher_exit=$p.ExitCode
 foreach($entry in @(@('stdout',$stdoutTask),@('stderr',$stderrTask))){
  if($entry[1].Wait(3000)){$report[$entry[0]]=$entry[1].Result}else{$report[($entry[0]+'_drain_deadline')]=$true}
 }
 if($ownedWer){
  # WER can finish after the process. Keep a finite flush observation, never
  # retry native execution or treat absence of a dump as absence of a crash.
  Start-Sleep -Seconds 10
  $report.dump_files=@(Get-ChildItem -LiteralPath $dump -Filter '*.dmp' -File|Select-Object Name,Length)
  $report.application_events=@(Get-WinEvent -FilterHashtable @{LogName='Application';StartTime=[DateTime]::Parse($report.native_attempt_utc);Id=1000,1001} -MaxEvents 16 -ErrorAction SilentlyContinue|Select-Object TimeCreated,Id,ProviderName,Message)
 }
 $report.phase='finished';$report.ended_utc=[DateTime]::UtcNow.ToString('o');Save-Report
}catch{$report.error=($_|Out-String);$report.phase='failed';Save-Report}
finally{
 if($null -ne $localCapture){$localCapture.Dispose();$report.local_capture_stopped=$true;Save-Report}
 if($ownedWer){
  Remove-Item -LiteralPath $werKey -Recurse -Force
  $report.owned_wer_removed=-not (Test-Path -LiteralPath $werKey);Save-Report
  [IO.File]::WriteAllText((Join-Path $seed 'native-windows-crash.json'),($report|ConvertTo-Json -Depth 12))
 }
}
