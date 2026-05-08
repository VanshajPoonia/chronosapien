$ErrorActionPreference = "Stop"

$kernelTargetDir = "target\x86_64-unknown-none\debug"
$uefiTargetDir = "target\x86_64-unknown-uefi\debug"
$builderDir = "$kernelTargetDir\uefi"
$builder = "$builderDir\uefi_image_builder.exe"
$kernel = "$kernelTargetDir\kernel"
$loader = "$uefiTargetDir\uefi-loader.efi"
$image = "$kernelTargetDir\chronosapien-uefi.img"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo is required to build ChronoOS."
}

if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    throw "rustc is required for the UEFI image builder."
}

if (Get-Command rustup -ErrorAction SilentlyContinue) {
    rustup target add x86_64-unknown-uefi | Out-Host
}

New-Item -ItemType Directory -Force -Path $builderDir | Out-Null

Write-Host "Building ChronoOS kernel..."
cargo build -p kernel

Write-Host "Building ChronoOS UEFI loader..."
cargo build -p uefi-loader --target x86_64-unknown-uefi

if (-not (Test-Path $kernel)) {
    throw "Kernel ELF not found at $kernel"
}

if (-not (Test-Path $loader)) {
    throw "UEFI loader not found at $loader"
}

Write-Host "Building UEFI image builder..."
rustc tools\uefi_image_builder.rs -o $builder

Write-Host "Creating GPT/FAT32 ESP image..."
& $builder $loader $kernel $image

Write-Host "UEFI image written to $image"
Write-Host "ESP layout:"
Write-Host "  EFI\BOOT\BOOTX64.EFI"
Write-Host "  CHRONO\KERNEL.ELF"
