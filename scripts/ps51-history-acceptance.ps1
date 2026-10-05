# Verify the mtty PowerShell 5.1 history record on a real Windows desktop
#
# Run this from a normal, logged-on Windows desktop session (not over ssh):
# it launches the mtty app in an isolated state, types two commands into a
# Windows PowerShell 5.1 pane through the shell integration, and reads the
# recorded history back over MTP. It prints PASS/FAIL and where the evidence is.
#
#   powershell -NoProfile -ExecutionPolicy Bypass -File ps51-history-acceptance.ps1 -App <path to mtty.exe>
#
# Get the app from the v0.1.6 release zip (mtty.exe, mtty-cli.exe, mtty-ptyhost.exe).

param(
    [Parameter(Mandatory = $true)] [string] $App,
    [string] $WorkDir = "$env:USERPROFILE\mtty-ps51-acceptance"
)

$ErrorActionPreference = 'Stop'
$app = (Resolve-Path $App).Path
$root = Split-Path $app -Parent
$cli = Join-Path $root 'mtty-cli.exe'
if (-not (Test-Path $cli)) { throw "mtty-cli.exe not found next to $app" }

# 0xdtee: this script must run in a logged-on session. Interactive = $true.
if (-not [Environment]::UserInteractive) {
    throw "Not an interactive session. Run this from the Windows desktop (RDP or the console), not over ssh."
}

# Use Windows PowerShell 5.1 as the pane shell, so MTTY's 5.1 history hook is exercised.
$ps51 = "$env:WINDIR\System32\WindowsPowerShell\v1.0\powershell.exe"
if (-not (Test-Path $ps51)) { throw "Windows PowerShell 5.1 not found at $ps51" }
$ver = & $ps51 -NoProfile -Command '$PSVersionTable.PSVersion.ToString()'

New-Item -ItemType Directory -Force -Path $WorkDir | Out-Null
$base = Join-Path $WorkDir 'run'
Remove-Item -Recurse -Force $base -ErrorAction SilentlyContinue
foreach ($d in 'home','config','data','runtime','tmp') {
    New-Item -ItemType Directory -Force -Path (Join-Path $base $d) | Out-Null
}

# Isolated state so this never touches the real config; Windows PowerShell 5.1 pane.
$env:HOME = Join-Path $base 'home'
$env:XDG_CONFIG_HOME = Join-Path $base 'config'
$env:XDG_DATA_HOME = Join-Path $base 'data'
$env:XDG_RUNTIME_DIR = Join-Path $base 'runtime'
$env:TMP = Join-Path $base 'tmp'
$env:TEMP = Join-Path $base 'tmp'
$env:SHELL = $ps51

$sock = Join-Path $base 'runtime\mtty.sock'
Write-Host "PowerShell 5.1: $ver"
Write-Host "Launching mtty in an isolated state..."

$p = Start-Process -FilePath $app -PassThru
try {
    function Cli { param([string[]] $a) & $cli --socket $sock --wait 20 @a 2>&1 | Out-String }

    # Wait for the pane to exist.
    $ready = $false
    for ($i = 0; $i -lt 40; $i++) {
        if ((Cli @('pane','list')) -match 'pane0') { $ready = $true; break }
        Start-Sleep -Milliseconds 500
    }
    if (-not $ready) { throw "mtty never exposed pane0" }

    # Type two commands into the 5.1 pane. `pane run` writes them as keystrokes,
    # so the PSReadLine / PSConsoleHostReadLine hook records each one.
    Cli @('pane','run','--pane','pane0','--data',"echo alpha-$([guid]::NewGuid().ToString('N').Substring(0,8))`r") | Out-Null
    Start-Sleep -Seconds 2
    $mark = "echo beta-$([guid]::NewGuid().ToString('N').Substring(0,8))"
    Cli @('pane','run','--pane','pane0','--data',"$mark`r") | Out-Null
    Start-Sleep -Seconds 2

    $hist = Cli @('history','list')
    $hist | Out-File (Join-Path $base 'history.json') -Encoding utf8
    $out = Cli @('pane','output','--pane','pane0')
    $out | Out-File (Join-Path $base 'output.txt') -Encoding utf8

    $recorded = $hist -match [regex]::Escape($mark)
    $result = if ($recorded) { 'PASS' } else { 'FAIL' }
    Write-Host ""
    Write-Host "=== $result ===" -ForegroundColor ($(if ($recorded) {'Green'} else {'Red'}))
    Write-Host "Recorded marker '$mark': $recorded"
    Write-Host "history.json: $(Join-Path $base 'history.json')"
    Write-Host "output.txt:   $(Join-Path $base 'output.txt')"
    if (-not $recorded) {
        Write-Host "history list returned:"
        Write-Host $hist
    }
    exit $(if ($recorded) { 0 } else { 1 })
}
finally {
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
}
