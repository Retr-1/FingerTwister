param([switch]$DirectDocker)

# Build in an isolated container copy: modern host Cargo may rewrite lock v3 to v4.
$ErrorActionPreference = 'Stop'
$projectRoot = $PSScriptRoot
$buildTarget = Join-Path $projectRoot 'target'
New-Item -ItemType Directory -Force -Path $buildTarget | Out-Null
$savedContext = $env:DOCKER_CONTEXT
$savedConfig = $env:DOCKER_CONFIG
$dockerArgs = @()
try {
    if ($DirectDocker) {
        # Use a private CLI config and direct endpoint to bypass locked context files.
        $env:DOCKER_CONTEXT = $null
        $env:DOCKER_CONFIG = Join-Path $buildTarget ('docker-cli-' + [guid]::NewGuid().ToString('N'))
        New-Item -ItemType Directory -Path $env:DOCKER_CONFIG | Out-Null
        $dockerArgs = @('--config', $env:DOCKER_CONFIG, '--host', 'npipe:////./pipe/dockerDesktopLinuxEngine')
    }
    docker @dockerArgs info --format '{{.OSType}}'
    if ($LASTEXITCODE -ne 0) { throw 'Docker is unavailable. Restart Docker Desktop with Linux containers. For a locked context file, retry with -DirectDocker.' }
Get-Content -Raw (Join-Path $projectRoot 'Dockerfile.android') | docker @dockerArgs build --tag fingertwister-android:api36 -
if ($LASTEXITCODE -ne 0) { throw 'Android build image update failed.' }
docker @dockerArgs run --rm --mount "type=volume,source=fingertwister-android-signing,target=/root/.android" --mount "type=volume,source=fingertwister-android-registry,target=/usr/local/cargo/registry" -e CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse --mount "type=bind,source=$projectRoot,target=/project,readonly" --mount "type=bind,source=$buildTarget,target=/root/src/target" -w /root/src fingertwister-android:api36 sh /project/build-android.sh
if ($LASTEXITCODE -ne 0) { throw 'Android build failed; see Docker output above.' }
Write-Host "APK directory: $buildTarget\android-artifacts\release\apk"






} finally {
    $env:DOCKER_CONTEXT = $savedContext
    $env:DOCKER_CONFIG = $savedConfig
}
