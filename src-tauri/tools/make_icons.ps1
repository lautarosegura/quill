# make_icons.ps1 — generates all Quill icon assets from a single vector drawing routine.
# Uses .NET System.Drawing (built-in on Windows). No external deps.
#
# Outputs:
#   - icon.png (512x512) + all size variants + Windows Store squares
#   - icon.ico (multi-res PNG-embedded)
#   - tray-idle.png / tray-recording.png / tray-error.png (32x32 state icons)
#
# Run:  powershell -ExecutionPolicy Bypass -File src-tauri/tools/make_icons.ps1

Add-Type -AssemblyName System.Drawing

$ErrorActionPreference = 'Stop'

$IconsDir = Join-Path $PSScriptRoot '..\icons'
$IconsDir = [System.IO.Path]::GetFullPath($IconsDir)
if (-not (Test-Path $IconsDir)) { New-Item -ItemType Directory -Path $IconsDir | Out-Null }

# Brand color: accent purple oklch(0.72 0.22 295) ~ #9B7EDE
$Accent = [System.Drawing.Color]::FromArgb(255, 0x9B, 0x7E, 0xDE)
$Gray   = [System.Drawing.Color]::FromArgb(255, 0x88, 0x88, 0x88)
$Red    = [System.Drawing.Color]::FromArgb(255, 0xE0, 0x50, 0x4C)
$Amber  = [System.Drawing.Color]::FromArgb(255, 0xE0, 0xA0, 0x4C)

# Draw a stylized quill/feather into the given Graphics at the requested size.
# The glyph is designed on a 512x512 canvas and scaled via transforms.
function Draw-Quill {
    param(
        [System.Drawing.Graphics]$g,
        [int]$size,
        [System.Drawing.Color]$color
    )

    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality

    # Scale from 512-unit design space to $size pixels.
    $scale = $size / 512.0
    $g.ScaleTransform($scale, $scale)

    # Feather silhouette — a leaf-like blade tapering to a nib at bottom-left.
    # Points form a smooth bezier along the right edge and a straight spine on the left.
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath

    # Spine tip (top-right) -> bulge (right side) -> nib point (bottom-left)
    # then a short quill shaft and back up via the left spine.
    $p1 = New-Object System.Drawing.PointF(400, 60)      # tip of feather
    $c1 = New-Object System.Drawing.PointF(510, 200)     # outer bulge control
    $c2 = New-Object System.Drawing.PointF(470, 360)     # lower curve control
    $p2 = New-Object System.Drawing.PointF(200, 430)     # base of blade (right of nib)
    $p3 = New-Object System.Drawing.PointF(110, 470)     # nib point
    $p4 = New-Object System.Drawing.PointF(150, 440)     # shaft start
    $c3 = New-Object System.Drawing.PointF(260, 260)     # inner spine control
    $c4 = New-Object System.Drawing.PointF(340, 140)     # upper spine control

    $path.AddBezier($p1, $c1, $c2, $p2)
    $path.AddLine($p2, $p3)
    $path.AddLine($p3, $p4)
    $path.AddBezier($p4, $c3, $c4, $p1)
    $path.CloseFigure()

    $brush = New-Object System.Drawing.SolidBrush($color)
    $g.FillPath($brush, $path)

    # Center vein — a slightly lighter line down the spine for depth.
    $veinColor = [System.Drawing.Color]::FromArgb(180,
        [math]::Min(255, $color.R + 30),
        [math]::Min(255, $color.G + 30),
        [math]::Min(255, $color.B + 30))
    $pen = New-Object System.Drawing.Pen($veinColor, 8.0)
    $pen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $pen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $g.DrawBezier($pen, $p1, $c4, $c3, $p4)

    $pen.Dispose()
    $brush.Dispose()
    $path.Dispose()

    $g.ResetTransform()
}

