# 打免安装（绿色）包：把 release exe 拷成 WinGauge.exe 并压 zip
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root 'target\release\app.exe'
if (-not (Test-Path $exe)) {
    Write-Error "未找到 $exe，请先执行 npm run tauri:build"
}
$dist = Join-Path $root 'dist-portable'
$stage = Join-Path $dist 'WinGauge-portable'
if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
New-Item -ItemType Directory -Path $stage | Out-Null

Copy-Item $exe (Join-Path $stage 'WinGauge.exe')

$readme = @"
WinGauge 绿色版（免安装）
========================

1. 双击 WinGauge.exe 即可运行（Win10/11，一般自带 WebView2）
2. 托盘图标在任务栏右侧，可固定
3. 不需要管理员权限；退出请用托盘菜单「退出 WinGauge」
4. 开机自启写在当前用户 HKCU，卸载绿色版前可在设置里关掉

版本：v0.1.0
"@
Set-Content -Path (Join-Path $stage '说明.txt') -Value $readme -Encoding UTF8

$zip = Join-Path $dist 'WinGauge-0.1.0-portable.zip'
if (Test-Path $zip) { Remove-Item -Force $zip }
Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip
Write-Host "OK $zip"
Get-Item $zip | Select-Object FullName, @{n='MB';e={[math]::Round($_.Length/1MB,2)}}
