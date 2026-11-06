param([ValidateSet('CA','NewPatcher','Both')][string]$Recipe='CA',[string]$RunId='ca-control-r1',[ValidateRange(1024,65535)][int]$SentinelPort=39177)
$ErrorActionPreference='Stop'
if ($RunId -notmatch '^[a-z0-9-]+$') { throw 'Unsafe run ID' }
$root='\\host.lan\Data\boom\repos\boom-kr-server\local\research\codex\local-acceptance-r1'
$out=Join-Path $root $RunId
$game='C:\BoomKR-CP178\game'
$traceName='KRECP178-'+$RunId
$localOut=Join-Path 'C:\BoomKR-CP178\traces' $RunId
$reportPath=Join-Path $out 'control.json'
if (-not (Test-Path -LiteralPath $out) -or (Test-Path -LiteralPath $reportPath) -or (Test-Path -LiteralPath $localOut)) { throw 'Run paths missing or already used' }
$owned=@{};$traceStarted=$false;$etl=$null;$driversBefore=@()
$record=[ordered]@{schema='kre-original-windows-control-r1';recipe=$Recipe;run_id=$RunId;start_utc=[DateTime]::UtcNow.ToString('o');configured_cwd=$game;roots=@();samples=@();errors=@();cleanup=@();limits='Original source bytes copied to fixed C; observation only/no hooks or state writes. Windows control, not Wine gameplay or K1/login/gate success.'}
Add-Type @'
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;
public static class KreControlOS {
 public delegate bool EnumProc(IntPtr w,IntPtr a);
 [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb,IntPtr arg);
 [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr parent,EnumProc cb,IntPtr arg);
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr w);
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr w,out uint pid);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr w,StringBuilder s,int n);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetClassNameW(IntPtr w,StringBuilder s,int n);
 [DllImport("kernel32.dll",SetLastError=true)] static extern IntPtr OpenProcess(uint access,bool inherit,int pid);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool ReadProcessMemory(IntPtr h,IntPtr addr,byte[] b,int n,out IntPtr read);
 [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
 [DllImport("ntdll.dll")] static extern int NtQueryInformationProcess(IntPtr p,int cl,out IntPtr info,int size,out int len);
 [DllImport("advapi32.dll",SetLastError=true)] static extern bool OpenProcessToken(IntPtr p,uint access,out IntPtr token);
 [DllImport("advapi32.dll",SetLastError=true)] static extern bool GetTokenInformation(IntPtr token,int cl,out int info,int size,out int len);
 public static Dictionary<string,object> Inspect(int id) {
  var result=new Dictionary<string,object>();IntPtr h=OpenProcess(0x1010,false,id);
  if(h==IntPtr.Zero){result["open_error"]=Marshal.GetLastWin32Error();return result;}
  try {
   IntPtr token;int n,v;
   if(OpenProcessToken(h,8,out token)) {try {if(GetTokenInformation(token,20,out v,4,out n))result["elevated"]=v;if(GetTokenInformation(token,18,out v,4,out n))result["elevation_type"]=v;}finally{CloseHandle(token);}}
   else result["token_error"]=Marshal.GetLastWin32Error();
   IntPtr peb;int size;int status=NtQueryInformationProcess(h,26,out peb,IntPtr.Size,out size);result["wow64_query_status"]=status;
   if(status==0 && peb!=IntPtr.Zero) {
    IntPtr read;var b=new byte[4];
    if(!ReadProcessMemory(h,IntPtr.Add(peb,0x10),b,4,out read)||read.ToInt64()!=4)throw new Exception("ReadPEB32params");
    uint parameters=BitConverter.ToUInt32(b,0);b=new byte[8];
    if(!ReadProcessMemory(h,new IntPtr((long)parameters+0x24),b,8,out read)||read.ToInt64()!=8)throw new Exception("ReadCurrentDirectoryDescriptor");
    int len=BitConverter.ToUInt16(b,0);uint ptr=BitConverter.ToUInt32(b,4);
    if(len>65534 || len%2!=0 || len>BitConverter.ToUInt16(b,2))throw new Exception("InvalidCwdDescriptor");
    b=new byte[len];if(!ReadProcessMemory(h,new IntPtr((long)ptr),b,len,out read)||read.ToInt64()!=len)throw new Exception("ReadCurrentDirectory");
    result["native_cwd"]=Encoding.Unicode.GetString(b);result["peb32"]=peb.ToInt64();
   }
   else result["cwd_limit"]="Only Wow64 PEB32 layout inspected; no native64 inference";
  }catch(Exception e){result["inspect_error"]=e.Message;}finally{CloseHandle(h);}return result;
 }
 public static List<Dictionary<string,object>> Windows(int[] pids) {
  var rows=new List<Dictionary<string,object>>();var wanted=new HashSet<int>(pids);
  EnumWindows((w,a)=>{uint pid;GetWindowThreadProcessId(w,out pid);if(wanted.Contains((int)pid)&&IsWindowVisible(w)){var title=new StringBuilder(2048);var cl=new StringBuilder(256);GetWindowTextW(w,title,2048);GetClassNameW(w,cl,256);var children=new List<Dictionary<string,object>>();EnumChildWindows(w,(child,unused)=>{uint owner;GetWindowThreadProcessId(child,out owner);var ct=new StringBuilder(4096);var cc=new StringBuilder(256);GetWindowTextW(child,ct,4096);GetClassNameW(child,cc,256);children.Add(new Dictionary<string,object>{{"pid",owner},{"handle",child.ToInt64()},{"text",ct.ToString()},{"class",cc.ToString()}});return true;},IntPtr.Zero);rows.Add(new Dictionary<string,object>{{"pid",pid},{"handle",w.ToInt64()},{"title",title.ToString()},{"class",cl.ToString()},{"children",children}});}return true;},IntPtr.Zero);return rows;
 }
}
'@
function Remember($proc) {
 $pidValue=[int]$proc.Id
 $live=Get-Process -Id $pidValue -ErrorAction SilentlyContinue
 if (-not $live) { return }
 $cim=Get-CimInstance Win32_Process -Filter ('ProcessId='+$pidValue)
 if (-not $cim.ExecutablePath.StartsWith($game+'\',[StringComparison]::OrdinalIgnoreCase)) { throw 'Process escaped owned game path; do not claim ownership' }
 if (-not $owned.ContainsKey($pidValue)) {
  $owned[$pidValue]=[ordered]@{pid=$pidValue;parent_pid=$cim.ParentProcessId;start_ticks=$live.StartTime.ToUniversalTime().Ticks;path=$cim.ExecutablePath;command=$cim.CommandLine;sha256=(Get-FileHash -LiteralPath $cim.ExecutablePath -Algorithm SHA256).Hash.ToLowerInvariant()}
 }
}
try {
 $busy=@(Get-CimInstance Win32_Process | Where-Object {$_.Name -in @('CA.exe','NewPatcher.exe','CALauncher.exe','nmcosrv.exe')})
 if ($busy.Count) { throw 'Unowned game process already exists; do not disturb it' }
 $identity=[Security.Principal.WindowsIdentity]::GetCurrent();$principal=New-Object Security.Principal.WindowsPrincipal($identity)
 $record.parent_identity=[ordered]@{name=$identity.Name;elevated=$principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)}
 if (-not $record.parent_identity.elevated) { throw 'Controller must already be elevated' }
 $pins=@{'CA.exe'='a52545c14b1fea423de655c3163d6360b628d61c1cd472b46fdbc8a9ffe15f4e';'NewPatcher.exe'='b59e4dce9ad23c439a7632ce04ecb2eed3481585bd2454100b05c9d7527f5c02'}
 foreach ($leaf in $pins.Keys) {if((Get-FileHash -LiteralPath (Join-Path $game $leaf) -Algorithm SHA256).Hash.ToLowerInvariant() -ne $pins[$leaf]){throw 'Source image hash mismatch'}}
 $driversBefore=@(Get-CimInstance Win32_SystemDriver | Select-Object Name,PathName,State,StartMode)
 $driversBefore | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $out 'drivers-before.json') -Encoding UTF8

 # Preparation only: every previous owned process must be absent before restore.
 $manifest=Get-Content -LiteralPath (Join-Path $root 'source-tree.json') -Raw | ConvertFrom-Json
 $expected=@{};foreach($entry in $manifest.files){$expected[$entry.relative.Replace('/','\')]=$entry}
 $record.before_restore_differences=@()
 foreach($file in Get-ChildItem -LiteralPath $game -File -Recurse -Force){
  $relative=$file.FullName.Substring($game.Length+1);$hash=(Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
  if(-not $expected.ContainsKey($relative) -or $hash -ne $expected[$relative].sha256){
   $save=Join-Path (Join-Path $out 'before-restore') $relative;New-Item -Path (Split-Path -Parent $save) -ItemType Directory -Force | Out-Null;Copy-Item -LiteralPath $file.FullName -Destination $save
   $record.before_restore_differences += [ordered]@{relative=$relative;bytes=$file.Length;sha256=$hash;saved=$save}
  }
 }
 $restoreLog=Join-Path $out 'cold-restore.txt'
 & robocopy.exe 'C:\BoomKR-CP178\base' $game /MIR /COPY:DAT /DCOPY:DA /R:0 /W:0 /NP /NFL /NDL "/LOG:$restoreLog" | Out-Null
 $record.restore_returncode=$LASTEXITCODE
 if($LASTEXITCODE -ge 8){throw 'Owned cold-copy restore failed'}
 $checked=0
 foreach($entry in $manifest.files){
  $file=Get-Item -LiteralPath (Join-Path $game $entry.relative.Replace('/','\'))
  if($file.Length -ne $entry.bytes -or (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.sha256){throw ('Cold-copy hash mismatch:'+ $entry.relative)}
  $checked++
 }
 if(@(Get-ChildItem -LiteralPath $game -File -Recurse -Force).Count -ne $checked){throw 'Cold-copy file count mismatch'}
 $record.cold_copy_verified=[ordered]@{files=$checked;source_manifest_sha256=(Get-FileHash -LiteralPath (Join-Path $root 'source-tree.json') -Algorithm SHA256).Hash.ToLowerInvariant();completed_utc=[DateTime]::UtcNow.ToString('o');phase='Before ETW/negativecontrol/client launch'}
 New-Item -Path $localOut -ItemType Directory -Force | Out-Null
 foreach ($leaf in @('CALog.log','Patcher.log')) {
  $file=Join-Path $game $leaf
  if(Test-Path -LiteralPath $file){Copy-Item -LiteralPath $file -Destination (Join-Path $out ('before-'+$leaf));Remove-Item -LiteralPath $file}
 }
 $etl=Join-Path $localOut 'process.etl'
 & logman.exe create trace $traceName -o $etl -p Microsoft-Windows-Kernel-Process 0xFFFFFFFFFFFFFFFF 5 -f bincirc -max 32 -ets > (Join-Path $out 'trace-start.txt') 2>&1
 $record.trace_start_returncode=$LASTEXITCODE
 if($LASTEXITCODE -eq 0){$traceStarted=$true}else{throw 'ETW session could not start; no client launched'}
 $test=New-Object Net.Sockets.TcpClient
 try {
  $started=[DateTime]::UtcNow.ToString('o');$pending=$test.BeginConnect('172.18.0.1',$SentinelPort,$null,$null)
  $done=$pending.AsyncWaitHandle.WaitOne(2000)
  $record.network_negative=[ordered]@{owner_pid=$PID;target_ip='172.18.0.1';target_port=$SentinelPort;start_utc=$started;end_utc=[DateTime]::UtcNow.ToString('o');completed=$done;connected=$test.Connected;scope='Owned passive host sentinel; no public endpoint'}
  if($done -or $test.Connected){throw 'Expected egress DROP timeout absent; do not launch client'}
 }finally{$test.Close()}
 $images=if($Recipe -eq 'Both'){@('NewPatcher.exe','CA.exe')}elseif($Recipe -eq 'CA'){@('CA.exe')}else{@('NewPatcher.exe')}
 foreach ($leaf in $images) {
  $start=[DateTime]::UtcNow.ToString('o');$proc=Start-Process -FilePath (Join-Path $game $leaf) -WorkingDirectory $game -PassThru
  $record.roots += [ordered]@{pid=$proc.Id;leaf=$leaf;requested_utc=$start};Remember $proc
 }
 $record.owned=@($owned.Values)
 $record | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $out 'progress.json') -Encoding UTF8
 for($tick=0;$tick -lt 10;$tick++) {
  $snap=[ordered]@{utc=[DateTime]::UtcNow.ToString('o');processes=@();tcp=@();windows=@()}
  $children=@(Get-CimInstance Win32_Process | Where-Object {$_.ExecutablePath -and $_.ExecutablePath.StartsWith($game+'\',[StringComparison]::OrdinalIgnoreCase)})
  foreach($child in $children){$p=Get-Process -Id $child.ProcessId -ErrorAction SilentlyContinue;if($p){Remember $p}}
  foreach($pidValue in @($owned.Keys)) {
   $p=Get-Process -Id $pidValue -ErrorAction SilentlyContinue
   if(-not $p){continue}
   $info=[ordered]@{pid=$pidValue;name=$p.ProcessName;start_ticks=$p.StartTime.ToUniversalTime().Ticks;native=[KreControlOS]::Inspect($pidValue);modules=@();module_error=$null}
   try{$info.modules=@($p.Modules | Select-Object ModuleName,FileName,@{n='base';e={$_.BaseAddress.ToInt64()}},ModuleMemorySize)}catch{$info.module_error=$_.Exception.Message}
   $snap.processes += $info
  }
  $pids=[int[]]@($owned.Keys)
  $snap.windows=@([KreControlOS]::Windows($pids))
  $snap.tcp=@(Get-NetTCPConnection -ErrorAction SilentlyContinue | Where-Object {$_.OwningProcess -in $pids} | Select-Object OwningProcess,State,LocalAddress,LocalPort,RemoteAddress,RemotePort)
  $record.samples += $snap
  if($snap.tcp | Where-Object {$_.RemoteAddress -notin @('0.0.0.0','::','127.0.0.1','::1','172.30.0.1','20.20.20.1')}){$record.stop_reason='nonlocal_attempt_blocked_by_container_policy';break}
  Start-Sleep -Seconds 2
 }
 if(-not $record.Contains('stop_reason')){$record.stop_reason='bounded20s_control_observation'}
}catch{$record.errors += $_.Exception.Message;$record.stop_reason='controller_exception'}
finally{
 foreach($pidValue in @($owned.Keys)) {
  try {
   $p=Get-Process -Id $pidValue -ErrorAction SilentlyContinue
   if($p){
    $old=$owned[$pidValue];$cim=Get-CimInstance Win32_Process -Filter ('ProcessId='+$pidValue)
    if($p.StartTime.ToUniversalTime().Ticks -ne $old.start_ticks -or $cim.ExecutablePath -ne $old.path -or (Get-FileHash -LiteralPath $old.path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $old.sha256){throw 'Cleanup ownership guard mismatch'}
    Stop-Process -Id $pidValue -Force -ErrorAction Stop
    $record.cleanup += [ordered]@{pid=$pidValue;action='stop_owned_only';start_ticks=$old.start_ticks;path=$old.path}
   }
  }catch{$record.errors += ('cleanup:'+ $_.Exception.Message)}
 }
 if($traceStarted){& logman.exe stop $traceName -ets > (Join-Path $out 'trace-stop.txt') 2>&1;$record.trace_stop_returncode=$LASTEXITCODE}
 Start-Sleep -Milliseconds 500
 $record.owned=@($owned.Values)
 $record.remaining_owned=@(Get-CimInstance Win32_Process | Where-Object {$_.ExecutablePath -and $_.ExecutablePath.StartsWith($game+'\',[StringComparison]::OrdinalIgnoreCase)} | Select-Object ProcessId,ExecutablePath,CreationDate)
 foreach($leaf in @('CALog.log','Patcher.log')){$file=Join-Path $game $leaf;if(Test-Path -LiteralPath $file){Copy-Item -LiteralPath $file -Destination (Join-Path $out ('after-'+$leaf))}}
 if($etl -and (Test-Path -LiteralPath $etl)){Copy-Item -LiteralPath $etl -Destination (Join-Path $out 'process.etl')}
 if($driversBefore.Count){$record.new_drivers=@(Get-CimInstance Win32_SystemDriver | Where-Object {$_.Name -notin @($driversBefore.Name)} | Select-Object Name,PathName,State,StartMode)}else{$record.new_drivers=@()}
 $record.remaining_external_children=@(Get-CimInstance Win32_Process | Where-Object {$_.ParentProcessId -in @($owned.Keys) -and $_.ExecutablePath -and -not $_.ExecutablePath.StartsWith($game+'\',[StringComparison]::OrdinalIgnoreCase)} | Select-Object ProcessId,ParentProcessId,ExecutablePath,CreationDate)
 $record.end_utc=[DateTime]::UtcNow.ToString('o')
 $record | ConvertTo-Json -Depth 14 | Set-Content -LiteralPath $reportPath -Encoding UTF8
}
