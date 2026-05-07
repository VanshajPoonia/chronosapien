$ErrorActionPreference = "Stop"

$dataImage = "target\x86_64-unknown-none\debug\chronofs-data.img"
$toolDir = "target\tools"
$tool = "$toolDir\chronofs-put.exe"
$object = "user\hello.o"
$elf = "user\hello.elf"

New-Item -ItemType Directory -Force -Path $toolDir | Out-Null

if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    throw "rustc is required to build tools\chronofs_put.rs"
}
if (-not (Get-Command clang -ErrorAction SilentlyContinue)) {
    throw "clang is required to build user\hello.c"
}
if (-not (Get-Command ld.lld -ErrorAction SilentlyContinue)) {
    throw "ld.lld is required to link user\hello.elf"
}

Write-Host "Building ChronoFS injector..."
rustc tools\chronofs_put.rs -o $tool

Write-Host "Compiling user hello object..."
clang `
    -target x86_64-unknown-none `
    -ffreestanding `
    -fno-stack-protector `
    -fno-pic `
    -mno-red-zone `
    -c `
    -o $object `
    user\hello.c

Write-Host "Linking user hello ELF..."
ld.lld `
    -T user\user.ld `
    --build-id=none `
    -o $elf `
    $object

Write-Host "Installing hello.elf into ChronoFS data disk..."
& $tool $dataImage $elf hello.elf

Write-Host "Installed hello.elf. Boot ChronoOS and run: exec hello.elf"
