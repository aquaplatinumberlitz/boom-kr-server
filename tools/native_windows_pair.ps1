# Bounded vi/ko coexistence control. Inputs and all observations stay on owned seed.
$ErrorActionPreference='Stop'
$volume=Get-Volume|Where-Object FileSystemLabel -eq 'M8SETUP'|Select-Object -First 1
$seed=$volume.DriveLetter+':\'
$control=Get-Content -LiteralPath (Join-Path $seed 'multilang-run.json') -Raw|ConvertFrom-Json
$pins=Get-Content -LiteralPath (Join-Path $seed 'control-pins.json') -Raw|ConvertFrom-Json
$deploy=Get-Content -LiteralPath (Join-Path $seed 'deployment-private.json') -Raw|ConvertFrom-Json
$reportPath=Join-Path $seed 'native-windows-install.json'
$report=[ordered]@{schema='kre-m8-windows-pair-v1';phase='starting';start_utc=[DateTime]::UtcNow.ToString('o');limits='Two declared language copies and distinct authenticated accounts. Process presence is not room/chat/gameplay acceptance.'}
$clients=New-Object Collections.ArrayList
$history=New-Object Collections.ArrayList
$lastFingerprint=''
$werKey='HKLM:\SOFTWARE\Microsoft\Windows\Windows Error Reporting\LocalDumps\CA.exe'
$ownedWer=$false
function Save-Report {[IO.File]::WriteAllText($reportPath,($report|ConvertTo-Json -Depth 14))}
function Prepare-Client($language,$runtime,$exe,$exeHash,$inputFile,$credentialFile){
 if($language -notin @('vi','ko') -or $runtime -notmatch '^M8Native[A-Za-z0-9_-]{1,40}$'){throw 'Pair language/runtime bounds'}
 $game=Join-Path 'C:\' $runtime;if(Test-Path -LiteralPath $game){throw 'Owned pair runtime already exists'}
 New-Item -ItemType Directory -Path $game|Out-Null
 $inputs=Get-Content -LiteralPath (Join-Path $seed $inputFile) -Raw|ConvertFrom-Json
 $count=0
 foreach($entry in $inputs){
  $source=Join-Path (Join-Path 'C:\BoomPlayM8' $language) $entry.path.Replace('/','\')
  $file=Get-Item -LiteralPath $source
  if($file.Length -ne $entry.bytes -or (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.sha256){throw 'Paired language input identity mismatch'}
  $target=Join-Path $game $entry.path.Replace('/','\');New-Item -ItemType Directory -Path (Split-Path $target) -Force|Out-Null
  if($file.Extension -eq '.idd'){New-Item -ItemType HardLink -Path $target -Target $source|Out-Null}else{Copy-Item -LiteralPath $source -Destination $target;(Get-Item -LiteralPath $target).IsReadOnly=$false}
  $count++
 }
 foreach($name in @('kr-launch.exe','kr-companion.exe','certificate.der')){
  $source=Join-Path $seed $name
  if((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -ne $pins.$name){throw 'Pair launcher/companion identity mismatch'}
  Copy-Item -LiteralPath $source -Destination (Join-Path $game $name)
 }
 $source=Join-Path $seed $exe
 if((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -ne $exeHash){throw 'Paired native copy identity mismatch'}
 Copy-Item -LiteralPath $source -Destination (Join-Path $game 'CA.exe')
 $address=Get-NetIPAddress -AddressFamily IPv4|Where-Object {$_.IPAddress -ne '127.0.0.1' -and $_.IPAddress -notlike '169.254.*'}|Select-Object -First 1
 $cancel=Join-Path $game 'harness-cancel.request'
 $config=[ordered]@{server=($deploy.server+':'+$deploy.tls_port);certificate=(Join-Path $game 'certificate.der');reported_login_ipv4=$address.IPAddress;native_ip='127.0.0.1';companion=(Join-Path $game 'kr-companion.exe');native_program=(Join-Path $game 'CA.exe');native_arguments=@('multiserveridx:0');native_directory=$game;port_budget=100;peer_diagnostics=$true;udp_backend='portable';cancel_file=$cancel}
 $path=Join-Path $game 'launcher.json';[IO.File]::WriteAllText($path,($config|ConvertTo-Json -Depth 8))
 return [pscustomobject]@{language=$language;game=$game;config=$path;cancel=$cancel;credentials=(Join-Path $seed $credentialFile);files=($count+1);exe_sha256=$exeHash;process=$null;stdout=$null;stderr=$null;ended=$false;started_utc=$null}
}
function Start-Client($client){
 $start=New-Object Diagnostics.ProcessStartInfo;$start.FileName=Join-Path $client.game 'kr-launch.exe';$start.Arguments=$client.config;$start.WorkingDirectory=$client.game;$start.UseShellExecute=$false;$start.RedirectStandardInput=$true;$start.RedirectStandardOutput=$true;$start.RedirectStandardError=$true
 $p=New-Object Diagnostics.Process;$p.StartInfo=$start;if(-not $p.Start()){throw 'Paired launcher did not start'}
 $client.process=$p;$client.stdout=$p.StandardOutput.ReadToEndAsync();$client.stderr=$p.StandardError.ReadToEndAsync()
 $client.started_utc=[DateTime]::UtcNow.ToString('o')
 $p.StandardInput.Write([IO.File]::ReadAllText($client.credentials));$p.StandardInput.Close()
}
function Observe-Client($client){
 if($null -eq $client.process){return [ordered]@{language=$client.language;started=$false}}
 $p=$client.process;$ended=$p.HasExited
 $value=[ordered]@{language=$client.language;started=$true;launcher_pid=$p.Id;started_utc=$client.started_utc;ended=$ended}
 if($ended){
  $value.launcher_exit=$p.ExitCode
  if($client.stdout.IsCompleted){$value.stdout=$client.stdout.Result}
  if($client.stderr.IsCompleted){$value.stderr=$client.stderr.Result}
 }
 $windows=@(Get-Process -Name CA -ErrorAction SilentlyContinue|Where-Object {$_.Path -eq (Join-Path $client.game 'CA.exe')}|Select-Object Id,MainWindowTitle,MainWindowHandle,Responding)
 $value.native_windows=$windows
 return $value
}
try{
 if($control.language -ne 'vi' -or $control.paired.language -ne 'ko' -or $control.observe_seconds -lt 30 -or $control.observe_seconds -gt 240){throw 'Pair declaration bounds'}
 $primary=Prepare-Client 'vi' $control.runtime_name 'CA-control.exe' $pins.'CA-control.exe' 'language-inputs.json' 'credentials.json'
 $secondary=Prepare-Client 'ko' $control.paired.runtime_name 'CA-paired-control.exe' $control.paired.exe_sha256 'paired-language-inputs.json' 'paired-credentials.json'
 if($control.PSObject.Properties.Name -contains 'paired_startup_probe_sha256'){
  $probe=Join-Path $seed 'socket-probe.exe'
  if($control.paired_startup_probe_sha256 -notmatch '^[a-f0-9]{64}$' -or (Get-FileHash -LiteralPath $probe -Algorithm SHA256).Hash.ToLowerInvariant() -ne $control.paired_startup_probe_sha256){throw 'Paired startup observer identity mismatch'}
  $config=Get-Content -LiteralPath $secondary.config -Raw|ConvertFrom-Json
  $config.native_arguments=@((Join-Path $seed 'native-socket-probe.jsonl'),$config.native_program)+@($config.native_arguments)
  $config.native_program=$probe
  [IO.File]::WriteAllText($secondary.config,($config|ConvertTo-Json -Depth 8))
  $report.startup_guard_observer='Secondary only, software breakpoint observation; results untouched. Debugger control, not uninstrumented native acceptance.'
 }
 [void]$clients.Add($primary);[void]$clients.Add($secondary)
 if($control.crash_dump){
  if(Test-Path -LiteralPath $werKey){throw 'Pair WER destination already exists'}
  $dump=Join-Path $seed 'dumps';if(Test-Path -LiteralPath $dump){throw 'Pair dump output already exists'}
  New-Item -ItemType Directory -Path $dump|Out-Null;New-Item -Path $werKey -Force|Out-Null;$ownedWer=$true
  New-ItemProperty -Path $werKey -Name DumpFolder -PropertyType ExpandString -Value $dump|Out-Null
  New-ItemProperty -Path $werKey -Name DumpType -PropertyType DWord -Value 1|Out-Null
  New-ItemProperty -Path $werKey -Name DumpCount -PropertyType DWord -Value 2|Out-Null
  $report.crash_capture='Owned WER pair minidumps by PID; no debugger or runtime exception handler'
 }
 $hosts=Join-Path $env:SystemRoot 'System32\drivers\etc\hosts'
 [IO.File]::AppendAllText($hosts,"`r`n"+$deploy.server+" lslist.crazyarcade.com dream-login-ca.nexon.com happy-login-ca.nexon.com`r`n")
 $report.install=[ordered]@{files_checked=$primary.files;exe_sha256=$primary.exe_sha256;launcher_sha256=$pins.'kr-launch.exe';companion_sha256=$pins.'kr-companion.exe'}
 $report.paired_install=[ordered]@{language='ko';files_checked=$secondary.files;exe_sha256=$secondary.exe_sha256}
 $report.phase='install_ready';Save-Report;Write-Output ('M8_INSTALL_READY files='+$primary.files)
 $online=$false;$deadline=[DateTime]::UtcNow.AddSeconds(90)
 while([DateTime]::UtcNow -lt $deadline){$s=New-Object Net.Sockets.TcpClient;try{$a=$s.BeginConnect($deploy.server,[int]$deploy.tls_port,$null,$null);if($a.AsyncWaitHandle.WaitOne(400)){$s.EndConnect($a);$online=$true}}catch{$online=$false}finally{$s.Close()};if($online){break};Start-Sleep -Milliseconds 500}
 if(-not $online){throw 'Pair service readiness deadline'}
 Start-Sleep -Milliseconds 2200
 $report.native_attempt_utc=[DateTime]::UtcNow.ToString('o')
 Start-Client $primary;$report.phase='launcher_running';$report.pair_stage='primary_running';$report.launcher_pid=$primary.process.Id;Save-Report
 $deadline=[DateTime]::UtcNow.AddSeconds([int]$control.observe_seconds)
 while([DateTime]::UtcNow -lt $deadline){
  if(Test-Path -LiteralPath (Join-Path $seed 'pair.end')){$report.finish_requested=$true;break}
  if($primary.process.HasExited){$report.primary_ended_before_deadline=$true;break}
  if($null -eq $secondary.process -and (Test-Path -LiteralPath (Join-Path $seed 'pair.go'))){
   $oldGuard=$env:KR_M8_STARTUP_GUARD_PROBE
   try{
    if($control.PSObject.Properties.Name -contains 'paired_startup_probe_sha256'){$env:KR_M8_STARTUP_GUARD_PROBE='1'}
    Start-Client $secondary
   }finally{$env:KR_M8_STARTUP_GUARD_PROBE=$oldGuard}
   $report.pair_stage='both_launchers_started';$report.paired_start_utc=[DateTime]::UtcNow.ToString('o')
  }
  $report.clients=@($clients|ForEach-Object {Observe-Client $_})
  $fingerprint=$report.clients|ConvertTo-Json -Depth 10 -Compress
  if($fingerprint -ne $lastFingerprint){
   if($history.Count -lt 32){[void]$history.Add([ordered]@{observed_utc=[DateTime]::UtcNow.ToString('o');clients=$report.clients})}else{$report.history_truncated=$true}
   $lastFingerprint=$fingerprint
  }
  $report.client_transitions=@($history.ToArray());Save-Report
  Start-Sleep -Milliseconds 500
 }
 $report.observation_deadline=[DateTime]::UtcNow -ge $deadline
 $report.termination='bounded_pair_harness_cancellation'
}catch{$report.error=($_|Out-String);$report.phase='failed';Save-Report}
finally{
 foreach($client in $clients){
  if($null -ne $client.process -and -not $client.process.HasExited){[IO.File]::WriteAllText($client.cancel,'pair_control_end')}
 }
 foreach($client in $clients){
  if($null -ne $client.process){
   if(-not $client.process.WaitForExit(15000)){$report.forced_launcher_stop=$true;$client.process.Kill();[void]$client.process.WaitForExit(5000)}
   [void]$client.stdout.Wait(3000);[void]$client.stderr.Wait(3000)
  }
 }
 $report.clients=@($clients|ForEach-Object {Observe-Client $_})
 if($ownedWer){
  Start-Sleep -Seconds 10
  $report.dump_files=@(Get-ChildItem -LiteralPath $dump -Filter '*.dmp' -File|Select-Object Name,Length)
  $report.application_events=@(Get-WinEvent -FilterHashtable @{LogName='Application';StartTime=[DateTime]::Parse($report.native_attempt_utc);Id=1000,1001} -MaxEvents 16 -ErrorAction SilentlyContinue|Select-Object TimeCreated,Id,ProviderName,Message)
  Remove-Item -LiteralPath $werKey -Recurse -Force
  $report.owned_wer_removed=-not (Test-Path -LiteralPath $werKey)
  [IO.File]::WriteAllText((Join-Path $seed 'native-windows-crash.json'),($report|ConvertTo-Json -Depth 14))
 }
 if($report.phase -ne 'failed'){$report.phase='finished'}
 $report.ended_utc=[DateTime]::UtcNow.ToString('o');Save-Report
}
