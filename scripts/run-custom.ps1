$ErrorActionPreference = "Stop"

$image = "target\x86_64-unknown-none\debug\chronosapien-custom.img"

if (-not (Test-Path $image)) {
    Write-Host "Custom BIOS image not found. Building it first..."
    .\scripts\build-custom.ps1
}

qemu-system-x86_64 `
    -drive "format=raw,file=$image" `
    -netdev "user,id=net0,hostfwd=udp::9000-:9000" `
    -device "rtl8139,netdev=net0,mac=52:54:00:12:34:56" `
    -serial stdio
