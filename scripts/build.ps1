$ErrorActionPreference = "Stop"

Write-Host "Building Time Capsule OS boot image..."
cargo bootimage -p kernel
