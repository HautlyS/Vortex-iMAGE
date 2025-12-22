# Vortex Build System - Interactive TUI
$ErrorActionPreference = "Stop"
Set-Location "$PSScriptRoot\.."
$Host.UI.RawUI.WindowTitle = "VORTEX BUILD SYSTEM"

# Colors
$script:Cyan = "`e[36m"
$script:Magenta = "`e[35m"
$script:Yellow = "`e[33m"
$script:Green = "`e[32m"
$script:Red = "`e[31m"
$script:White = "`e[97m"
$script:Gray = "`e[90m"
$script:Reset = "`e[0m"
$script:Bold = "`e[1m"

function Show-Banner {
    Clear-Host
    $banner = @"
$Magenta
    ╔═══════════════════════════════════════════════════════╗
    ║                                                       ║
    ║  $Cyan██╗   ██╗ ██████╗ ██████╗ ████████╗███████╗██╗  ██╗$Magenta  ║
    ║  $Cyan██║   ██║██╔═══██╗██╔══██╗╚══██╔══╝██╔════╝╚██╗██╔╝$Magenta  ║
    ║  $Cyan██║   ██║██║   ██║██████╔╝   ██║   █████╗   ╚███╔╝$Magenta   ║
    ║  $Cyan╚██╗ ██╔╝██║   ██║██╔══██╗   ██║   ██╔══╝   ██╔██╗$Magenta   ║
    ║  $Cyan ╚████╔╝ ╚██████╔╝██║  ██║   ██║   ███████╗██╔╝ ██╗$Magenta  ║
    ║  $Cyan  ╚═══╝   ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ╚══════╝╚═╝  ╚═╝$Magenta  ║
    ║                                                       ║
    ╠═══════════════════════════════════════════════════════╣
    ║       $White「 I N T E R A C T I V E   B U I L D E R 」$Magenta       ║
    ╚═══════════════════════════════════════════════════════╝
$Reset
"@
    Write-Host $banner
    Write-Host "  $Gray⟨ $(Get-Date -Format 'yyyy.MM.dd HH:mm') ⟩ System Online ⟨ Windows ⟩$Reset`n"
}

function Show-Spinner {
    param([string]$Text, [scriptblock]$Action)
    $frames = @('⠋','⠙','⠹','⠸','⠼','⠴','⠦','⠧','⠇','⠏')
    $job = Start-Job -ScriptBlock $Action
    $i = 0
    while ($job.State -eq 'Running') {
        Write-Host "`r  $Cyan$($frames[$i % 10])$Reset $Text" -NoNewline
        Start-Sleep -Milliseconds 80
        $i++
    }
    Write-Host "`r  $Green✓$Reset $Text    "
    Receive-Job $job -Wait
    Remove-Job $job
}

function Show-CheckboxMenu {
    param([string[]]$Options, [string]$Title)
    
    $selected = @{}
    $Options | ForEach-Object { $selected[$_] = $false }
    $currentIndex = 0
    
    $cursorVisible = [Console]::CursorVisible
    [Console]::CursorVisible = $false
    
    try {
        while ($true) {
            $startY = [Console]::CursorTop
            
            Write-Host "`n  $Magenta┌─────────────────────────────────────────┐$Reset"
            Write-Host "  $Magenta│$White  $Title$Magenta$((' ' * (38 - $Title.Length)))│$Reset"
            Write-Host "  $Magenta├─────────────────────────────────────────┤$Reset"
            Write-Host "  $Magenta│$Gray  [Space] Toggle  [Enter] Confirm$Magenta       │$Reset"
            Write-Host "  $Magenta└─────────────────────────────────────────┘$Reset`n"
            
            for ($i = 0; $i -lt $Options.Count; $i++) {
                $opt = $Options[$i]
                $check = if ($selected[$opt]) { "$Green◉$Reset" } else { "$Gray○$Reset" }
                $cursor = if ($i -eq $currentIndex) { "$Cyan▸$Reset" } else { " " }
                $color = if ($i -eq $currentIndex) { $White } else { $Gray }
                Write-Host "   $cursor $check $color$opt$Reset"
            }
            
            Write-Host ""
            
            $key = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
            
            switch ($key.VirtualKeyCode) {
                38 { $currentIndex = [Math]::Max(0, $currentIndex - 1) }  # Up
                40 { $currentIndex = [Math]::Min($Options.Count - 1, $currentIndex + 1) }  # Down
                32 { $selected[$Options[$currentIndex]] = -not $selected[$Options[$currentIndex]] }  # Space
                13 { 
                    return $Options | Where-Object { $selected[$_] }
                }  # Enter
                27 { return @() }  # Escape
            }
            
            # Clear menu for redraw
            [Console]::SetCursorPosition(0, $startY)
            for ($i = 0; $i -lt ($Options.Count + 8); $i++) {
                Write-Host (' ' * 50)
            }
            [Console]::SetCursorPosition(0, $startY)
        }
    } finally {
        [Console]::CursorVisible = $cursorVisible
    }
}

function Show-Confirm {
    param([string]$Message)
    Write-Host "`n  $Yellow$Message$Reset [Y/n] " -NoNewline
    $key = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    Write-Host ""
    return $key.Character -ne 'n' -and $key.Character -ne 'N'
}

