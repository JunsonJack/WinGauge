# W0 补测：带正确入参调用联想 EC 方法（上一轮漏传 in 参数，结论待推翻/确认）
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$gz   = Get-CimInstance -Namespace root\wmi -ClassName LENOVO_GAMEZONE_DATA
$lfc  = Get-CimInstance -Namespace root\wmi -ClassName Lfc_thermal_interface
$fans = Get-CimInstance -Namespace root\wmi -ClassName LENOVO_FAN_METHOD

function Try-Call {
    param($inst, [string]$Method, [hashtable]$Params)
    try {
        $r = Invoke-CimMethod -InputObject $inst -MethodName $Method -Arguments $Params -EA Stop
        $ret = if ($r.ReturnValue -ne $null) { $r.ReturnValue } else { '-' }
        $payload = ($r | Get-Member -MemberType Properties |
                    Where-Object { $_.Name -notin @('ReturnValue','PSComputerName','CimClass','CimInstanceProperties','CimSystemProperties') } |
                    ForEach-Object { "$($_.Name)=$($r.($_.Name))" }) -join ' '
        return "OK  ret=$ret  $payload"
    } catch {
        $m = $_.Exception.Message -replace '\s+',' '
        if ($m.Length -gt 90) { $m = $m.Substring(0,90) }
        return "ERR $m"
    }
}

function Round([string]$Tag) {
    Write-Output ""
    Write-Output "########## $Tag  ($(Get-Date -Format 'HH:mm:ss')) ##########"
    foreach ($d in 0,1,2) {
        Write-Output ("GZ  GetCPUTemp  Data=$d => " + (Try-Call $gz 'GetCPUTemp'  @{ Data = [uint32]$d }))
        Write-Output ("GZ  GetGPUTemp  Data=$d => " + (Try-Call $gz 'GetGPUTemp'  @{ Data = [uint32]$d }))
        Write-Output ("GZ  GetIRTemp   Data=$d => " + (Try-Call $gz 'GetIRTemp'   @{ Data = [uint32]$d }))
    }
    foreach ($d in 0,1) {
        Write-Output ("GZ  GetFan1Speed Data=$d => " + (Try-Call $gz 'GetFan1Speed' @{ Data = [uint32]$d }))
        Write-Output ("GZ  GetFan2Speed Data=$d => " + (Try-Call $gz 'GetFan2Speed' @{ Data = [uint32]$d }))
        Write-Output ("Lfc GetFan1Speed Data=$d => " + (Try-Call $lfc 'GetFan1Speed' @{ Data = [uint32]$d }))
    }
    Write-Output ("GZ  GetFanCount    => " + (Try-Call $gz 'GetFanCount' @{ Data = [uint32]0 }))
    Write-Output ("GZ  GetCpuFrequency=> " + (Try-Call $gz 'GetCpuFrequency' @{ Data = [uint32]0 }))
    Write-Output ("GZ  GetProductInfo => " + (Try-Call $gz 'GetProductInfo' @{ Data = [uint32]0 }))
    Write-Output ("Lfc GetCPUTemperature Data=0 => " + (Try-Call $lfc 'GetCPUTemperature' @{ Data = [uint32]0 }))
    Write-Output ("Lfc GetEnvironmentTemp  Data=0 => " + (Try-Call $lfc 'GetEnvironmentTemperature' @{ Data = [uint32]0 }))
    Write-Output ("Lfc GetVersion           Data=0 => " + (Try-Call $lfc 'GetVersion' @{ Data = [uint32]0 }))
    foreach ($id in 0,1) {
        Write-Output ("FAN Fan_GetCurrentFanSpeed FanID=$id => " + (Try-Call $fans 'Fan_GetCurrentFanSpeed' @{ FanID = [byte]$id }))
        Write-Output ("FAN Fan_GetCurrentSensorTemp SensorID=$id => " + (Try-Call $fans 'Fan_GetCurrentSensorTemperature' @{ SensorID = [byte]$id }))
    }
    Write-Output ("ACPI ThermalZone => " + ((Get-CimInstance -Namespace root\wmi -ClassName MSAcpi_ThermalZoneTemperature -EA SilentlyContinue).CurrentTemperature -join ','))
    Write-Output ("CPU _Total usage => " + ((Get-CimInstance Win32_PerfFormattedData_PerfOS_Processor -Filter "Name='_Total'").PercentProcessorTime))
}

Round "空载基线"

Write-Output ""
Write-Output ">>> 施加 CPU 负载 30 秒（占用所有核心）"
$jobs = 1..8 | ForEach-Object {
    Start-Job -ScriptBlock { $e=(Get-Date).AddSeconds(30); $s=0; $i=0; while((Get-Date) -lt $e){ $i++; $s += [math]::Sqrt(($i % 9973) + 1) } }
}
Start-Sleep -Seconds 14
Round "负载中 +14s"
Start-Sleep -Seconds 12
Round "负载中 +26s"
$jobs | ForEach-Object { Stop-Job $_ -EA SilentlyContinue; Remove-Job $_ -EA SilentlyContinue }
Start-Sleep -Seconds 10
Round "撤载后 +10s"

Write-Output ""
Write-Output "=== 补测结束：判读标准 —— 数值随负载变化 = 真实可读；恒定 0 = 固件未实现；报错 = 通路不通 ==="
