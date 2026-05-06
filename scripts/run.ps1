$ErrorActionPreference = "Stop"

$image = "target\x86_64-unknown-none\debug\chronosapien-bios.img"
$dataImage = "target\x86_64-unknown-none\debug\chronofs-data.img"
$dataImageBytes = 16 * 1024 * 1024

if (-not (Test-Path $image)) {
    Write-Host "BIOS image not found. Building it first..."
    .\scripts\build.ps1
}

if (-not (Test-Path $dataImage)) {
    Write-Host "ChronoFS data disk not found. Creating 16 MiB image..."
    New-Item -ItemType Directory -Force -Path (Split-Path $dataImage) | Out-Null
    $stream = [System.IO.File]::Open($dataImage, [System.IO.FileMode]::CreateNew, [System.IO.FileAccess]::ReadWrite)
    $stream.SetLength($dataImageBytes)
    $stream.Close()
}

qemu-system-x86_64 `
    -drive "format=raw,file=$image,if=ide,index=0,media=disk" `
    -drive "format=raw,file=$dataImage,if=ide,index=1,media=disk" `
    -netdev "user,id=net0,hostfwd=udp::9000-:9000" `
    -device "rtl8139,netdev=net0,mac=52:54:00:12:34:56" `
    -serial stdio
