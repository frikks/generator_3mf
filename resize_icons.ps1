$sourceIcon = "crates/app/res/ic_source.png"

if (-not (Test-Path $sourceIcon)) {
    Write-Error "Source icon not found: $sourceIcon. Place your 2048x2048 file as ic_source.png in res folder."
    exit 1
}

$sizes = @{
    "mipmap-mdpi"    = 48
    "mipmap-hdpi"    = 72
    "mipmap-xhdpi"   = 96
    "mipmap-xxhdpi"  = 144
    "mipmap-xxxhdpi" = 192
}

Add-Type -AssemblyName System.Drawing

foreach ($folder in $sizes.Keys) {
    $targetDir = "crates/app/res/$folder"
    if (-not (Test-Path $targetDir)) {
        New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
    }
    
    $targetSize = $sizes[$folder]
    $targetPath = Join-Path $targetDir "ic_launcher.png"
    
    Write-Host "Creating icon for $folder ($targetSize x $targetSize)..." -ForegroundColor Cyan
    
    $img = [System.Drawing.Image]::FromFile((Resolve-Path $sourceIcon))
    $bmp = New-Object System.Drawing.Bitmap $targetSize, $targetSize
    $graphics = [System.Drawing.Graphics]::FromImage($bmp)
    
    $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    
    $rect = New-Object System.Drawing.Rectangle 0, 0, $targetSize, $targetSize
    $graphics.DrawImage($img, $rect, 0, 0, $img.Width, $img.Height, [System.Drawing.GraphicsUnit]::Pixel)
    
    $bmp.Save($targetPath, [System.Drawing.Imaging.ImageFormat]::Png)
    
    $graphics.Dispose()
    $bmp.Dispose()
    $img.Dispose()
}

Write-Host "All icons successfully generated and sorted!" -ForegroundColor Green
