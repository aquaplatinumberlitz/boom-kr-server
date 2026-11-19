param([ValidateRange(15,120)][int]$ObserveSeconds=45,
      [ValidatePattern('^[a-z0-9-]{1,32}$')][string]$Case='baseline')
# Dedicated disposable VM only. No service, account or gameplay acceptance.
$ErrorActionPreference='Stop'
$report=[ordered]@{schema='kre-native-windows-startup-diagnostic-v1';started_utc=[DateTime]::UtcNow.ToString('o');limits='Offline startup control with synthetic token and WER capture; not login/gameplay.'}
$output=Get-Volume | Where-Object FileSystemLabel -eq 'M8SETUP' | Select-Object -First 1
if(-not $output){throw 'Owned output media missing'}
$root=$output.DriveLetter+':\'
$inputPath=Join-Path $root ('native-startup-input-'+$Case+'.json')
if(-not (Test-Path -LiteralPath $inputPath)){throw 'Explicit startup input manifest missing'}
$startupInput=Get-Content -LiteralPath $inputPath -Raw | ConvertFrom-Json
$program=$startupInput.program
$directory=$startupInput.native_directory
if($startupInput.sha256 -notmatch '^[a-f0-9]{64}$'){throw 'Input SHA-256 declaration missing'}
$report.case=$Case
$caseRoot=Join-Path $root $Case
if(Test-Path -LiteralPath $caseRoot){throw 'Case output already exists'}
New-Item -ItemType Directory -Path $caseRoot | Out-Null
$key='HKLM:\SOFTWARE\Microsoft\Windows\Windows Error Reporting\LocalDumps\CA.exe'
$ownedKey=$false
$native=$null
try {
 if(Test-Path -LiteralPath $key){throw 'Diagnostic registry destination already exists'}
 $identity=[Security.Principal.WindowsIdentity]::GetCurrent()
 $principal=New-Object Security.Principal.WindowsPrincipal($identity)
 if(-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'Owned WER diagnostic requires elevation'}
 $report.exe_sha256=(Get-FileHash -LiteralPath $program -Algorithm SHA256).Hash.ToLowerInvariant()
 if($report.exe_sha256 -ne $startupInput.sha256){throw 'Unexpected native input identity'}
 $dump=Join-Path $caseRoot 'dumps'
 New-Item -ItemType Directory -Path $dump | Out-Null
 New-Item -Path $key -Force | Out-Null
 $ownedKey=$true
 New-ItemProperty -Path $key -Name DumpFolder -PropertyType ExpandString -Value $dump | Out-Null
 New-ItemProperty -Path $key -Name DumpType -PropertyType DWord -Value 2 | Out-Null
 New-ItemProperty -Path $key -Name DumpCount -PropertyType DWord -Value 1 | Out-Null
 $start=Get-Date
 $native=Start-Process -FilePath $program -WorkingDirectory $directory -ArgumentList 'token_1:00000000000000000000000000000000' -PassThru
 $report.native_pid=$native.Id
 if(-not $native.WaitForExit($ObserveSeconds*1000)){
  $report.observation_deadline=$true
  $native.CloseMainWindow() | Out-Null
  if(-not $native.WaitForExit(5000)){$report.forced_stop=$true;$native.Kill();$native.WaitForExit()}
 }
 $native.Refresh()
 $report.native_exit=$native.ExitCode
 $deadline=(Get-Date).AddSeconds(30)
 do {
  Start-Sleep -Seconds 1
  $files=@(Get-ChildItem -LiteralPath $dump -Filter '*.dmp' -File)
 } while($files.Count -eq 0 -and (Get-Date) -lt $deadline)
 # WER may still be flushing the dump after process exit; do not hash a growing file.
 Start-Sleep -Seconds 10
 $report.dump_files=@(Get-ChildItem -LiteralPath $dump -Filter '*.dmp' -File | Select-Object Name,Length)
 $report.application_events=@(Get-WinEvent -FilterHashtable @{LogName='Application';StartTime=$start;Id=1000,1001} -MaxEvents 16 -ErrorAction SilentlyContinue | Select-Object TimeCreated,Id,ProviderName,Message)
 $report.phase='finished'
} catch {
 $report.phase='failed';$report.error=($_ | Out-String)
} finally {
 if($native -and -not $native.HasExited){$native.Kill();$native.WaitForExit()}
 if($ownedKey){Remove-Item -LiteralPath $key -Recurse -Force;$report.owned_registry_removed=-not (Test-Path -LiteralPath $key)}
 $report.exe_sha256_after=(Get-FileHash -LiteralPath $program -Algorithm SHA256).Hash.ToLowerInvariant()
 $report.ended_utc=[DateTime]::UtcNow.ToString('o')
 [IO.File]::WriteAllText((Join-Path $caseRoot 'native-startup-diagnostic.json'),($report | ConvertTo-Json -Depth 8))
 Write-Output ('M8_OFFLINE_DIAGNOSTIC '+$report.phase)
}
