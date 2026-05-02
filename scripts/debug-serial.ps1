$ErrorActionPreference = "Stop"

$image = "target\x86_64-unknown-none\debug\chronosapien-bios.img"

if (-not (Test-Path $image)) {
    Write-Host "BIOS image not found. Building it first..."
    .\scripts\build.ps1
}

qemu-system-x86_64 `
    -drive "format=raw,file=$image" `
    -display none `
    -serial stdio
