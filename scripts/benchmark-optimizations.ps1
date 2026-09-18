param(
    [string]$Project = "benchmarks/optimization",
    [ValidateRange(1, 10000)][int]$Iterations = 20
)

$ErrorActionPreference = "Stop"
$projectRoot = (Resolve-Path -LiteralPath $Project).Path
$manifest = Get-Content -LiteralPath (Join-Path $projectRoot "project.toml") -Raw
if ($manifest -notmatch '(?m)^name\s*=\s*"([^"]+)"') { throw "project.toml has no name" }
$name = $Matches[1]
$cli = Join-Path $PSScriptRoot "../target/release/adamantium.exe"
cargo build --release -p adamantium-cli
if ($LASTEXITCODE -ne 0) { throw "could not build Adamantium CLI" }

foreach ($level in "-O0", "-O1", "-O2") {
    $timer = [Diagnostics.Stopwatch]::StartNew()
    & $cli build $projectRoot $level | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "build failed for $level" }
    $timer.Stop()
    $executable = Join-Path $projectRoot "target/$name.exe"
    $assembly = Join-Path $projectRoot "target/$name.asm"
    $run = [Diagnostics.Stopwatch]::StartNew()
    for ($index = 0; $index -lt $Iterations; $index++) {
        $process = Start-Process -FilePath $executable -PassThru -WindowStyle Hidden
        if (-not $process.WaitForExit(30000)) {
            $process.Kill()
            throw "program timed out for $level"
        }
        if ($process.ExitCode -ne 0) { throw "program failed for $level" }
    }
    $run.Stop()
    [pscustomobject]@{
        Level = $level
        CompileMilliseconds = [math]::Round($timer.Elapsed.TotalMilliseconds, 3)
        AssemblyBytes = (Get-Item -LiteralPath $assembly).Length
        ExecutableBytes = (Get-Item -LiteralPath $executable).Length
        AverageRunMilliseconds = [math]::Round($run.Elapsed.TotalMilliseconds / $Iterations, 3)
    }
}
