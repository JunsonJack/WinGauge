# WinGauge W0 采集实测探测脚本
# 用途：逐项验证参考图里的每个卡片，在本机 Windows 上到底能不能读到、要不要管理员权限。
# 运行：powershell -NoProfile -ExecutionPolicy Bypass -File w0-probe.ps1 [-Section cpu,thermal]
# 说明：只读探测，不写入任何系统状态。

param([string]$Section = "all")

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$ErrorActionPreference = 'Continue'

function Probe {
    param([string]$Name, [scriptblock]$Body)
    if ($Section -ne "all" -and $Section -notlike "*$Name*") { return }
    Write-Output ""
    Write-Output "### $Name"
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        & $Body
        $script:Result = "OK"
    }
    catch {
        $msg = $_.Exception.Message -replace '\s+', ' '
        if ($msg.Length -gt 220) { $msg = $msg.Substring(0, 220) + "..." }
        Write-Output "FAIL: $msg"
        $script:Result = "FAIL"
    }
    $sw.Stop()
    Write-Output "--- ($Name => $($script:Result), 耗时 $($sw.ElapsedMilliseconds) ms)"
}

$Elevated = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
Write-Output "=== WinGauge W0 探测  $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') ==="
Write-Output "进程 elevated(管理员) = $Elevated"

# ---------- ① 设备头 ----------
Probe "identity" {
    $cs = Get-CimInstance Win32_ComputerSystem
    $os = Get-CimInstance Win32_OperatingSystem
    $cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
    $boot = $os.LastBootUpTime
    [PSCustomObject]@{
        Manufacturer = $cs.Manufacturer; Model = $cs.Model; HostName = $cs.Name
        OS           = "$($os.Caption) $($os.Version) build $($os.BuildNumber)"
        CPUName      = $cpu.Name.Trim()
        CoresLogical = $cpu.NumberOfLogicalProcessors; CoresPhysical = $cpu.NumberOfCores
        RamGB        = [math]::Round($cs.TotalPhysicalMemory / 1GB, 1)
        Uptime       = ((Get-Date) - $boot).ToString().Substring(0, 11)
    } | Format-List
}

# ---------- ③ CPU 使用率（每核） ----------
Probe "cpu-usage" {
    Get-CimInstance Win32_PerfFormattedData_PerfOS_Processor |
        Select-Object Name, PercentProcessorTime, PercentUserTime, PercentPrivilegedTime |
        Format-Table -AutoSize
    Write-Output "↑  locale-independent 的 WMI 性能类；_Total 行为总使用率"
}

Probe "cpu-queue" {
    Get-CimInstance Win32_PerfFormattedData_PerfOS_System |
        Select-Object Name, ProcessorQueueLength, Processes, Threads | Format-List
}

# ---------- PDH 英文计数器路径（本地化可用性验证，zh-CN 系统上是风险点） ----------
Probe "pdh-english-path" {
    $c = Get-Counter '\Processor Information(_Total)\% Processor Time' -ErrorAction Stop
    Write-Output "PDH 英文路径可用: $($c.CounterSamples[0].CookedValue)"
}

Probe "pdh-localized-path" {
    $c = Get-Counter '\System\Processor Queue Length' -ErrorAction Stop
    Write-Output "PDH \System\... 可用: $($c.CounterSamples[0].CookedValue)"
}

# ---------- ④ 内存 ----------
Probe "memory" {
    $os = Get-CimInstance Win32_OperatingSystem
    $totalGB = $os.TotalVisibleMemorySize / 1MB
    $freeGB = $os.FreePhysicalMemory / 1MB
    $commitLimitGB = $os.TotalVirtualMemorySize / 1MB
    $commitFreeGB = $os.FreeVirtualMemory / 1MB
    [PSCustomObject]@{
        PhysicalTotalGB = [math]::Round($totalGB, 2); PhysicalFreeGB = [math]::Round($freeGB, 2)
        PhysicalUsedPct = [math]::Round((1 - $freeGB / $totalGB) * 100, 1)
        CommitLimitGB   = [math]::Round($commitLimitGB, 2)
        CommitUsedGB    = [math]::Round(($commitLimitGB - $commitFreeGB), 2)
    } | Format-List
}

Probe "pagefile" {
    Get-CimInstance Win32_PageFileUsage |
        Select-Object Name, AllocatedBaseSize, CurrentUsage, PeakUsage | Format-Table -AutoSize
    Get-CimInstance Win32_PageFileSetting | Select-Object Name, InitialSize, MaximumSize | Format-Table -AutoSize
}

# ---------- 温度（参考图 51°C） ----------
Probe "thermal-msacpi" {
    $t = Get-CimInstance -Namespace root\wmi -ClassName MSAcpi_ThermalZoneTemperature -ErrorAction Stop
    $t | ForEach-Object {
        $k = $_.CurrentTemperature / 10.0
        [PSCustomObject]@{ Instance = $_.InstanceName; KelvinTenths = $_.CurrentTemperature; Celsius = [math]::Round($k - 273.15, 1) }
    } | Format-Table -AutoSize
}

Probe "thermal-msft-zone" {
    Get-CimInstance -Namespace root\standardcimv2 -ClassName MSFT_ThermalZone -ErrorAction Stop |
        Select-Object InstanceName, ActiveTripPoints | Format-List
}

Probe "thermal-probe-class" {
    Get-CimInstance Win32_TemperatureProbe | Select-Object InstanceName, CurrentReading | Format-List
    Write-Output "（Win32_TemperatureProbe 在多数现代机型上为空）"
}

