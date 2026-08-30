[CmdletBinding()]
param([switch]$Check)

$ErrorActionPreference = 'Stop'
$repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$source = Join-Path $repo 'skills'
$approved = @('github-issue-to-pr', 'github-profile-curator', 'github-readme-polish', 'github-release-prep', 'github-repository-audit')
$names = if (Test-Path -LiteralPath $source) { Get-ChildItem -LiteralPath $source -Directory | Where-Object Name -in $approved | ForEach-Object Name } else { @() }
$destinations = @('.agents/skills', '.claude/skills') | ForEach-Object { [IO.Path]::GetFullPath((Join-Path $repo $_)) }

function Assert-SafeDestination($destination) {
    if (-not $destination.StartsWith($repo + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw "Destination escapes repository: $destination" }
    $relative = [IO.Path]::GetRelativePath($repo, $destination)
    $current = $repo
    foreach ($part in $relative -split '[\\/]') {
        $current = Join-Path $current $part
        if ((Test-Path -LiteralPath $current) -and ((Get-Item -Force -LiteralPath $current).Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw "Destination contains reparse point: $current" }
    }
}

foreach ($destination in $destinations) {
    Assert-SafeDestination $destination
    if ($Check) {
        $expectedDirectories = @($names)
        $expectedFiles = @()
        foreach ($name in $names) {
            $skillSource = Join-Path $source $name
            $expectedDirectories += Get-ChildItem -LiteralPath $skillSource -Directory -Recurse | ForEach-Object { [IO.Path]::GetRelativePath($source, $_.FullName) }
            $expectedFiles += Get-ChildItem -LiteralPath $skillSource -File -Recurse | ForEach-Object { [IO.Path]::GetRelativePath($source, $_.FullName) }
        }
        $actualDirectories = if (Test-Path -LiteralPath $destination) { @(Get-ChildItem -LiteralPath $destination -Directory -Recurse | ForEach-Object { [IO.Path]::GetRelativePath($destination, $_.FullName) }) } else { @() }
        $actualFiles = if (Test-Path -LiteralPath $destination) { @(Get-ChildItem -LiteralPath $destination -File -Recurse | ForEach-Object { [IO.Path]::GetRelativePath($destination, $_.FullName) }) } else { @() }
        $drift = (Compare-Object (@($expectedDirectories) + '') (@($actualDirectories) + '')) -or (Compare-Object (@($expectedFiles) + '') (@($actualFiles) + ''))
        foreach ($file in $expectedFiles) {
            if ((Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $source $file)).Hash -ne (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $destination $file)).Hash) { $drift = $true }
        }
        if ($drift) { Write-Error "Skill copies are out of sync in $destination" }
        continue
    }
    foreach ($name in $approved) {
        $target = Join-Path $destination $name
        if (Test-Path -LiteralPath $target) { Remove-Item -LiteralPath $target -Recurse -Force }
    }
    foreach ($name in $names) {
        New-Item -ItemType Directory -Force -Path $destination | Out-Null
        Copy-Item -LiteralPath (Join-Path $source $name) -Destination (Join-Path $destination $name) -Recurse
    }
}
