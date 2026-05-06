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
    -display none `
    -serial stdio
