Write-Host "Compiling Rust project..."

$cargoPath = "C:\Users\Administrator\.cargo\bin\cargo.exe"
if (-not (Test-Path $cargoPath)) {
    $cargoPath = "cargo"
}

# Run Cargo release build
& $cargoPath build --release

if ($LASTEXITCODE -eq 0) {
    if (-not (Test-Path "build")) {
        New-Item -Path "build" -ItemType "directory" | Out-Null
    }
    # Copy build artifact to outputs
    Copy-Item -Path "target\release\copy-path-tool.exe" -Destination "build\CopyPathTool.exe" -Force
    Write-Host "=================================================="
    Write-Host "Build complete! Output saved to: build\CopyPathTool.exe"
    Write-Host "=================================================="
} else {
    Write-Error "Build failed!"
    exit $LASTEXITCODE
}
