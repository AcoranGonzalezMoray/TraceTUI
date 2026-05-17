param (
    [Parameter(Mandatory=$true)]
    [string]$ExePath,
    
    [Parameter(Mandatory=$false)]
    [int]$Width = 12
)

function Show-ExeIcon {
    param (
        [string]$ExePath,
        [int]$Width
    )

    if (-Not (Test-Path $ExePath)) { return }

    Add-Type -AssemblyName System.Drawing

    try {
        $icon = [System.Drawing.Icon]::ExtractAssociatedIcon($ExePath)
        $originalBmp = $icon.ToBitmap()
        
        # --- LÓGICA DE REDIMENSIONAMIENTO ---
        $aspectRatio = $originalBmp.Height / $originalBmp.Width
        $newHeight = [int]($Width * $aspectRatio)
        
        $bmp = New-Object System.Drawing.Bitmap($Width, $newHeight)
        $graph = [System.Drawing.Graphics]::FromImage($bmp)
        
        $graph.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $graph.DrawImage($originalBmp, 0, 0, $Width, $newHeight)
        # ------------------------------------

        $blockChar = [char]0x2580
        $esc = [char]27

        for ($y = 0; $y -lt $bmp.Height; $y += 2) {
            $line = ""
            for ($x = 0; $x -lt $bmp.Width; $x++) {
                $topPixel = $bmp.GetPixel($x, $y)
                $bottomPixel = if ($y + 1 -lt $bmp.Height) { $bmp.GetPixel($x, $y + 1) } else { [System.Drawing.Color]::Transparent }

                if ($topPixel.A -eq 0 -and $bottomPixel.A -eq 0) {
                    $line += " "
                } else {
                    $fg = if ($topPixel.A -gt 0) { "$esc[38;2;$($topPixel.R);$($topPixel.G);$($topPixel.B)m" } else { "$esc[39m" }
                    $bg = if ($bottomPixel.A -gt 0) { "$esc[48;2;$($bottomPixel.R);$($bottomPixel.G);$($bottomPixel.B)m" } else { "$esc[49m" }
                    
                    if ($topPixel.A -eq 0) { $line += "${fg}${bg} " } else { $line += "${fg}${bg}$blockChar" }
                }
            }
            Write-Host "$line$esc[0m"
        }
    }
    finally {
        if ($graph) { $graph.Dispose() }
        if ($bmp) { $bmp.Dispose() }
        if ($originalBmp) { $originalBmp.Dispose() }
        if ($icon) { $icon.Dispose() }
    }
}

Show-ExeIcon -ExePath $ExePath -Width $Width