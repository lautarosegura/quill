# Downloads pre-built whisper-cli binary for Windows and places it into
# src-tauri/binaries/ along with its DLL dependencies.
#
# Tauri bundles `whisper-cli-<triple>.exe` as a sidecar (via `externalBin` in
# tauri.conf.json). The DLLs are needed at runtime and are copied alongside
# so Windows can resolve them.

$ErrorActionPreference = "Stop"

$WhisperVersion = "v1.7.6"
$Repo = "ggerganov/whisper.cpp"
$BinDir = Join-Path $PSScriptRoot "..\src-tauri\binaries"
New-Item -ItemType Directory -Path $BinDir -Force | Out-Null

Write-Host "Downloading whisper.cpp $WhisperVersion prebuilts..."

$Asset = "whisper-bin-x64.zip"
$TargetTriple = "x86_64-pc-windows-msvc"
$Url = "https://github.com/$Repo/releases/download/$WhisperVersion/$Asset"

$Tmp = New-Item -ItemType Directory -Path (Join-Path $env:TEMP "quill-whisper-$([guid]::NewGuid())") -Force
$ZipPath = Join-Path $Tmp $Asset

Write-Host "Fetching $Url"
Invoke-WebRequest -Uri $Url -OutFile $ZipPath
Expand-Archive -Path $ZipPath -DestinationPath $Tmp -Force

# Find whisper-cli.exe (the CLI transcription tool — NOT main.exe which is a stub).
$ReleaseDir = Join-Path $Tmp "Release"
$Src = Join-Path $ReleaseDir "whisper-cli.exe"
if (-not (Test-Path $Src)) {
	Write-Error "whisper-cli.exe not found at $Src. Archive contents:"
	Get-ChildItem -Recurse $Tmp
	exit 1
}

# Main binary renamed with Tauri's target-triple suffix.
$Dest = Join-Path $BinDir "whisper-cli-$TargetTriple.exe"
Copy-Item -Path $Src -Destination $Dest -Force
Write-Host "Installed $Dest"

# Copy every DLL next to the binary. Windows resolves DLLs from the exe's
# directory first, so placing them here makes the sidecar self-contained.
Get-ChildItem -Path $ReleaseDir -Filter "*.dll" | ForEach-Object {
	$dllDest = Join-Path $BinDir $_.Name
	Copy-Item -Path $_.FullName -Destination $dllDest -Force
	Write-Host "Installed $dllDest"
}

# VAD model — Silero v6.2.0 from ggml-org's HuggingFace repo. Bundled
# alongside whisper-cli so users get hallucination-free transcription
# (no trailing-silence "you" / "thanks for watching") out of the box.
# ~2 MB, single static download.
$VadModelUrl = "https://huggingface.co/ggml-org/whisper-vad/resolve/main/ggml-silero-v6.2.0.bin"
$VadDest = Join-Path $BinDir "ggml-silero-v6.2.0.bin"
Write-Host "Downloading Silero VAD model..."
Invoke-WebRequest -Uri $VadModelUrl -OutFile $VadDest
Write-Host "Installed $VadDest"

Remove-Item -Recurse -Force $Tmp
Write-Host "Done."
