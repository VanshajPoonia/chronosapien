$ErrorActionPreference = "Stop"

Write-Host "Building Chronosapian BIOS image..."
$hostTarget = ((rustc -vV | Select-String "^host:").ToString() -split " ")[1]
cargo build -p chronosapien --target $hostTarget
