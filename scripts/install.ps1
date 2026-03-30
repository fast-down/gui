$ErrorActionPreference = 'Stop'

$ARCH = $env:PROCESSOR_ARCHITECTURE
$BASE_URL = "https://fast-down-update.s121.top/gui/download/latest"
$DOWNLOAD_URL = "$BASE_URL/windows/$ARCH"
$INSTALL_DIR = "$env:LOCALAPPDATA\Programs\fast-down-gui"
$BIN_NAME = "fast-down.exe"
$EXE_PATH = "$INSTALL_DIR\$BIN_NAME"
$TMP_FILE = [System.IO.Path]::GetTempFileName()

Write-Host "✨ Downloading $DOWNLOAD_URL"
try
{
    Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile $TMP_FILE -UseBasicParsing
} catch
{
    Write-Host "❌ Error: Failed to download the file: $_"
    if (Test-Path $TMP_FILE)
    {
        Remove-Item -Path $TMP_FILE
    }
    exit 1
}

if (-not (Test-Path -Path $INSTALL_DIR))
{
    New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null
}
Move-Item -Path $TMP_FILE -Destination $EXE_PATH -Force
Write-Host "🎉 Installed to $EXE_PATH"

try
{
    Write-Host "🔗 Creating desktop shortcut..."
    $DesktopPath = [Environment]::GetFolderPath("Desktop")
    $ShortcutPath = Join-Path $DesktopPath "fast-down.lnk"

    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut($ShortcutPath)
    $Shortcut.TargetPath = $EXE_PATH
    $Shortcut.WorkingDirectory = $INSTALL_DIR
    $Shortcut.Description = "Fast Down GUI Client"
    $Shortcut.Save()

    Write-Host "✅ Shortcut created on Desktop: $ShortcutPath"
} catch
{
    Write-Host "❌ Warning: Failed to create shortcut: $_"
}
