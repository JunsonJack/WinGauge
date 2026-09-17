$ErrorActionPreference='Continue'
$inst = Get-CimInstance -Namespace root\wmi -ClassName LENOVO_FAN_METHOD
$r = Invoke-CimMethod -InputObject $inst -MethodName Fan_GetCurrentSensorTemperature -Arguments @{SensorID=[byte]3}
Write-Output "=== FULL OBJECT ==="
$r | Format-List * | Out-String | Write-Output
Write-Output "=== PROPS ==="
foreach ($p in $r.PSObject.Properties) {
  Write-Output "$($p.Name) type=$($p.TypeNameOfValue) val=$($p.Value)"
}
Write-Output "=== CIM PROPS ==="
foreach ($p in $r.CimInstanceProperties) {
  Write-Output "$($p.Name) val=$($p.Value) type=$($p.CimType)"
}
Write-Output "=== FAN0 ==="
$f = Invoke-CimMethod -InputObject $inst -MethodName Fan_GetCurrentFanSpeed -Arguments @{FanID=[byte]0}
$f | Format-List * | Out-String | Write-Output
foreach ($p in $f.CimInstanceProperties) {
  Write-Output "fanprop $($p.Name) val=$($p.Value)"
}
