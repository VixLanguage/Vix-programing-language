
$version = Read-Host "Tell me the version of your Minecraft"

$source = "C:\Users\Collin\Download\optifine.jar"
$versionsPath = "C:\Users\Collin\AppData\Roaming\.minecraft\versions"
$targetVersionPath = Join-Path $versionsPath $version
$modsPath = Join-Path $targetVersionPath "mods"

if (Test-Path $source) {
    Copy-Item -Path $source -Destination $targetVersionPath -Force
    Write-Host "Optifine.jar copied to $targetVersionPath"
} else {
    Write-Host "optifine.jar not found in Downloads."
}

if (Test-Path $modsPath) {
    Write-Host "Mods folder exists at $modsPath"
} else {
    Write-Host "Mods folder not found."
    $troubleshoot = Read-Host "Type '/troubleshoot' to see everything inside the version folder"
    if ($troubleshoot -eq "/troubleshoot") {
        Get-ChildItem -Path $targetVersionPath
    }
}
