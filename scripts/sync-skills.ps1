[CmdletBinding()]
param([switch]$Check)

$ErrorActionPreference = 'Stop'
$repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$source = Join-Path $repo 'skills'
$approved = @('github-issue-to-pr', 'github-profile-curator', 'github-readme-polish', 'github-release-prep', 'github-repository-audit')
$names = if (Test-Path -LiteralPath $source) { Get-ChildItem -LiteralPath $source -Directory | Where-Object Name -in $approved | ForEach-Object Name } else { @() }
$destinations = @('.agents/skills', '.claude/skills') | ForEach-Object { [IO.Path]::GetFullPath((Join-Path $repo $_)) }

foreach ($destination in $destinations) {
    if (-not $destination.StartsWith($repo + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw "Destination escapes repository: $destination" }
    $drift = $false
    foreach ($name in $approved) {
        $target = Join-Path $destination $name
        $wanted = $names -contains $name
        if ($wanted -and -not (Test-Path -LiteralPath $target)) { $drift = $true }
        if (-not $wanted -and (Test-Path -LiteralPath $target)) { $drift = $true }
    }
    if ($Check) {
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
