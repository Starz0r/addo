$PackageName = 'addo'
$Url = "https://github.com/Starz0r/addo/releases/download/v0.9.0/addo-i686-0.9.0-msvc.exe"
$Checksum = "37C53F462238F3CAFF7A3E46211BFCB194CD2EB81DEF7979F3AB62F6F16B77625AEF1F6B2523CD3D36C6AF0A9C6AE929FAAE8D7D1A1EE162CD67812DB07CFDF1"
$ChecksumType = 'sha512'
$Url64 = 'https://github.com/Starz0r/addo/releases/download/v0.9.0/addo-x86_64-0.9.0-msvc.exe'
$Checksum64 = '9670ED4ADFAE90C638C01102D0F5050E4775F4254A50E16782497918E88424320DF9A8EEE2EAA02EB06C9728B501D9584E8497A83CBD725D96379023AD22AF03'
$ChecksumType64 = 'sha512'
$ToolsPath = Split-Path -Parent $MyInvocation.MyCommand.Definition
$InstallDir = Join-Path $(Get-ToolsLocation) $PackageName

$desktop = [System.Environment]::GetFolderPath("Desktop")

$PackageArgs = @{
	PackageName = $PackageName
	Url = $Url
	Checksum = $Checksum
	ChecksumType = $ChecksumType
	Url64 = $Url64
	Checksum64 = $Checksum64
	ChecksumType64 = $ChecksumType64
	FileFullPath = Join-Path $InstallDir $([IO.Path]::GetFileName("addo.exe"))
}
Get-ChocolateyWebFile @PackageArgs

Install-BinFile addo -path "$InstallDir\addo.exe" -UseStart