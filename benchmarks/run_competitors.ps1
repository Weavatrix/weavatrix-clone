[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Corpus,
    [Parameter(Mandatory = $true)]
    [string]$Weavatrix,
    [Parameter(Mandatory = $true)]
    [string]$Jscpd,
    [Parameter(Mandatory = $true)]
    [string]$WeavatrixFormat,
    [Parameter(Mandatory = $true)]
    [string]$JscpdFormat,
    [string]$Pmd,
    [string]$JavaHome,
    [string]$PmdLanguage,
    [ValidateRange(3, 101)]
    [int]$Samples = 21,
    [ValidateRange(3, 101)]
    [int]$PmdSamples = 11,
    [switch]$PmdAllowErrors,
    [string]$OutputJson
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ($Samples % 2 -eq 0 -or $PmdSamples % 2 -eq 0) {
    throw "Samples and PmdSamples must be odd so the median is observed."
}
foreach ($path in @($Corpus, $Weavatrix, $Jscpd)) {
    if (-not (Test-Path -LiteralPath $path)) {
        throw "Required path does not exist: $path"
    }
}
if ($Pmd -and ((-not $JavaHome) -or (-not $PmdLanguage))) {
    throw "Pmd requires JavaHome and PmdLanguage."
}
if ($Pmd) {
    foreach ($path in @($Pmd, $JavaHome)) {
        if (-not (Test-Path -LiteralPath $path)) {
            throw "PMD path does not exist: $path"
        }
    }
    $env:JAVA_HOME = (Resolve-Path -LiteralPath $JavaHome).Path
    $javaBin = Join-Path $env:JAVA_HOME "bin"
    $env:PATH = "$javaBin;$env:PATH"
}

$Corpus = (Resolve-Path -LiteralPath $Corpus).Path
$Weavatrix = (Resolve-Path -LiteralPath $Weavatrix).Path
$Jscpd = (Resolve-Path -LiteralPath $Jscpd).Path
if ($Pmd) {
    $Pmd = (Resolve-Path -LiteralPath $Pmd).Path
}

function Invoke-Detector {
    param([Parameter(Mandatory = $true)][string]$Name)

    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    switch ($Name) {
        "weavatrix" {
            & $Weavatrix --summary --mode exact --min-tokens 24 `
                --min-lines 3 --format $WeavatrixFormat $Corpus *> $null
        }
        "jscpd" {
            & $Jscpd --format $JscpdFormat --min-tokens 24 `
                --min-lines 3 --max-size 1500000 --no-gitignore `
                --reporters silent $Corpus *> $null
        }
        "pmd" {
            $arguments = @(
                "cpd", "--minimum-tokens", "24",
                "--language", $PmdLanguage,
                "--format", "csv",
                "--no-fail-on-violation",
                "--dir", $Corpus
            )
            if ($PmdAllowErrors) {
                $arguments += "--no-fail-on-error"
            }
            & $Pmd @arguments *> $null
        }
        default { throw "Unknown detector: $Name" }
    }
    $timer.Stop()
    if ($LASTEXITCODE -ne 0) {
        throw "$Name exited with code $LASTEXITCODE"
    }
    $timer.Elapsed.TotalMilliseconds
}

function Convert-Samples {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][double[]]$Values
    )

    $sorted = @($Values | Sort-Object)
    $middle = [int][Math]::Floor($sorted.Count / 2)
    $p95 = [Math]::Min(
        $sorted.Count - 1,
        [int][Math]::Ceiling($sorted.Count * 0.95) - 1
    )
    [pscustomobject]@{
        tool = $Name
        medianMs = [Math]::Round($sorted[$middle], 3)
        p95Ms = [Math]::Round($sorted[$p95], 3)
        minMs = [Math]::Round($sorted[0], 3)
        samples = $sorted.Count
    }
}

foreach ($name in @("weavatrix", "jscpd")) {
    1..2 | ForEach-Object { Invoke-Detector $name | Out-Null }
}
$native = @{ weavatrix = @(); jscpd = @() }
for ($index = 0; $index -lt $Samples; $index++) {
    $order = if ($index % 2 -eq 0) {
        @("weavatrix", "jscpd")
    } else {
        @("jscpd", "weavatrix")
    }
    foreach ($name in $order) {
        $native[$name] += Invoke-Detector $name
    }
}

$result = @(
    Convert-Samples "weavatrix" $native.weavatrix
    Convert-Samples "jscpd" $native.jscpd
)
if ($Pmd) {
    Invoke-Detector "pmd" | Out-Null
    $values = 1..$PmdSamples | ForEach-Object { Invoke-Detector "pmd" }
    $result += Convert-Samples "pmd" $values
}

$result | Format-Table -AutoSize
if ($OutputJson) {
    $result | ConvertTo-Json | Set-Content -LiteralPath $OutputJson
}
