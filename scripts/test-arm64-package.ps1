$ErrorActionPreference = "Stop"

if (-not $env:TARGET -or -not $env:EXECUTABLE) {
    throw "TARGET and EXECUTABLE are required."
}

$root = Split-Path -Parent $PSScriptRoot
$dist = Join-Path $root "dist"
$name = "adamantium-$($env:TARGET)-packaging-smoke"
$stage = Join-Path $dist $name
$archive = "$stage.zip"
$cliName = if ($env:TARGET -like "*-windows-*") { "adamantium.exe" } else { "adamantium" }

if (Test-Path -LiteralPath $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
if (Test-Path -LiteralPath $archive) { Remove-Item -LiteralPath $archive -Force }
New-Item -ItemType Directory -Force -Path $stage | Out-Null
Copy-Item -LiteralPath (Join-Path $root "target/release/$cliName") -Destination $stage
Copy-Item -LiteralPath (Join-Path $root $env:EXECUTABLE) -Destination $stage
Copy-Item -LiteralPath (Join-Path $root "README.md") -Destination $stage
Copy-Item -LiteralPath (Join-Path $root "LICENSE.md") -Destination $stage
@"
This archive validates ARM64 release layout only. It is not yet the complete
portable Adamantium toolchain and does not claim bundled native build tools.
Target: $($env:TARGET)
"@ | Set-Content -LiteralPath (Join-Path $stage "PACKAGING_STATUS.txt") -Encoding UTF8

Compress-Archive -LiteralPath $stage -DestinationPath $archive
$verify = Join-Path $dist "$name-verify"
if (Test-Path -LiteralPath $verify) { Remove-Item -LiteralPath $verify -Recurse -Force }
Expand-Archive -LiteralPath $archive -DestinationPath $verify
$contents = Join-Path $verify $name
foreach ($required in $cliName, $env:EXECUTABLE, "README.md", "LICENSE.md", "PACKAGING_STATUS.txt") {
    if (-not (Test-Path -LiteralPath (Join-Path $contents $required) -PathType Leaf)) {
        throw "ARM64 archive is missing $required."
    }
}
if ($env:TARGET -notlike "*-windows-*") {
    & chmod +x (Join-Path $contents $cliName) (Join-Path $contents $env:EXECUTABLE)
    if ($LASTEXITCODE -ne 0) { throw "Could not restore executable permissions." }
}
& (Join-Path $contents $cliName) --version | Out-Null
if ($LASTEXITCODE -ne 0) { throw "Packaged ARM64 CLI failed." }
& (Join-Path $contents $env:EXECUTABLE)
if ($LASTEXITCODE -ne 0) { throw "Packaged ARM64 smoke executable failed." }
Remove-Item -LiteralPath $verify -Recurse -Force
Write-Output "Verified $archive"