# Create a PNG as a byte array at the given size with the given fill color.
function New-QuillPngBytes {
    param(
        [int]$size,
        [System.Drawing.Color]$color
    )
    $bmp = New-Object System.Drawing.Bitmap($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.Clear([System.Drawing.Color]::Transparent)
    Draw-Quill -g $g -size $size -color $color
    $g.Dispose()

    $ms = New-Object System.IO.MemoryStream
    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    $bytes = $ms.ToArray()
    $ms.Dispose()
    return ,$bytes
}

function Save-QuillPng {
    param(
        [int]$size,
        [System.Drawing.Color]$color,
        [string]$path
    )
    $bytes = New-QuillPngBytes -size $size -color $color
    [System.IO.File]::WriteAllBytes($path, $bytes)
    Write-Host "wrote $path ($($bytes.Length) bytes)"
}

# Build a Windows ICO file containing multiple PNG-compressed images.
# ICO format:
#   ICONDIR    (6 bytes):   reserved(2)=0, type(2)=1, count(2)=N
#   ICONDIRENTRY * N (16 bytes each):
#       width(1), height(1) — 0 means 256
#       colors(1)=0, reserved(1)=0
#       planes(2)=1, bitCount(2)=32
#       bytesInRes(4), imageOffset(4)
#   then each PNG blob concatenated.
function Save-MultiIco {
    param(
        [int[]]$sizes,
        [System.Drawing.Color]$color,
        [string]$path
    )

    $pngs = @()
    foreach ($s in $sizes) {
        $pngs += ,(New-QuillPngBytes -size $s -color $color)
    }

    $count = $sizes.Length
    $headerSize = 6 + (16 * $count)
    $totalSize = $headerSize
    foreach ($p in $pngs) { $totalSize += $p.Length }

    $buf = New-Object byte[] $totalSize
    # ICONDIR
    $buf[0] = 0; $buf[1] = 0            # reserved
    $buf[2] = 1; $buf[3] = 0            # type = 1 (ICO)
    $buf[4] = [byte]($count -band 0xFF); $buf[5] = [byte](($count -shr 8) -band 0xFF)

    $offset = $headerSize
    for ($i = 0; $i -lt $count; $i++) {
        $s = $sizes[$i]
        $png = $pngs[$i]
        $entryOffset = 6 + ($i * 16)

        $wByte = if ($s -ge 256) { 0 } else { [byte]$s }
        $hByte = if ($s -ge 256) { 0 } else { [byte]$s }
        $buf[$entryOffset + 0] = $wByte
        $buf[$entryOffset + 1] = $hByte
        $buf[$entryOffset + 2] = 0       # colors in palette
        $buf[$entryOffset + 3] = 0       # reserved
        $buf[$entryOffset + 4] = 1       # planes lo
        $buf[$entryOffset + 5] = 0       # planes hi
        $buf[$entryOffset + 6] = 32      # bit count lo
        $buf[$entryOffset + 7] = 0       # bit count hi

        # bytesInRes (little-endian u32)
        $len = $png.Length
        $buf[$entryOffset + 8]  = [byte]($len -band 0xFF)
        $buf[$entryOffset + 9]  = [byte](($len -shr 8) -band 0xFF)
        $buf[$entryOffset + 10] = [byte](($len -shr 16) -band 0xFF)
        $buf[$entryOffset + 11] = [byte](($len -shr 24) -band 0xFF)
        # imageOffset
        $buf[$entryOffset + 12] = [byte]($offset -band 0xFF)
        $buf[$entryOffset + 13] = [byte](($offset -shr 8) -band 0xFF)
        $buf[$entryOffset + 14] = [byte](($offset -shr 16) -band 0xFF)
        $buf[$entryOffset + 15] = [byte](($offset -shr 24) -band 0xFF)

        [Array]::Copy($png, 0, $buf, $offset, $png.Length)
        $offset += $png.Length
    }

    [System.IO.File]::WriteAllBytes($path, $buf)
    Write-Host "wrote $path ($($buf.Length) bytes, $count sizes)"
}

# ---- Main app icon (purple) and all size variants ----
Save-QuillPng -size 512 -color $Accent -path (Join-Path $IconsDir 'icon.png')
Save-QuillPng -size 32  -color $Accent -path (Join-Path $IconsDir '32x32.png')
Save-QuillPng -size 128 -color $Accent -path (Join-Path $IconsDir '128x128.png')
Save-QuillPng -size 256 -color $Accent -path (Join-Path $IconsDir '128x128@2x.png')

# Windows Store square logos (same glyph, different sizes)
Save-QuillPng -size 30  -color $Accent -path (Join-Path $IconsDir 'Square30x30Logo.png')
Save-QuillPng -size 44  -color $Accent -path (Join-Path $IconsDir 'Square44x44Logo.png')
Save-QuillPng -size 71  -color $Accent -path (Join-Path $IconsDir 'Square71x71Logo.png')
Save-QuillPng -size 89  -color $Accent -path (Join-Path $IconsDir 'Square89x89Logo.png')
Save-QuillPng -size 107 -color $Accent -path (Join-Path $IconsDir 'Square107x107Logo.png')
Save-QuillPng -size 142 -color $Accent -path (Join-Path $IconsDir 'Square142x142Logo.png')
Save-QuillPng -size 150 -color $Accent -path (Join-Path $IconsDir 'Square150x150Logo.png')
Save-QuillPng -size 284 -color $Accent -path (Join-Path $IconsDir 'Square284x284Logo.png')
Save-QuillPng -size 310 -color $Accent -path (Join-Path $IconsDir 'Square310x310Logo.png')
Save-QuillPng -size 50  -color $Accent -path (Join-Path $IconsDir 'StoreLogo.png')

# ICO: multi-size PNG-embedded. Tauri/Windows accept 16/32/48/64/128/256.
Save-MultiIco -sizes @(16, 32, 48, 64, 128, 256) -color $Accent -path (Join-Path $IconsDir 'icon.ico')

# ---- Tray state icons (32x32) ----
Save-QuillPng -size 32 -color $Gray  -path (Join-Path $IconsDir 'tray-idle.png')
Save-QuillPng -size 32 -color $Red   -path (Join-Path $IconsDir 'tray-recording.png')
Save-QuillPng -size 32 -color $Amber -path (Join-Path $IconsDir 'tray-error.png')

# Also dump raw RGBA buffers for the tray icons so Rust can embed them via
# `Image::new_owned` without pulling in the `image-png` feature on `tauri`.
# Format: 32*32*4 = 4096 bytes, row-major, RGBA (non-premultiplied).
function Save-QuillRgba {
    param(
        [int]$size,
        [System.Drawing.Color]$color,
        [string]$path
    )
    $bmp = New-Object System.Drawing.Bitmap($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.Clear([System.Drawing.Color]::Transparent)
    Draw-Quill -g $g -size $size -color $color
    $g.Dispose()

    $rect = New-Object System.Drawing.Rectangle(0, 0, $size, $size)
    $data = $bmp.LockBits($rect, [System.Drawing.Imaging.ImageLockMode]::ReadOnly, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $stride = $data.Stride
    $buf = New-Object byte[] ($stride * $size)
    [System.Runtime.InteropServices.Marshal]::Copy($data.Scan0, $buf, 0, $buf.Length)
    $bmp.UnlockBits($data)
    $bmp.Dispose()

    # GDI+ Format32bppArgb is stored as BGRA little-endian. Swap to RGBA.
    $out = New-Object byte[] ($size * $size * 4)
    for ($y = 0; $y -lt $size; $y++) {
        for ($x = 0; $x -lt $size; $x++) {
            $src = ($y * $stride) + ($x * 4)
            $dst = (($y * $size) + $x) * 4
            $out[$dst + 0] = $buf[$src + 2]   # R <- src B (actually BGRA stored means [B,G,R,A])
            $out[$dst + 1] = $buf[$src + 1]   # G
            $out[$dst + 2] = $buf[$src + 0]   # B <- src R
            $out[$dst + 3] = $buf[$src + 3]   # A
        }
    }
    [System.IO.File]::WriteAllBytes($path, $out)
    Write-Host "wrote $path ($($out.Length) bytes RGBA, ${size}x${size})"
}

Save-QuillRgba -size 32 -color $Gray  -path (Join-Path $IconsDir 'tray-idle.rgba')
Save-QuillRgba -size 32 -color $Red   -path (Join-Path $IconsDir 'tray-recording.rgba')
Save-QuillRgba -size 32 -color $Amber -path (Join-Path $IconsDir 'tray-error.rgba')

Write-Host ""
Write-Host "Done. Icons written to $IconsDir"
