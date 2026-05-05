$projectName = "Cohbit-Legal-GMI"
$version = "1.0.0-hardened"
$zipFile = "${projectName}-${version}.zip"

Write-Host "Packaging ${projectName} version ${version}..."

# Clean up old zip if exists
if (Test-Path $zipFile) {
    Remove-Item $zipFile
}

# Create a temporary staging directory
$stagingDir = "zenodo_staging"
if (Test-Path $stagingDir) {
    Remove-Item -Recurse -Force $stagingDir
}
New-Item -ItemType Directory -Path $stagingDir

# Copy relevant files/folders
$exclude = @("target", "target_legal", "target_legal_adapters_2", "data", "zenodo_staging", ".git", ".github", $zipFile)

Get-ChildItem -Exclude $exclude | Copy-Item -Destination $stagingDir -Recurse

# Create the zip
Compress-Archive -Path "${stagingDir}\*" -DestinationPath $zipFile

# Clean up staging
Remove-Item -Recurse -Force $stagingDir

Write-Host "Zenodo package created: $zipFile"
