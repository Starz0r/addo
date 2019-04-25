$PackageName = 'addo'
$InstallDir = Join-Path $(Get-ToolsLocation) $PackageName

Uninstall-BinFile addo -path "$InstallDir\addo.exe"

Remove-Item $InstallDir