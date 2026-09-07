$ErrorActionPreference = 'Stop'
$repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$source = Join-Path $repo 'skills'
$approved = @('github-issue-to-pr', 'github-profile-curator', 'github-readme-polish', 'github-release-prep', 'github-repository-audit')
$destinations = @('.agents/skills', '.claude/skills') | ForEach-Object { [IO.Path]::GetFullPath((Join-Path $repo $_)) }

function Assert-SafeDestination($destination) {
    if (-not $destination.StartsWith($repo + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw "Destination escapes repository: $destination" }
    $relative = [IO.Path]::GetRelativePath($repo, $destination)
    $current = $repo
    foreach ($part in $relative -split '[\\/]') {
        $current = Join-Path $current $part
        $item = Get-Item -Force -LiteralPath $current -ErrorAction SilentlyContinue
        if ($null -ne $item -and ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw "Destination contains reparse point: $current" }
    }
}

function Get-OrdinaryTree($root, $label) {
    $rootItem = Get-Item -Force -LiteralPath $root -ErrorAction SilentlyContinue
    if ($null -eq $rootItem) {
        return [pscustomobject]@{ Directories = @(); Files = @() }
    }
    if ($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "$label contains reparse point: $($rootItem.FullName)" }
    if (-not $rootItem.PSIsContainer) { throw "$label is not a directory: $($rootItem.FullName)" }

    $directories = [Collections.Generic.List[string]]::new()
    $files = [Collections.Generic.List[string]]::new()
    $pending = [Collections.Generic.Stack[string]]::new()
    $pending.Push($rootItem.FullName)
    while ($pending.Count) {
        $current = Get-Item -Force -LiteralPath $pending.Pop()
        if ($current.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "$label contains reparse point: $($current.FullName)" }
        [void]$directories.Add($current.FullName)
        foreach ($child in Get-ChildItem -Force -LiteralPath $current.FullName) {
            if ($child.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "$label contains reparse point: $($child.FullName)" }
            if ($child.PSIsContainer) { $pending.Push($child.FullName) } else { [void]$files.Add($child.FullName) }
        }
    }
    return [pscustomobject]@{ Directories = $directories.ToArray(); Files = $files.ToArray() }
}

function Remove-OrdinaryTree($root) {
    $tree = Get-OrdinaryTree $root 'Generated skill tree'
    foreach ($file in $tree.Files) { Remove-Item -Force -LiteralPath $file }
    foreach ($directory in @($tree.Directories | Sort-Object Length -Descending)) { Remove-Item -Force -LiteralPath $directory }
}

$canonicalTree = Get-OrdinaryTree $source 'Canonical skill tree'
$names = if ($canonicalTree.Directories.Count) {
    Get-ChildItem -LiteralPath $source -Directory | Where-Object Name -in $approved | ForEach-Object Name | Sort-Object
} else { @() }
$sourceTrees = @{}
foreach ($name in $names) { $sourceTrees[$name] = Get-OrdinaryTree (Join-Path $source $name) "Canonical skill '$name'" }

foreach ($destination in $destinations) {
    Assert-SafeDestination $destination
    [void](Get-OrdinaryTree $destination 'Generated runtime tree')
}

$expectedDirectories = @()
$expectedFiles = @()
foreach ($name in $names) {
    $expectedDirectories += $sourceTrees[$name].Directories | ForEach-Object { [IO.Path]::GetRelativePath($source, $_) }
    $expectedFiles += $sourceTrees[$name].Files | ForEach-Object { [IO.Path]::GetRelativePath($source, $_) }
}

foreach ($destination in $destinations) {
    foreach ($name in $approved) {
        $target = Join-Path $destination $name
        if ($null -ne (Get-Item -Force -LiteralPath $target -ErrorAction SilentlyContinue)) { Remove-OrdinaryTree $target }
    }
    foreach ($directory in $expectedDirectories) { New-Item -ItemType Directory -Force -Path (Join-Path $destination $directory) | Out-Null }
    foreach ($file in $expectedFiles) { Copy-Item -Force -LiteralPath (Join-Path $source $file) -Destination (Join-Path $destination $file) }
}
