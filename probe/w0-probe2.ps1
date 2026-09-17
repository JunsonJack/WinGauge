# W0 第二轮：针对第一轮暴露的问题做定向确认
param([string]$Section = "all")
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

function Probe {
    param([string]$Name, [scriptblock]$Body)
    if ($Section -ne "all" -and $Section -notlike "*$Name*") { return }
    Write-Output ""; Write-Output "### $Name"
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    try { & $Body; $r = "OK" }
    catch {
        $msg = $_.Exception.Message -replace '\s+', ' '
        if ($msg.Length -gt 200) { $msg = $msg.Substring(0, 200) + "..." }
        Write-Output "FAIL: $msg"; $r = "FAIL"
    }
    $sw.Stop(); Write-Output "--- ($Name => $r, $($sw.ElapsedMilliseconds) ms)"
}

Write-Output "=== W0 第二轮探测 $(Get-Date -Format 'HH:mm:ss') ==="

# 1. 联想 GamZone WMI：本机是否有可用的温度/风扇方法（第一轮只看到 *_EVENT 类）
Probe "lenovo-gamzone-classes" {
    $cls = Get-CimClass -Namespace root\wmi | Where-Object { $_.CimClassName -match 'LENOVO|GAMEZONE|Lfc_' }
    foreach ($c in $cls) {
        $methods = ($c.CimClassMethods | ForEach-Object { $_.Name }) -join ','
        Write-Output ("{0}  | methods: {1}" -f $c.CimClassName, $(if ($methods) { $methods } else { '-' }))
    }
}

Probe "lenovo-fan-method" {
    $inst = Get-CimInstance -Namespace root\wmi -ClassName LENOVO_FAN_METHOD -ErrorAction SilentlyContinue
    if (-not $inst) { Write-Output "无 LENOVO_FAN_METHOD 实例"; return }
    Write-Output "实例属性: $(($inst | Get-Member -MemberType Properties | Select-Object -ExpandProperty Name | Sort-Object) -join ', ')"
    $r = Invoke-CimMethod -InputObject $inst -MethodName QueryThermalTableMode -ErrorAction SilentlyContinue
    Write-Output "QueryThermalTableMode => $($r | ConvertTo-Json -Compress -Depth 3)"
}

Probe "lenovo-temp-via-event-query" {
    # 温度类是事件类：验证能否用轮询式事件查询拿到最近一次上报
    $q = "SELECT * FROM LENOVO_GAMEZONE_TEMP_EVENT"
    $ev = @(Register-CimIndicationEvent -Query $q -SourceIdentifier w0temp -Namespace root\wmi -ErrorAction Stop)
    Start-Sleep -Seconds 6
    $n = (Get-Event -SourceIdentifier w0temp -ErrorAction SilentlyContinue | Measure-Object).Count
    Unregister-Event -SourceIdentifier w0temp -ErrorAction SilentlyContinue
    Write-Output "6 秒内收到 LENOVO_GAMEZONE_TEMP_EVENT 条数 = $n（0 表示固件不主动上报，需走方法调用）"
}

# 2. 磁盘 SMART 明细（第一轮参数集写法有误）
Probe "disk-reliability-fixed" {
    Get-PhysicalDisk | Get-StorageReliabilityCounter -ErrorAction Stop |
        Select-Object DeviceId, Temperature, PowerOnHours, Wear, ReadErrorsTotal, ReadErrorsUncorrected |
        Format-Table -AutoSize
}

Probe "disk-smart-attr" {
    Get-CimInstance -Namespace root\wmi -ClassName MSFT_PhysicalDisk -ErrorAction Stop |
        Select-Object FriendlyName, MediaType, Size | Format-Table -AutoSize
}

