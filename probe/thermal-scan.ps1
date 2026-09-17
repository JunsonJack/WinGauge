$ErrorActionPreference='SilentlyContinue'
$inst = Get-CimInstance -Namespace root\wmi -ClassName LENOVO_FAN_METHOD
if (-not $inst) { Write-Output 'CLASS_MISSING'; exit 0 }
Write-Output 'CLASS_OK'
foreach ($id in 0,1,2,3,4,5,6,7,8,9,10,11,12) {
  $r = Invoke-CimMethod -InputObject $inst -MethodName Fan_GetCurrentSensorTemperature -Arguments @{SensorID=[byte]$id}
  $rv = if ($r) { $r.ReturnValue } else { 'null' }
  $vals = @()
  if ($r) {
    foreach ($p in $r.CimInstanceProperties) {
      if ($p.Name -notin @('ReturnValue','PSComputerName')) {
        $vals += "$($p.Name)=$($p.Value)"
      }
    }
  }
  Write-Output "T id=$id rv=$rv $($vals -join ' ')"
}
foreach ($id in 0,1,2,3) {
  $r = Invoke-CimMethod -InputObject $inst -MethodName Fan_GetCurrentFanSpeed -Arguments @{FanID=[byte]$id}
  $rv = if ($r) { $r.ReturnValue } else { 'null' }
  $vals = @()
  if ($r) {
    foreach ($p in $r.CimInstanceProperties) {
      if ($p.Name -notin @('ReturnValue','PSComputerName')) {
        $vals += "$($p.Name)=$($p.Value)"
      }
    }
  }
  Write-Output "F id=$id rv=$rv $($vals -join ' ')"
}
