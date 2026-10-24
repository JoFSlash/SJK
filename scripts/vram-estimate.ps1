<#
.SYNOPSIS
Estimates how much video memory each map's textures take in SJK, from the pk3 files
alone (no game, no build, no graphics card needed).

.DESCRIPTION
Reads every .pk3 in the given folders (as the game does: in order, a later file
replacing an earlier one), each map's shader list from its .bsp, the .shader scripts
and the image headers (TGA, PNG, JPG). For each map it prints the memory its map
textures take uncompressed as SJK uploads them today (RGBA8 with a full mip chain) at
texture detail r_picmip 0, 1 and 2, what block compression (BC1 without alpha, BC3/BC7
with alpha) would take, and the extra material maps (_n, _s, _e ...) found beside
them. Then the largest textures of all the pk3 files.

An estimate of map textures only: not models placed on the map, players' skins,
effects, fonts, lightmaps, screen targets or lighting caches. Textures the map uses
that are in none of the given folders count as missing: pass retail GameData\base too.

.EXAMPLE
powershell -ExecutionPolicy Bypass -File vram-estimate.ps1 -Path "C:\Games\Jedi Academy\GameData\base","C:\SJK\jof\base"

.EXAMPLE
powershell -ExecutionPolicy Bypass -File vram-estimate.ps1 -Path "C:\SJK\jof\base" -Csv maps.csv
#>
param(
    # Folders holding .pk3 files, lowest priority first (retail base, then mods).
    [Parameter(Mandatory = $true)][string[]]$Path,
    # Also write the per-map table to this CSV file.
    [string]$Csv,
    # How many of the largest textures to list.
    [int]$Top = 20
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem

# As world_texture_mips.rs: r_picmip never shrinks a texture below 128 pixels.
$PicmipFloor = 128
$ImageExtensions = @('tga', 'png', 'jpg', 'jpeg')
# Material map suffixes SJK looks up beside a diffuse image (material_map_images.rs).
$MaterialSuffixes = @('_n', '_nh', '_s', '_specgloss', '_rmo', '_orm', '_e')
$Mib = 1MB

function Normalize([string]$name) {
    $name.Replace('\', '/').Trim('"').ToLowerInvariant()
}

function Strip-Extension([string]$name) {
    $dot = $name.LastIndexOf('.')
    $slash = $name.LastIndexOf('/')
    if ($dot -gt $slash) { return $name.Substring(0, $dot) }
    $name
}

# --- Index the pk3 files: the last file holding a path wins. ---
$archives = New-Object System.Collections.Generic.List[System.IO.Compression.ZipArchive]
$images = @{}   # "path/without/ext" -> @{ tga = entry; png = entry; ... }
$bsps = @{}     # "mp/ffa3" -> entry
$shaderTexts = New-Object System.Collections.Generic.List[object]

# Run with -File, PowerShell passes "a,b" as one string: split it here.
$Path = @($Path | ForEach-Object { $_ -split '[,;]' } | ForEach-Object { $_.Trim().Trim('"') } | Where-Object { $_ })
$pk3s = foreach ($folder in $Path) {
    if (-not (Test-Path -LiteralPath $folder)) { throw "Folder not found: $folder" }
    Get-ChildItem -LiteralPath $folder -Filter *.pk3 -File | Sort-Object Name
}
if (-not $pk3s) { throw "No .pk3 files in: $($Path -join ', ')" }

try {
    $done = 0
    foreach ($pk3 in $pk3s) {
        $done++
        Write-Progress -Activity 'Reading pk3 files' -Status $pk3.Name -PercentComplete (100 * $done / @($pk3s).Count)
        try {
            $zip = [System.IO.Compression.ZipFile]::OpenRead($pk3.FullName)
        } catch {
            Write-Warning "Skipped $($pk3.Name): $($_.Exception.Message)"
            continue
        }
        $archives.Add($zip)
        foreach ($entry in $zip.Entries) {
            if ($entry.Length -eq 0) { continue }
            $name = Normalize $entry.FullName
            $dot = $name.LastIndexOf('.')
            if ($dot -lt 0) { continue }
            $ext = $name.Substring($dot + 1)
            if ($ImageExtensions -contains $ext) {
                $key = $name.Substring(0, $dot)
                if (-not $images.ContainsKey($key)) { $images[$key] = @{} }
                $images[$key][$ext] = $entry
            } elseif ($ext -eq 'bsp' -and $name.StartsWith('maps/')) {
                $bsps[$name.Substring(5, $dot - 5)] = $entry
            } elseif ($ext -eq 'shader' -and $name.StartsWith('shaders/')) {
                $shaderTexts.Add($entry)
            }
        }
    }
    Write-Progress -Activity 'Reading pk3 files' -Completed

    function Read-Bytes($entry, [int]$limit) {
        $stream = $entry.Open()
        try {
            $size = [int][Math]::Min([int64]$limit, $entry.Length)
            $buffer = New-Object byte[] $size
            $read = 0
            while ($read -lt $size) {
                $n = $stream.Read($buffer, $read, $size - $read)
                if ($n -le 0) { break }
                $read += $n
            }
            if ($read -lt $size) { [Array]::Resize([ref]$buffer, $read) }
            , $buffer
        } finally {
            $stream.Dispose()
        }
    }

    # --- Shader scripts: which images each shader draws. ---
    $shaders = @{}   # shader name -> list of image names
    foreach ($entry in $shaderTexts) {
        $text = [System.Text.Encoding]::ASCII.GetString((Read-Bytes $entry ([int]::MaxValue)))
        $text = [regex]::Replace($text, '/\*.*?\*/', ' ', 'Singleline')
        $depth = 0
        $current = $null
        foreach ($line in $text -split "`n") {
            $comment = $line.IndexOf('//')
            if ($comment -ge 0) { $line = $line.Substring(0, $comment) }
            $tokens = @([regex]::Matches($line, '\{|\}|"[^"]*"|[^\s{}]+') | ForEach-Object { $_.Value })
            # The first word of a line is its keyword; braces anywhere on it still count.
            $lineDone = $false
            for ($i = 0; $i -lt $tokens.Count; $i++) {
                $token = $tokens[$i]
                if ($token -eq '{') { $depth++; continue }
                if ($token -eq '}') { $depth = [Math]::Max(0, $depth - 1); continue }
                if ($lineDone) { continue }
                $lineDone = $true
                if ($depth -eq 0) {
                    $current = New-Object System.Collections.Generic.List[string]
                    $shaders[(Strip-Extension (Normalize $token))] = $current
                    continue
                }
                if ($null -eq $current) { continue }
                $keyword = $token.ToLowerInvariant()
                if ($depth -eq 1 -and $keyword -eq 'skyparms' -and $i + 1 -lt $tokens.Count) {
                    $box = Normalize $tokens[$i + 1]
                    if ($box -ne '-') {
                        foreach ($side in 'rt', 'lf', 'bk', 'ft', 'up', 'dn') { $current.Add("${box}_$side") }
                    }
                } elseif ($depth -ge 2) {
                    if (($keyword -eq 'map' -or $keyword -eq 'clampmap') -and $i + 1 -lt $tokens.Count) {
                        $current.Add((Normalize $tokens[$i + 1]))
                    } elseif ($keyword -eq 'animmap' -or $keyword -eq 'clampanimmap' -or $keyword -eq 'oneshotanimmap') {
                        for ($j = $i + 2; $j -lt $tokens.Count -and $tokens[$j] -ne '}'; $j++) { $current.Add((Normalize $tokens[$j])) }
                    }
                }
            }
        }
    }

    # --- Image sizes, read from the headers. ---
    $sizes = @{}
    function Get-ImageInfo($entry) {
        if ($null -eq $entry) { return $null }
        $id = "$($entry.Archive.GetHashCode())|$($entry.FullName)"
        if ($sizes.ContainsKey($id)) { return $sizes[$id] }
        $info = $null
        $ext = [System.IO.Path]::GetExtension($entry.Name).ToLowerInvariant()
        try {
            if ($ext -eq '.tga') {
                $b = Read-Bytes $entry 18
                if ($b.Length -ge 18) {
                    $info = @{ W = [int]$b[12] -bor ([int]$b[13] -shl 8); H = [int]$b[14] -bor ([int]$b[15] -shl 8)
                        Alpha = ($b[16] -eq 32 -or ($b[17] -band 0x0F) -gt 0) }
                }
            } elseif ($ext -eq '.png') {
                $b = Read-Bytes $entry 26
                if ($b.Length -ge 26 -and $b[1] -eq 0x50) {
                    $info = @{ W = ([int]$b[16] -shl 24) -bor ([int]$b[17] -shl 16) -bor ([int]$b[18] -shl 8) -bor [int]$b[19]
                        H = ([int]$b[20] -shl 24) -bor ([int]$b[21] -shl 16) -bor ([int]$b[22] -shl 8) -bor [int]$b[23]
                        Alpha = ($b[25] -eq 4 -or $b[25] -eq 6 -or $b[25] -eq 3) }
                }
            } else {
                $b = Read-Bytes $entry 262144
                $i = 2
                while ($i + 9 -lt $b.Length) {
                    if ($b[$i] -ne 0xFF) { $i++; continue }
                    $marker = [int]$b[$i + 1]
                    if ($marker -eq 0xFF) { $i++; continue }
                    if ($marker -eq 0x01 -or $marker -eq 0xD8 -or ($marker -ge 0xD0 -and $marker -le 0xD7)) { $i += 2; continue }
                    if ($marker -ge 0xC0 -and $marker -le 0xCF -and $marker -ne 0xC4 -and $marker -ne 0xC8 -and $marker -ne 0xCC) {
                        $info = @{ H = ([int]$b[$i + 5] -shl 8) -bor [int]$b[$i + 6]; W = ([int]$b[$i + 7] -shl 8) -bor [int]$b[$i + 8]; Alpha = $false }
                        break
                    }
                    $i += 2 + (([int]$b[$i + 2] -shl 8) -bor [int]$b[$i + 3])
                }
            }
        } catch {
            Write-Warning "Could not read $($entry.FullName): $($_.Exception.Message)"
        }
        if ($info -and ($info.W -le 0 -or $info.H -le 0)) { $info = $null }
        $sizes[$id] = $info
        $info
    }

    function Find-Image([string]$name) {
        $found = $images[(Strip-Extension $name)]
        if (-not $found) { return $null }
        foreach ($ext in $ImageExtensions) { if ($found.ContainsKey($ext)) { return $found[$ext] } }
        $null
    }

    # Bytes on the card: RGBA8 with every mip level, after r_picmip leaves levels off.
    function Get-Bytes([int]$w, [int]$h, [int]$picmip) {
        $skip = 0
        while ($skip -lt $picmip -and ([Math]::Max($w, $h) -shr ($skip + 1)) -ge $PicmipFloor) { $skip++ }
        $w = [Math]::Max(1, $w -shr $skip); $h = [Math]::Max(1, $h -shr $skip)
        [int64]$total = 0
        while ($true) {
            $total += [int64]$w * $h * 4
            if ($w -eq 1 -and $h -eq 1) { break }
            $w = [Math]::Max(1, $w -shr 1); $h = [Math]::Max(1, $h -shr 1)
        }
        $total
    }

    # --- Each map. ---
    $rows = New-Object System.Collections.Generic.List[object]
    $done = 0
    foreach ($map in ($bsps.Keys | Sort-Object)) {
        $done++
        Write-Progress -Activity 'Measuring maps' -Status $map -PercentComplete (100 * $done / $bsps.Count)
        $entry = $bsps[$map]
        $bsp = Read-Bytes $entry ([int]::MaxValue)
        $ident = [System.Text.Encoding]::ASCII.GetString($bsp, 0, 4)
        if ($ident -ne 'RBSP' -and $ident -ne 'IBSP') { Write-Warning "$map is not a Quake 3 style BSP ($ident)"; continue }
        $offset = [BitConverter]::ToInt32($bsp, 8 + 8)
        $length = [BitConverter]::ToInt32($bsp, 8 + 12)
        $used = New-Object 'System.Collections.Generic.HashSet[string]'
        for ($at = $offset; $at + 72 -le $offset + $length; $at += 72) {
            $name = [System.Text.Encoding]::ASCII.GetString($bsp, $at, 64).Split([char]0)[0]
            $name = Strip-Extension (Normalize $name)
            if (-not $name -or $name -eq 'noshader') { continue }
            if ($shaders.ContainsKey($name)) {
                foreach ($image in $shaders[$name]) {
                    if (-not $image.StartsWith('$') -and -not $image.StartsWith('*')) { [void]$used.Add((Strip-Extension $image)) }
                }
            } else {
                [void]$used.Add($name)
            }
        }
        $sum = @(0, 0, 0); [int64]$compressed = 0; [int64]$material = 0
        $missing = 0; $count = 0; $biggest = $null; [int64]$biggestBytes = 0
        foreach ($image in $used) {
            $file = Find-Image $image
            if (-not $file) { $missing++; continue }
            $info = Get-ImageInfo $file
            if (-not $info) { $missing++; continue }
            $count++
            for ($p = 0; $p -lt 3; $p++) { $sum[$p] += Get-Bytes $info.W $info.H $p }
            $full = Get-Bytes $info.W $info.H 0
            if ($info.Alpha) { $compressed += $full / 4 } else { $compressed += $full / 8 }
            if ($full -gt $biggestBytes) { $biggestBytes = $full; $biggest = "$image ($($info.W)x$($info.H))" }
            foreach ($suffix in $MaterialSuffixes) {
                $extra = Find-Image "$image$suffix"
                if ($extra) {
                    $extraInfo = Get-ImageInfo $extra
                    if ($extraInfo) { $material += Get-Bytes $extraInfo.W $extraInfo.H 0 }
                }
            }
        }
        $rows.Add([pscustomobject]@{
                Map            = $map
                Textures       = $count
                Missing        = $missing
                'MiB picmip0'  = [Math]::Round($sum[0] / $Mib)
                'MiB picmip1'  = [Math]::Round($sum[1] / $Mib)
                'MiB picmip2'  = [Math]::Round($sum[2] / $Mib)
                'MiB BC'       = [Math]::Round($compressed / $Mib)
                'MiB material' = [Math]::Round($material / $Mib)
                Biggest        = $biggest
            })
    }
    Write-Progress -Activity 'Measuring maps' -Completed

    $sorted = $rows | Sort-Object 'MiB picmip0' -Descending | Select-Object Map, Textures, Missing, 'MiB picmip0', 'MiB picmip1', 'MiB picmip2', 'MiB BC', 'MiB material', Biggest
    Write-Host ''
    Write-Host 'Map textures on the card (RGBA8 with mips, as SJK uploads them today):' -ForegroundColor Cyan
    $sorted | Format-Table -AutoSize | Out-String -Width 400 | Write-Host
    if ($Csv) { $sorted | Export-Csv -LiteralPath $Csv -NoTypeInformation -Encoding UTF8; Write-Host "Written: $Csv" }

    # --- The largest textures of all the pk3 files. ---
    $all = foreach ($key in $images.Keys) {
        # The key is already without its extension: a name with another dot in it
        # ("skin.v2") must not lose that part too.
        $file = $null
        foreach ($ext in $ImageExtensions) { if (-not $file -and $images[$key].ContainsKey($ext)) { $file = $images[$key][$ext] } }
        if (-not $file) { continue }
        $info = Get-ImageInfo $file
        if ($info) {
            [pscustomobject]@{ Texture = $key; Size = "$($info.W)x$($info.H)"; MiB = [Math]::Round((Get-Bytes $info.W $info.H 0) / $Mib, 1); Alpha = $info.Alpha }
        }
    }
    Write-Host "The $Top largest textures (any kind: maps, models, skins, effects):" -ForegroundColor Cyan
    $all | Sort-Object MiB -Descending | Select-Object -First $Top | Format-Table -AutoSize | Out-String -Width 400 | Write-Host
    $total = ($all | Measure-Object MiB -Sum).Sum
    Write-Host ("All {0} textures in these folders: {1:N0} MiB if every one were on the card at once." -f @($all).Count, $total)
    Write-Host 'MiB BC = with block compression (BC1 opaque, BC3/BC7 with alpha); SJK does not compress yet.'
    Write-Host 'Missing = images the map names that are in none of the folders (often retail base: pass it too).'
} finally {
    foreach ($zip in $archives) { $zip.Dispose() }
}