function Test-Dependencies {
    $missing = @()
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { $missing += "cargo" }
    if (-not (Get-Command node -ErrorAction SilentlyContinue)) { $missing += "node" }
    
    if ($missing.Count -gt 0) {
        Write-Host "  $Red✗ Missing: $($missing -join ', ')$Reset"
        exit 1
    }
    
    try { cargo tauri --version 2>$null | Out-Null }
    catch {
        Write-Host "  $Yellow⚠ Installing tauri-cli...$Reset"
        cargo install tauri-cli
    }
    
    $script:PkgMgr = "npm"
    if (Test-Path "pnpm-lock.yaml") { $script:PkgMgr = "pnpm" }
    elseif (Test-Path "yarn.lock") { $script:PkgMgr = "yarn" }
    
    if (-not (Test-Path "node_modules")) {
        Write-Host "  $Cyan⟳$Reset Installing dependencies..."
        & $script:PkgMgr install | Out-Null
    }
    
    Write-Host "  $Green✓$Reset Dependencies OK"
}

function Initialize-Android {
    Write-Host "`n  $Cyan⟳$Reset Setting up Android..."
    
    if (-not $env:ANDROID_HOME) {
        @("$env:LOCALAPPDATA\Android\Sdk", "C:\Android\Sdk") | ForEach-Object {
            if (Test-Path $_) { $env:ANDROID_HOME = $_ }
        }
    }
    
    if (-not $env:ANDROID_HOME -or -not (Test-Path $env:ANDROID_HOME)) {
        Write-Host "  $Yellow⚠ Android SDK not found$Reset"
        if (Show-Confirm "Download Android SDK?") {
            $env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"
            New-Item -ItemType Directory -Force -Path "$env:ANDROID_HOME\cmdline-tools" | Out-Null
            
            $url = "https://dl.google.com/android/repository/commandlinetools-win-11076708_latest.zip"
            Write-Host "  $Cyan⟳$Reset Downloading SDK..."
            Invoke-WebRequest -Uri $url -OutFile "$env:TEMP\sdk.zip"
            Expand-Archive "$env:TEMP\sdk.zip" -DestinationPath "$env:TEMP\sdk" -Force
            Move-Item "$env:TEMP\sdk\cmdline-tools" "$env:ANDROID_HOME\cmdline-tools\latest" -Force
            Remove-Item "$env:TEMP\sdk.zip", "$env:TEMP\sdk" -Recurse -Force
        } else { return $false }
    }
    
    $sdkm = "$env:ANDROID_HOME\cmdline-tools\latest\bin\sdkmanager.bat"
    if (Test-Path $sdkm) {
        Write-Host "  $Cyan⟳$Reset Installing NDK..."
        "y`n" * 10 | & $sdkm --licenses 2>$null
        & $sdkm "ndk;25.2.9519653" "platform-tools" "platforms;android-34" "build-tools;34.0.0" | Out-Null
        $env:NDK_HOME = "$env:ANDROID_HOME\ndk\25.2.9519653"
    }
    
    cargo tauri android init 2>$null
    Write-Host "  $Green✓$Reset Android ready"
    return $true
}

function Build-Target {
    param([string]$Target)
    
    Write-Host "`n  $Magenta►$Reset Building: $White$Target$Reset"
    
    switch -Wildcard ($Target) {
        "Windows*NSIS*" { cargo tauri build --bundles nsis }
        "Windows*MSI*" { cargo tauri build --bundles msi }
        "Windows*Portable*" { cargo tauri build --bundles nsis }
        "Android*APK*" {
            if (Initialize-Android) { cargo tauri android build --release }
        }
        "Android*AAB*" {
            if (Initialize-Android) { cargo tauri android build --release --aab }
        }
    }
}

function Main {
    Show-Banner
    Test-Dependencies
    
    # Windows can only build Windows + Android
    $targets = @(
        "Windows » NSIS Installer",
        "Windows » MSI Package",
        "Windows » Portable EXE",
        "Android » APK",
        "Android » AAB (Play Store)"
    )
    
    $selected = Show-CheckboxMenu -Options $targets -Title "Select Build Targets"
    
    if ($selected.Count -eq 0) {
        Write-Host "`n  $Yellow⚠ No targets selected$Reset"
        return
    }
    
    Write-Host "`n  $Cyan┌─────────────────────────────────────────┐$Reset"
    Write-Host "  $Cyan│$White  Selected Targets:$Cyan                      │$Reset"
    Write-Host "  $Cyan└─────────────────────────────────────────┘$Reset"
    $selected | ForEach-Object { Write-Host "    $Green✓$Reset $_" }
    
    if (Show-Confirm "`n  Start build?") {
        $selected | ForEach-Object { Build-Target $_ }
        
        Write-Host "`n  $Green╔═══════════════════════════════════════╗$Reset"
        Write-Host "  $Green║       ✓ BUILD COMPLETE                ║$Reset"
        Write-Host "  $Green╚═══════════════════════════════════════╝$Reset"
        Write-Host "  $Gray  Output: .\src-tauri\target\release\bundle\$Reset`n"
    }
    
    if (Show-Confirm "  Build more?") { Main }
}

Main
