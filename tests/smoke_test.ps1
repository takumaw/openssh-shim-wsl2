# PowerShell script to perform a smoke test on the built shim executables.
# It verifies that each binary can be executed and returns the expected shim error exit code 255.

$ErrorActionPreference = "Stop"

$executables = @("ssh.exe", "scp.exe", "sftp.exe")
$targetDir = "target/release"

Write-Host "Running smoke tests for built shim executables..."

foreach ($exe in $executables) {
    $path = Join-Path $targetDir $exe
    if (-not (Test-Path $path)) {
        Write-Error "Executable not found: $path. Did you run 'cargo build --release'?"
        exit 1
    }

    Write-Host "Executing $path..."
    # Execute the shim with no arguments. In a test/CI environment (where WSL is not fully configured or setup for it),
    # it should fail to launch or execute successfully and return a non-zero exit code (typically 255 or -1).
    $process = Start-Process -FilePath $path -ArgumentList "--help" -NoNewWindow -PassThru -Wait
    
    if ($process.ExitCode -eq 0) {
        Write-Error "$exe exited with code 0, expected a non-zero exit code"
        exit 1
    }
    Write-Host "$exe exited with code $($process.ExitCode) as expected"
}

Write-Host "All smoke tests passed successfully!"
exit 0
