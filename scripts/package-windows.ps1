param(
    [string]$NasmVersion = "3.02",
    [string]$NasmSha256 = "161D0BFAFF53C2F9E9F3E69FD0672323EBABAFD1268976A5CEC11BE92A19AEE7",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$compilerRoot = Split-Path -Parent $PSScriptRoot
$distributionRoot = Join-Path $compilerRoot "dist"
$packageRoot = Join-Path $distributionRoot "adamantium-windows-x86_64"
$archivePath = Join-Path $distributionRoot "nasm-$NasmVersion-win64.zip"
$extractedPath = Join-Path $distributionRoot "nasm-$NasmVersion"

if (-not $SkipBuild) {
    $env:RUSTFLAGS = "-C target-feature=+crt-static"
    cargo build --manifest-path (Join-Path $compilerRoot "Cargo.toml") --locked --release -p adamantium-cli
    if ($LASTEXITCODE -ne 0) { throw "Cargo release build failed." }
}

New-Item -ItemType Directory -Force -Path $distributionRoot | Out-Null
Invoke-WebRequest `
    -Uri "https://www.nasm.us/pub/nasm/releasebuilds/$NasmVersion/win64/nasm-$NasmVersion-win64.zip" `
    -OutFile $archivePath
$actualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $archivePath).Hash
if ($actualHash -ne $NasmSha256) {
    throw "NASM archive checksum mismatch: expected $NasmSha256, found $actualHash."
}

if (Test-Path -LiteralPath $extractedPath) {
    Remove-Item -LiteralPath $extractedPath -Recurse -Force
}
Expand-Archive -LiteralPath $archivePath -DestinationPath $extractedPath
if (Test-Path -LiteralPath $packageRoot) {
    Remove-Item -LiteralPath $packageRoot -Recurse -Force
}
New-Item -ItemType Directory -Force -Path (Join-Path $packageRoot "tools") | Out-Null
Copy-Item -LiteralPath (Join-Path $compilerRoot "target/release/adamantium.exe") -Destination $packageRoot
$nasmExecutable = Get-ChildItem -LiteralPath $extractedPath -Filter "nasm.exe" -Recurse | Select-Object -First 1
if (-not $nasmExecutable) { throw "The NASM archive does not contain nasm.exe." }
Copy-Item -LiteralPath $nasmExecutable.FullName -Destination (Join-Path $packageRoot "tools/nasm.exe")

$rustSysroot = (& rustc --print sysroot).Trim()
$rustLld = Join-Path $rustSysroot "lib/rustlib/x86_64-pc-windows-msvc/bin/rust-lld.exe"
if (-not (Test-Path -LiteralPath $rustLld -PathType Leaf)) {
    throw "The Rust toolchain does not contain rust-lld.exe at $rustLld."
}
Copy-Item -LiteralPath $rustLld -Destination (Join-Path $packageRoot "tools/lld-link.exe")
$rustCopyright = Join-Path $rustSysroot "share/doc/rust/COPYRIGHT.html"
if (-not (Test-Path -LiteralPath $rustCopyright -PathType Leaf)) {
    throw "The Rust toolchain does not contain its third-party copyright notice."
}
Copy-Item -LiteralPath $rustCopyright -Destination (Join-Path $packageRoot "LLVM-RUST-COPYRIGHT.html")

$libraryRoot = Join-Path $packageRoot "tools/lib"
New-Item -ItemType Directory -Force -Path $libraryRoot | Out-Null
$windowsLibraries = @(
    "advapi32.lib",
    "bcrypt.lib",
    "dbghelp.lib",
    "kernel32.lib",
    "msvcrt.lib",
    "ntdll.lib",
    "ucrt.lib",
    "userenv.lib",
    "vcruntime.lib",
    "ws2_32.lib"
)
$libraryDirectories = $env:LIB -split ";" | Where-Object { $_ -and (Test-Path -LiteralPath $_ -PathType Container) }
foreach ($library in $windowsLibraries) {
    $source = $libraryDirectories |
        ForEach-Object { Join-Path $_ $library } |
        Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } |
        Select-Object -First 1
    if (-not $source) {
        throw "Could not find required Windows import library $library in LIB."
    }
    Copy-Item -LiteralPath $source -Destination $libraryRoot
}
Copy-Item -LiteralPath (Join-Path $compilerRoot "THIRD_PARTY_LICENSES/NASM.txt") -Destination $packageRoot
Copy-Item -LiteralPath (Join-Path $compilerRoot "README.md") -Destination $packageRoot
@"
Adamantium portable for Windows x86-64

1. Extract the entire adamantium-windows-x86_64 directory.
2. Add that directory to your user PATH.
3. Open a new terminal and run: adamantium --version

Keep the tools directory next to adamantium.exe. Rust, Cargo, NASM, Visual Studio,
and the Windows SDK are not required on the computer using this package.
"@ | Set-Content -LiteralPath (Join-Path $packageRoot "INSTALL.txt") -Encoding UTF8

$smokeRoot = Join-Path $distributionRoot "portable-smoke-test"
if (Test-Path -LiteralPath $smokeRoot) {
    Remove-Item -LiteralPath $smokeRoot -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $smokeRoot | Out-Null
$savedPath = $env:PATH
$savedLib = $env:LIB
$savedLibPath = $env:LIBPATH
try {
    $env:PATH = "$packageRoot;$env:SystemRoot\System32"
    $env:LIB = ""
    $env:LIBPATH = ""
    Push-Location $smokeRoot
    try {
        & (Join-Path $packageRoot "adamantium.exe") new PortableSmoke
        if ($LASTEXITCODE -ne 0) { throw "Portable CLI project creation failed." }
        Push-Location (Join-Path $smokeRoot "PortableSmoke")
        try {
            & (Join-Path $packageRoot "adamantium.exe") build
            if ($LASTEXITCODE -ne 0) { throw "Portable CLI build failed without external tools." }
            & (Join-Path $smokeRoot "PortableSmoke/target/PortableSmoke.exe") | Out-Null
            if ($LASTEXITCODE -ne 0) { throw "Executable built by the portable CLI failed." }
        } finally {
            Pop-Location
        }
    } finally {
        Pop-Location
    }
} finally {
    $env:PATH = $savedPath
    $env:LIB = $savedLib
    $env:LIBPATH = $savedLibPath
    if (Test-Path -LiteralPath $smokeRoot) {
        Remove-Item -LiteralPath $smokeRoot -Recurse -Force
    }
}

$packageArchive = "$packageRoot.zip"
if (Test-Path -LiteralPath $packageArchive) {
    Remove-Item -LiteralPath $packageArchive -Force
}
Compress-Archive -LiteralPath $packageRoot -DestinationPath $packageArchive
$packageHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $packageArchive).Hash.ToLowerInvariant()
"$packageHash  adamantium-windows-x86_64.zip" |
    Set-Content -LiteralPath "$packageArchive.sha256" -Encoding ASCII
Write-Output "Created $packageArchive"