# 列出 root\wmi 里所有含 thermal/fan 的类名，用于判断本机厂商暴露了什么
Probe "thermal-fan-classscan" {
    $names = Get-CimClass -Namespace root\wmi -ErrorAction Stop |
        Where-Object { $_.CimClassName -match 'therm|fan|temp|cool' } |
        Select-Object -ExpandProperty CimClassName
    if ($names) { $names | ForEach-Object { Write-Output "root\wmi  class: $_" } } else { Write-Output "root\wmi 无相关类" }
    $n2 = Get-CimClass -Namespace root\cimv2 |
        Where-Object { $_.CimClassName -match '^Win32_(Fan|Temperature|Cooling|Battery)' } |
        Select-Object -ExpandProperty CimClassName
    if ($n2) { $n2 | ForEach-Object { Write-Output "root\cimv2 class: $_" } }
}

# ---------- ⑤ 风扇（预判读不到，验证并记录） ----------
Probe "fan" {
    $f = Get-CimInstance Win32_Fan -ErrorAction Stop
    if ($f) { $f | Select-Object Name, DesiredSpeed, ActualSpeed | Format-Table -AutoSize }
    else { Write-Output "Win32_Fan 返回空集合（无实例）" }
}

# ---------- ⑥ 网络 ----------
Probe "net-adapters" {
    Get-NetAdapter | Where-Object { $_.Status -ne 'Not Present' } |
        Select-Object Name, InterfaceDescription, ifIndex, Status, LinkSpeed | Format-Table -AutoSize
}

Probe "net-throughput" {
    Get-CimInstance Win32_PerfFormattedData_Tcpip_NetworkInterface |
        Select-Object Name, BytesReceivedPersec, BytesSentPersec, BytesTotalPersec | Format-Table -AutoSize
    Write-Output "↑  Name 形如 'Intel..._0'，需要与 Get-NetAdapter 的 ifIndex 做映射"
}

Probe "net-default-route" {
    Get-NetRoute -DestinationPrefix '0.0.0.0/0' -ErrorAction Stop |
        Sort-Object RouteMetric | Select-Object -First 5 ifIndex, InterfaceAlias, NextHop, RouteMetric |
        Format-Table -AutoSize
}

# ---------- ⑦ 磁盘 ----------
Probe "disk-capacity" {
    Get-CimInstance Win32_LogicalDisk -Filter "DriveType=3" | ForEach-Object {
        [PSCustomObject]@{
            Device = $_.DeviceID; SizeGB = [math]::Round($_.Size / 1GB, 0)
            FreeGB = [math]::Round($_.FreeSpace / 1GB, 0)
            UsedPct = if ($_.Size) { [math]::Round((1 - $_.FreeSpace / $_.Size) * 100, 1) } else { 'n/a' }
            FileSystem = $_.FileSystem
        }
    } | Format-Table -AutoSize
}

Probe "disk-activity" {
    Get-CimInstance Win32_PerfFormattedData_PerfDisk_PhysicalDisk |
        Where-Object { $_.Name -match '^\d|^_Total|^0 ' } |
        Select-Object Name, PercentDiskTime, DiskReadBytesPersec, DiskWriteBytesPersec |
        Format-Table -AutoSize
}

Probe "disk-smart" {
    $p = Get-PhysicalDisk -ErrorAction Stop
    $p | Select-Object FriendlyName, MediaType, HealthStatus, OperationalStatus, Size | Format-Table -AutoSize
}

Probe "disk-reliability" {
    Get-StorageReliabilityCounter -ErrorAction Stop |
        Select-Object DeviceId, Temperature, PowerOnHours, Wear, ReadErrorsTotal | Format-Table -AutoSize
}

Probe "disk-temperature-wmi" {
    Get-CimInstance -Namespace root\wmi -ClassName MSFT_DiskTemperatureData -ErrorAction Stop | Format-List
}

# ---------- ⑧ 输入（只做可行性判定，不装钩子） ----------
Probe "input-hooks-feasibility" {
    Write-Output "全局低级钩子 WH_KEYBOARD_LL / WH_MOUSE_LL 需 Win32 SetWindowsHookEx，纯 Rust windows-rs 可实现。"
    Write-Output "本项记录：当前进程是否 elevated = $Elevated（elevated 进程对更高完整性窗口读不到，反之正常）"
    $uiPI = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System' -Name EnableLUA -EA SilentlyContinue).EnableLUA
    Write-Output "UAC EnableLUA = $uiPI"
}

# ---------- 电池 / GPU（Backlog 预判） ----------
Probe "battery" {
    Get-CimInstance Win32_Battery | Select-Object BatteryStatus, EstimatedChargeRemaining | Format-List
    Get-CimInstance -Namespace root\wmi -ClassName BatteryStaticData -EA SilentlyContinue |
        Select-Object DesignedCapacity, FullyChargedCapacity | Format-List
    Write-Output "（台式机通常无电池实例）"
}

Probe "gpu" {
    Get-CimInstance Win32_VideoController | Select-Object Name, AdapterRAM, DriverVersion | Format-Table -AutoSize
}

# ---------- 桌面形态相关 ----------
Probe "desktop-metrics" {
    Add-Type -AssemblyName System.Windows.Forms
    [Screen]::AllScreens | ForEach-Object {
        [PSCustomObject]@{
            Device = $_.DeviceName; Primary = $_.Primary
            Bounds = $_.Bounds; WorkingArea = $_.WorkingArea
            DPI = if ($_.Primary) { (Get-ItemProperty 'HKCU:\Control Panel\Desktop\WindowMetrics' -Name AppliedDPI -EA SilentlyContinue).AppliedDPI } else { '-' }
        }
    } | Format-Table -AutoSize
    Write-Output "↑  弹窗定位与钉住位置需按物理像素 + WorkingArea 收敛"
}

Write-Output ""
Write-Output "=== 探测结束 ==="
