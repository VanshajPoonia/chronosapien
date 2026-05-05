$ErrorActionPreference = "Stop"

$targetDir = "target\x86_64-unknown-none\debug"
$stageDir = "$targetDir\custom-boot"
$stage1Asm = "boot\stage1\stage1.asm"
$stage2Asm = "boot\stage2\stage2_real.asm"
$stage1Bin = "$stageDir\stage1.bin"
$stage2Bin = "$stageDir\stage2.bin"
$builder = "$stageDir\custom_image_builder.exe"
$kernel = "$targetDir\kernel"
$image = "$targetDir\chronosapien-custom.img"

if (-not (Get-Command nasm -ErrorAction SilentlyContinue)) {
    throw "nasm is required for the custom bootloader path."
}

if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    throw "rustc is required for the custom image builder."
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo is required to build the ChronoOS kernel."
}

New-Item -ItemType Directory -Force -Path $stageDir | Out-Null

Write-Host "Assembling custom Stage 1..."
nasm -f bin $stage1Asm -o $stage1Bin

Write-Host "Assembling custom Stage 2..."
nasm -f bin $stage2Asm -o $stage2Bin

Write-Host "Building ChronoOS kernel..."
cargo build -p kernel

Write-Host "Building custom image builder..."
rustc tools\custom_image_builder.rs -o $builder

Write-Host "Creating ChronoOS custom BIOS image..."
& $builder $stage1Bin $stage2Bin $kernel $image

$stage1Bytes = [System.IO.File]::ReadAllBytes($stage1Bin)
if ($stage1Bytes.Length -ne 512) {
    throw "Stage 1 must be exactly 512 bytes."
}

if ($stage1Bytes[510] -ne 0x55 -or $stage1Bytes[511] -ne 0xAA) {
    throw "Stage 1 is missing the BIOS 55 AA signature."
}

Write-Host "Custom image written to $image"
Write-Host "Manifest written to $($image -replace '\.img$', '.manifest.txt')"
