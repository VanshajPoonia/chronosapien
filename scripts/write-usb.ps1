param(
    [string]$Image = "target\x86_64-unknown-none\debug\chronosapien-uefi.img",
    [int]$DiskNumber = -1,
    [switch]$ConfirmWrite
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $Image)) {
    throw "UEFI image not found at $Image. Run .\scripts\build-uefi.ps1 first."
}

if (-not (Get-Command Get-Disk -ErrorAction SilentlyContinue)) {
    throw "This guarded writer uses Windows Storage cmdlets. On other hosts, use a platform-native raw disk writer with the same image."
}

if ($DiskNumber -lt 0) {
    Get-Disk | Format-Table Number, FriendlyName, BusType, Size, PartitionStyle, IsBoot, IsSystem
    throw "Pass -DiskNumber <n> after identifying the USB disk."
}

if (-not $ConfirmWrite) {
    throw "Refusing to write without -ConfirmWrite."
}

$disk = Get-Disk -Number $DiskNumber
if ($disk.IsBoot -or $disk.IsSystem) {
    throw "Refusing to write to a boot/system disk."
}

$expected = "WRITE CHRONOOS TO DISK $DiskNumber"
Write-Host "This will overwrite disk $DiskNumber: $($disk.FriendlyName), $([math]::Round($disk.Size / 1GB, 2)) GiB"
$typed = Read-Host "Type '$expected' to continue"
if ($typed -ne $expected) {
    throw "Confirmation text did not match."
}

$imageBytes = [System.IO.File]::ReadAllBytes($Image)
$physicalDrive = "\\.\PhysicalDrive$DiskNumber"

Write-Host "Taking disk offline..."
Set-Disk -Number $DiskNumber -IsOffline $true

try {
    Write-Host "Writing $Image to $physicalDrive..."
    $stream = [System.IO.File]::Open($physicalDrive, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Write)
    try {
        $stream.Write($imageBytes, 0, $imageBytes.Length)
        $stream.Flush()
    } finally {
        $stream.Close()
    }
} finally {
    Write-Host "Bringing disk online..."
    Set-Disk -Number $DiskNumber -IsOffline $false
}

Write-Host "USB image write complete. Boot it from UEFI with Secure Boot disabled."
