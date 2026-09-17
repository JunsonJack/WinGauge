# W0 第三轮：实测调用联想 EC 温度/风扇方法，并用 CPU 负载交叉验证读数真实性
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

function Call {
    param($inst, [string]$Method, [hashtable]$Params)
    try {
        if ($Params) { $r = Invoke-CimMethod -InputObject $inst -MethodName $Method -Arguments $Params -EA Stop }
        else { $r = Invoke-CimMethod -InputObject $inst -MethodName $Method -EA Stop }
        return ($r | ConvertTo-Json -Compress -Depth 3)
    } catch {
        $m = $_.Exception.Message -replace '\s+', ' '
        if ($m.Length -gt 120) { $m = $m.Substring(0, 120) + '...' }
        return "ERR: $m"
    }
}

Write-Output "=== W0 第三轮 $(Get-Date -Format 'HH:mm:ss') ==="

$gz = Get-CimInstance -Namespace root\wmi -ClassName LENOVO_GAMEZONE_DATA -EA SilentlyContinue
$lfc = Get-CimInstance -Namespace root\wmi -ClassName Lfc_thermal_interface -EA SilentlyContinue
$fanm = Get-CimInstance -Namespace root\wmi -ClassName LENOVO_FAN_METHOD -EA SilentlyContinue
Write-Output "实例存在性: GAMEZONE_DATA=$([bool]$gz)  Lfc_thermal_interface=$([bool]$lfc)  FAN_METHOD=$([bool]$fanm)"

function Snapshot([string]$Tag) {
    Write-Output ""
    Write-Output "---- 采样点: $Tag  ($(Get-Date -Format 'HH:mm:ss')) ----"
    Write-Output ("GAMEZONE GetCPUTemp        => " + (Call $gz 'GetCPUTemp'))
    Write-Output ("GAMEZONE GetGPUTemp        => " + (Call $gz 'GetGPUTemp'))
    Write-Output ("GAMEZONE GetIRTemp         => " + (Call $gz 'GetIRTemp'))
    Write-Output ("GAMEZONE GetFanCount       => " + (Call $gz 'GetFanCount'))
    Write-Output ("GAMEZONE GetFan1Speed      => " + (Call $gz 'GetFan1Speed'))
    Write-Output ("GAMEZONE GetFan2Speed      => " + (Call $gz 'GetFan2Speed'))
    Write-Output ("GAMEZONE GetFanMaxSpeed    => " + (Call $gz 'GetFanMaxSpeed' @{ FanNumber = 1 }))
    Write-Output ("GAMEZONE GetThermalMode    => " + (Call $gz 'GetThermalMode'))
    Write-Output ("Lfc GetCPUTemperature      => " + (Call $lfc 'GetCPUTemperature'))
    Write-Output ("Lfc GetEnvironmentTemp     => " + (Call $lfc 'GetEnvironmentTemperature'))
    Write-Output ("Lfc GetSSDTemperature      => " + (Call $lfc 'GetSSDTemperature'))
    Write-Output ("Lfc GetRAMTemperature      => " + (Call $lfc 'GetRAMTemperature'))
    Write-Output ("FAN_METHOD CurrentFanSpeed => " + (Call $fanm 'Fan_GetCurrentFanSpeed'))
    Write-Output ("FAN_METHOD SensorTemp      => " + (Call $fanm 'Fan_GetCurrentSensorTemperature'))
    Write-Output ("ACPI ThermalZone           => " + ((Get-CimInstance -Namespace root\wmi -ClassName MSAcpi_ThermalZoneTemperature -EA SilentlyContinue).CurrentTemperature -join ','))
    Write-Output ("WMI CPU _Total usage       => " + ((Get-CimInstance Win32_PerfFormattedData_PerfOS_Processor -Filter "Name='_Total'").PercentProcessorTime))
}

Snapshot "空载基线"

Write-Output ""
Write-Output ">>> 制造 CPU 负载：4 个后台 busy loop，持续 25 秒"
$jobs = @()
1..4 | ForEach-Object {
    $jobs += Start-Job -ScriptBlock { $x = 0; $end = (Get-Date).AddSeconds(25); while ((Get-Date) -lt $end) { $x += [math]::Sqrt($args[0]) } } -ArgumentList (Get-Random -Maximum 100000)
}
Start-Sleep -Seconds 12
Snapshot "负载中(+12s)"
Start-Sleep -Seconds 10
Snapshot "负载中(+22s)"
$jobs | ForEach-Object { Stop-Job $_ -EA SilentlyContinue; Remove-Job $_ -EA SilentlyContinue }
Start-Sleep -Seconds 8
Snapshot "撤载后(+8s，应回落)"

Write-Output ""
Write-Output "### 网络计数源补测"
Write-Output "-- Get-NetAdapterStatistics（不加过滤）--"
Get-NetAdapterStatistics | Select-Object Name, ifIndex, OperationalStatus, ReceivedBytes, SentBytes | Format-Table -AutoSize
Write-Output "-- Get-NetIPInterface + ifIndex 主接口判定 --"
$def = Get-NetRoute -DestinationPrefix '0.0.0.0/0' | Sort-Object RouteMetric | Select-Object -First 1
Write-Output "默认路由 ifIndex=$($def.ifIndex) alias=$($def.InterfaceAlias)"
$stat = Get-NetAdapterStatistics | Where-Object { $_.ifIndex -eq $def.ifIndex }
Write-Output "该接口累计 RX=$($stat.ReceivedBytes) TX=$($stat.SentBytes)  →  两次采样差值即为速率"

Write-Output ""
Write-Output "=== 第三轮结束 ==="
