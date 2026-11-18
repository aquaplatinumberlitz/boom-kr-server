# Parameterized M8 control for an already verified vi/en/ko installation.
# Inputs, credentials and endpoints are private seed files; source installation is retained.
$ErrorActionPreference='Stop'
$output=Get-Volume|Where-Object FileSystemLabel -eq 'M8SETUP'|Select-Object -First 1
$seed=$output.DriveLetter+':\';$reportPath=Join-Path $seed 'native-windows-install.json'
$control=Get-Content -LiteralPath (Join-Path $seed 'multilang-run.json') -Raw|ConvertFrom-Json
if($control.language -notin @('vi','en','ko') -or $control.runtime_name -notmatch '^M8Native[A-Za-z0-9_-]{1,40}$' -or $control.observe_seconds -lt 30 -or $control.observe_seconds -gt 240){throw 'Multilingual control bounds'}
$report=[ordered]@{schema='kre-m8-multilang-windows-native-v2';start_utc=[DateTime]::UtcNow.ToString('o');phase='starting';language=$control.language;limits='Existing verified language inputs with declared loader-control exe; original L1/installation unchanged. Runtime/wire/UI evidence required.'}
function Save-Report {[IO.File]::WriteAllText($reportPath,($report|ConvertTo-Json -Depth 12))}
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
 [IO.File]::AppendAllText($hosts,"`r`n"+$deploy.server+" lslist.crazyarcade.com dream-login-ca.nexon.com happy-login-ca.nexon.com`r`n")
 $address=Get-NetIPAddress -AddressFamily IPv4|Where-Object {$_.IPAddress -ne '127.0.0.1' -and $_.IPAddress -notlike '169.254.*'}|Select-Object -First 1
 $config=[ordered]@{server=($deploy.server+':'+$deploy.tls_port);certificate=(Join-Path $game 'certificate.der');reported_login_ipv4=$address.IPAddress;native_ip='127.0.0.1';companion=(Join-Path $game 'kr-companion.exe');native_program=(Join-Path $game 'CA.exe');native_arguments=@('multiserveridx:0');native_directory=$game;port_budget=100;peer_diagnostics=$true;udp_backend='portable'}
 if($control.PSObject.Properties.Name -contains 'socket_probe_sha256'){
  $probe=Join-Path $seed 'socket-probe.exe'
  if($control.socket_probe_sha256 -notmatch '^[a-f0-9]{64}$' -or (Get-FileHash -LiteralPath $probe -Algorithm SHA256).Hash.ToLowerInvariant() -ne $control.socket_probe_sha256){throw 'Socket probe identity mismatch'}
  $config.native_arguments=@((Join-Path $seed 'native-socket-probe.jsonl'),$config.native_program)+@($config.native_arguments)
  $config.native_program=$probe
  $report.socket_probe=[ordered]@{sha256=$control.socket_probe_sha256;limits='Owned debug child; software breakpoints can perturb startup and concurrency; not native acceptance.'}
 }
 $path=Join-Path $game 'launcher.json';[IO.File]::WriteAllText($path,($config|ConvertTo-Json -Depth 8))
 $report.install=[ordered]@{files_checked=($count+1);exe_sha256=$pins.'CA-control.exe';launcher_sha256=$pins.'kr-launch.exe';companion_sha256=$pins.'kr-companion.exe'}
 $report.phase='install_ready';Save-Report;Write-Output ('M8_INSTALL_READY files='+($count+1))
 $online=$false;$deadline=[DateTime]::UtcNow.AddSeconds(90)
 while([DateTime]::UtcNow -lt $deadline){$s=New-Object Net.Sockets.TcpClient;try{$a=$s.BeginConnect($deploy.server,[int]$deploy.tls_port,$null,$null);if($a.AsyncWaitHandle.WaitOne(400)){$s.EndConnect($a);$online=$true}}catch{$online=$false}finally{$s.Close()};if($online){break};Start-Sleep -Milliseconds 500}
 if(-not $online){throw 'Service readiness deadline'}
 $start=New-Object Diagnostics.ProcessStartInfo;$start.FileName=Join-Path $game 'kr-launch.exe';$start.Arguments=$path;$start.WorkingDirectory=$game;$start.UseShellExecute=$false;$start.RedirectStandardInput=$true;$start.RedirectStandardOutput=$true;$start.RedirectStandardError=$true
 $p=New-Object Diagnostics.Process;$p.StartInfo=$start;if(-not $p.Start()){throw 'Native launcher did not start'}
 $p.StandardInput.Write([IO.File]::ReadAllText((Join-Path $seed 'credentials.json')));$p.StandardInput.Close()
 $report.launcher_pid=$p.Id;$report.phase='launcher_running';$report.native_attempt_utc=[DateTime]::UtcNow.ToString('o');Save-Report
 if(-not $p.WaitForExit(([int]$control.observe_seconds*1000))){$report.observation_deadline=$true;Get-CimInstance Win32_Process|Where-Object {$_.ExecutablePath -eq (Join-Path $game 'CA.exe') -and $_.ParentProcessId -eq $p.Id}|ForEach-Object {$native=Get-Process -Id $_.ProcessId -ErrorAction SilentlyContinue;if($native){$native.CloseMainWindow()|Out-Null}};if(-not $p.WaitForExit(15000)){$report.forced_launcher_stop=$true;$p.Kill();$p.WaitForExit()}}
 $report.launcher_exit=$p.ExitCode;$report.stdout=$p.StandardOutput.ReadToEnd();$report.stderr=$p.StandardError.ReadToEnd();$report.phase='finished';$report.ended_utc=[DateTime]::UtcNow.ToString('o');Save-Report
}catch{$report.error=($_|Out-String);$report.phase='failed';Save-Report}
