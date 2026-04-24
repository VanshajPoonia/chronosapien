$ErrorActionPreference = "Stop"

$image = "target\x86_64-unknown-none\debug\bootimage-kernel.bin"

if (-not (Test-Path $image)) {
    Write-Host "Boot image not found. Building it first..."
    cargo bootimage -p kernel
}

qemu-system-x86_64 `
    -drive "format=raw,file=$image" `
    -serial stdio
