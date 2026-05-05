$ErrorActionPreference = "Stop"

$image = "target\x86_64-unknown-none\debug\chronosapien-custom.img"

if (-not (Test-Path $image)) {
    Write-Host "Custom BIOS image not found. Building it first..."
    .\scripts\build-custom.ps1
}

qemu-system-x86_64 `
    -drive "format=raw,file=$image" `
    -serial stdio
