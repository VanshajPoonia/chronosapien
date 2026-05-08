param(
    [string]$Image = "target\x86_64-unknown-none\debug\chronosapien-uefi.img",
    [string]$OvmfCode = $env:OVMF_CODE
)

$ErrorActionPreference = "Stop"

function Find-OvmfCode {
    param([string]$ExplicitPath)

    if ($ExplicitPath -and (Test-Path $ExplicitPath)) {
        return $ExplicitPath
    }

    $candidates = @(
        "C:\Program Files\qemu\share\edk2-x86_64-code.fd",
        "C:\Program Files\qemu\share\OVMF_CODE.fd",
        "/usr/share/OVMF/OVMF_CODE.fd",
        "/usr/share/edk2-ovmf/x64/OVMF_CODE.fd",
        "/opt/homebrew/share/qemu/edk2-x86_64-code.fd",
        "/usr/local/share/qemu/edk2-x86_64-code.fd"
    )

    foreach ($candidate in $candidates) {
        if (Test-Path $candidate) {
            return $candidate
        }
    }

    throw "OVMF firmware not found. Set OVMF_CODE to an OVMF_CODE.fd or edk2-x86_64-code.fd path."
}

if (-not (Test-Path $Image)) {
    Write-Host "UEFI image not found. Building it first..."
    .\scripts\build-uefi.ps1
}

$ovmf = Find-OvmfCode $OvmfCode

qemu-system-x86_64 `
    -machine q35 `
    -smp 2 `
    -drive "if=pflash,format=raw,readonly=on,file=$ovmf" `
    -drive "format=raw,file=$Image,if=virtio,media=disk" `
    -serial stdio
