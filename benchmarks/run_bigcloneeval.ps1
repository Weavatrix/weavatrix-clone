param(
    [Parameter(Mandatory = $true)]
    [string]$DatasetRoot,
    [Parameter(Mandatory = $true)]
    [string]$OutputPath,
    [int]$MinTokens = 50,
    [int]$MinLines = 6
)

$ErrorActionPreference = "Stop"
$crateRoot = Split-Path -Parent $PSScriptRoot
$dataset = (Resolve-Path -LiteralPath $DatasetRoot).Path
$output = [System.IO.Path]::GetFullPath($OutputPath)
$outputParent = Split-Path -Parent $output
if (-not [System.IO.Directory]::Exists($outputParent)) {
    throw "Output directory does not exist: $outputParent"
}

& cargo build --release --manifest-path (Join-Path $crateRoot "Cargo.toml")
if ($LASTEXITCODE -ne 0) {
    throw "Release build failed"
}

$binary = Join-Path $crateRoot "target/release/weavatrix-clone.exe"
if (-not (Test-Path -LiteralPath $binary)) {
    $binary = Join-Path $crateRoot "target/release/weavatrix-clone"
}
$writer = [System.IO.StreamWriter]::new($output, $false, [System.Text.UTF8Encoding]::new($false))
try {
    $subsets = Get-ChildItem -LiteralPath $dataset -Directory | Sort-Object Name
    foreach ($subset in $subsets) {
        $arguments = @(
            "--mode", "near",
            "--min-tokens", $MinTokens,
            "--min-lines", $MinLines,
            "--format", "java",
            "--output-format", "bigcloneeval",
            $subset.FullName
        )
        & $binary @arguments | ForEach-Object { $writer.WriteLine($_) }
        if ($LASTEXITCODE -ne 0) {
            throw "Detection failed for subset $($subset.Name)"
        }
    }
}
finally {
    $writer.Dispose()
}

Write-Output "BigCloneEval import file: $output"