# 3. 网络：验证按 ifIndex 取字节数是否可靠（第一轮证明 WMI 性能类的 Name 会被截断/变形）
Probe "net-per-ifindex" {
    Get-NetAdapterStatistics | Where-Object { $_.OperationalStatus -eq 'Up' } |
        Select-Object Name, ifIndex, ReceivedBytes, SentBytes | Format-Table -AutoSize
    Write-Output "↑  与 Get-NetAdapter / Get-NetRoute 的 ifIndex 一一对应，是无歧义的关联键"
}

Probe "net-wmi-perf-name-mangling" {
    Get-CimInstance Win32_PerfFormattedData_Tcpip_NetworkInterface |
        Select-Object Name | Format-Table -AutoSize
    Write-Output "↑  对比上一项：WMI 类里名称被规范化（[R]、GbE→FE、无 ifIndex），据此匹配会错"
}

# 4. 显示器与工作区（第一轮类型名写错）
Probe "desktop-metrics-fixed" {
    Add-Type -AssemblyName System.Windows.Forms
    [System.Windows.Forms.Screen]::AllScreens | ForEach-Object {
        [PSCustomObject]@{ Device = $_.DeviceName; Primary = $_.Primary
            Bounds = "$($_.Bounds)"; WorkingArea = "$($_.WorkingArea)" }
    } | Format-Table -AutoSize
    $dpi = (Get-ItemProperty 'HKCU:\Control Panel\Desktop\WindowMetrics' -Name AppliedDPI -EA SilentlyContinue).AppliedDPI
    Write-Output "AppliedDPI = $dpi  (缩放 = $([math]::Round($dpi/96*100))%)"
}

# 5. 温度可信度：TZ00 读数 0.1°C 明显是无效值，交叉验证
Probe "thermal-cross-check" {
    $z = Get-CimInstance -Namespace root\wmi -ClassName MSAcpi_ThermalZoneTemperature
    Write-Output "热区数量 = $($z.Count)；读数(0.1K) = $($z.CurrentTemperature -join ', ')"
    Write-Output "判读：$([math]::Round(($z.CurrentTemperature[0]/10)-273.15,1)) °C —— 与 i5-12500H 实际负载温度不符，属固件未实现"
    Get-CimInstance -Namespace root\wmi -ClassName KernelThermalPolicyChange -EA SilentlyContinue |
        Select-Object Active, CriticalTripPoint | Format-List
    Write-Output "↑  若连 TripPoint 都为空，说明 ACPI 热管理在本机基本不可用"
}

# 6. GPU 利用率通路（NVIDIA 走 NVML，命令行工具是否存在）
Probe "nvml-availability" {
    $nvml = @('C:\Windows\System32\nvml.dll', 'C:\Program Files\NVIDIA Corporation\NVSMI\nvml.dll') |
        Where-Object { Test-Path $_ }
    Write-Output "nvml.dll: $(if ($nvml) { $nvml -join '; ' } else { '未在标准路径' })"
    $smi = Get-Command nvidia-smi -EA SilentlyContinue
    if ($smi) { nvidia-smi --query-gpu=name,utilization.gpu,memory.used,temperature.gpu --format=csv }
    else { Write-Output "nvidia-smi 不在 PATH" }
    $g = Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine -EA SilentlyContinue
    Write-Output "GPUEngine 计数器实例数 = $(($g | Measure-Object).Count)（UtilizationPercentage 常为 0，需实测）"
}

# 7. 权限基线：本机 UAC 关闭，无法区分"需管理员"项 —— 记录该结论
Probe "elevation-baseline" {
    $lua = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System' -Name EnableLUA -EA SilentlyContinue).EnableLUA
    $elev = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    Write-Output "EnableLUA=$lua  当前进程 elevated=$elev"
    if ($lua -eq 0) { Write-Output "⚠ 本机 UAC 已关闭 => 所有进程天然带管理员权限。本轮所有 OK 结论在普通 UAC 机器上可能变成 AccessDenied，需二次验证。" }
}

Write-Output "`n=== 第二轮结束 ==="
