param([ValidateRange(1, 10000)][int]$Iterations = 20)

$ErrorActionPreference = "Stop"
foreach ($tool in "cargo", "rustc", "clang") {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
        throw "required benchmark tool '$tool' was not found in PATH"
    }
}
$root = $PSScriptRoot
$repository = (Resolve-Path (Join-Path $root "../..")).Path
$output = Join-Path $root "target"
New-Item -ItemType Directory -Force -Path $output | Out-Null

Push-Location $repository
try {
    cargo build --release --locked -p adamantium-cli
    if ($LASTEXITCODE -ne 0) { throw "could not build the Adamantium CLI" }
    $adamantium = Join-Path $repository "target/release/adamantium.exe"
    $adamantiumProject = Join-Path $root "adamantium"
    $adamantiumExe = Join-Path $adamantiumProject "target/LanguageComparison.exe"
    $cExe = Join-Path $output "comparison-c.exe"
    $rustExe = Join-Path $output "comparison-rust.exe"

    $results = @()
    $builds = @(
        @{ Name = "Adamantium"; Executable = $adamantiumExe; Build = { & $adamantium build $adamantiumProject -O2 | Out-Null } },
        @{ Name = "C"; Executable = $cExe; Build = { & clang -O2 (Join-Path $root "c/main.c") -o $cExe } },
        @{ Name = "Rust"; Executable = $rustExe; Build = { & rustc -C opt-level=3 (Join-Path $root "rust/main.rs") -o $rustExe } }
    )

    foreach ($entry in $builds) {
        $timer = [Diagnostics.Stopwatch]::StartNew()
        & $entry.Build
        if ($LASTEXITCODE -ne 0) { throw "$($entry.Name) build failed" }
        $timer.Stop()
        $actual = (& $entry.Executable).Trim()
        if ($actual -ne "6") { throw "$($entry.Name) returned unexpected output '$actual'" }

        $run = [Diagnostics.Stopwatch]::StartNew()
        for ($index = 0; $index -lt $Iterations; ++$index) {
            & $entry.Executable | Out-Null
            if ($LASTEXITCODE -ne 0) { throw "$($entry.Name) execution failed" }
        }
        $run.Stop()
        $results += [pscustomobject]@{
            Language = $entry.Name
            BuildMilliseconds = [math]::Round($timer.Elapsed.TotalMilliseconds, 3)
            ExecutableBytes = (Get-Item -LiteralPath $entry.Executable).Length
            AverageRunMilliseconds = [math]::Round($run.Elapsed.TotalMilliseconds / $Iterations, 3)
        }
    }
    $results | ConvertTo-Csv -NoTypeInformation
} finally {
    Pop-Location
}
