[CmdletBinding()]
param(
    [Parameter()]
    [AllowEmptyCollection()]
    [string[]] $RunCargo
)

Set-StrictMode -Version Latest

function Get-RunBobCommandPath {
    param([Parameter(Mandatory)][string] $Name)

    $command = Get-Command -Name $Name -CommandType Application -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if ($null -eq $command) {
        return $null
    }
    if ($command.Source) {
        return $command.Source
    }
    return $command.Path
}

function Get-RunBobArchitecture {
    if (-not [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
            [System.Runtime.InteropServices.OSPlatform]::Windows)) {
        throw 'The PowerShell Rust bootstrap only supports native Windows.'
    }
    return [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
}

function Invoke-RunBobExternalProcess {
    param(
        [Parameter(Mandatory)][string] $FilePath,
        [Parameter()][AllowEmptyCollection()][string[]] $ArgumentList = @()
    )

    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $FilePath
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    foreach ($argument in $ArgumentList) {
        [void] $startInfo.ArgumentList.Add($argument)
    }

    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    try {
        if (-not $process.Start()) {
            throw "could not start $FilePath"
        }
        $standardOutput = $process.StandardOutput.ReadToEndAsync()
        $standardError = $process.StandardError.ReadToEndAsync()
        $process.WaitForExit()
        return [pscustomobject]@{
            ExitCode = $process.ExitCode
            StdOut = $standardOutput.GetAwaiter().GetResult()
            StdErr = $standardError.GetAwaiter().GetResult()
        }
    }
    finally {
        $process.Dispose()
    }
}

function Invoke-RunBobDownload {
    param(
        [Parameter(Mandatory)][string] $Uri,
        [Parameter(Mandatory)][string] $OutFile
    )

    Invoke-WebRequest -Uri $Uri -OutFile $OutFile -UseBasicParsing
}

function Get-RunBobManifestRequirement {
    param([Parameter(Mandatory)][string] $ManifestPath)

    if (-not (Test-Path -LiteralPath $ManifestPath -PathType Leaf)) {
        throw "Cargo.toml not found at $ManifestPath"
    }

    $inPackage = $false
    $packageName = $null
    $rustVersionText = $null
    foreach ($line in Get-Content -LiteralPath $ManifestPath) {
        if ($line -match '^\s*\[package\]\s*$') {
            $inPackage = $true
            continue
        }
        if ($inPackage -and $line -match '^\s*\[') {
            break
        }
        if (-not $inPackage) {
            continue
        }
        if ($line -match '^\s*name\s*=\s*"([^"]+)"\s*$') {
            $packageName = $Matches[1]
        }
        elseif ($line -match '^\s*rust-version\s*=\s*"([^"]+)"\s*$') {
            $rustVersionText = $Matches[1]
        }
    }

    if ($packageName -ne 'run-bob') {
        throw "expected package name run-bob in $ManifestPath"
    }
    if ($rustVersionText -notmatch '^(\d+)\.(\d+)(?:\.(\d+))?$') {
        throw 'Cargo.toml rust-version must be a complete numeric major.minor or major.minor.patch version'
    }
    $patch = if ($null -eq $Matches[3] -or $Matches[3] -eq '') { 0 } else { [int] $Matches[3] }
    $requiredVersion = [version]::new([int] $Matches[1], [int] $Matches[2], $patch)
    return [pscustomobject]@{
        PackageName = $packageName
        RequiredVersion = $requiredVersion
    }
}

function ConvertTo-RunBobToolVersion {
    param(
        [Parameter(Mandatory)][string] $VersionOutput,
        [Parameter(Mandatory)][ValidateSet('rustc', 'cargo')][string] $ExpectedTool
    )

    $firstLine = ($VersionOutput -split "`r?`n", 2)[0]
    $escapedTool = [regex]::Escape($ExpectedTool)
    if ($firstLine -notmatch "^$escapedTool\s+(\d+\.\d+\.\d+)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\s|$)") {
        throw "could not read a complete $ExpectedTool semantic version"
    }
    $coreText = $Matches[1]
    $preRelease = $Matches[2]
    if ($preRelease) {
        foreach ($identifier in $preRelease.Split('.')) {
            if ($identifier -match '^\d+$' -and $identifier.Length -gt 1 -and $identifier.StartsWith('0')) {
                throw "could not read a complete $ExpectedTool semantic version"
            }
        }
    }

    $display = if ($preRelease) { "$coreText-$preRelease" } else { $coreText }
    return [pscustomobject]@{
        Core = [version] $coreText
        PreRelease = $preRelease
        Display = $display
    }
}

function Test-RunBobVersionAtLeast {
    param(
        [Parameter(Mandatory)] $Actual,
        [Parameter(Mandatory)][version] $Required
    )

    $comparison = $Actual.Core.CompareTo($Required)
    if ($comparison -gt 0) {
        return $true
    }
    if ($comparison -lt 0) {
        return $false
    }
    return [string]::IsNullOrEmpty($Actual.PreRelease)
}

function Get-RunBobToolVersion {
    param(
        [Parameter(Mandatory)][string] $ToolPath,
        [Parameter(Mandatory)][ValidateSet('rustc', 'cargo')][string] $ToolName,
        [Parameter()][string] $RustupPath
    )

    if ($RustupPath) {
        $arguments = @('run', 'stable', $ToolName, '--version')
        $result = Invoke-RunBobExternalProcess -FilePath $RustupPath -ArgumentList $arguments
    }
    else {
        $result = Invoke-RunBobExternalProcess -FilePath $ToolPath -ArgumentList @('--version')
    }
    if ($result.ExitCode -ne 0) {
        throw "bootstrap did not provide a complete Rust toolchain ($ToolName unavailable): $($result.StdErr.Trim())"
    }
    return ConvertTo-RunBobToolVersion -VersionOutput $result.StdOut -ExpectedTool $ToolName
}

function Test-RunBobActiveCompilerRustupOwned {
    param(
        [Parameter(Mandatory)][string] $RustcPath,
        [Parameter(Mandatory)][string] $RustupPath
    )

    $sysrootResult = Invoke-RunBobExternalProcess -FilePath $RustcPath -ArgumentList @('--print', 'sysroot')
    $whichResult = Invoke-RunBobExternalProcess -FilePath $RustupPath -ArgumentList @('which', 'rustc')
    if ($sysrootResult.ExitCode -ne 0 -or $whichResult.ExitCode -ne 0) {
        return $false
    }
    $activeSysroot = $sysrootResult.StdOut.Trim()
    $rustupCompiler = $whichResult.StdOut.Trim()
    if (-not $activeSysroot -or -not $rustupCompiler -or
        -not (Test-Path -LiteralPath $activeSysroot -PathType Container) -or
        -not (Test-Path -LiteralPath $rustupCompiler -PathType Leaf)) {
        return $false
    }

    $trimCharacters = [char[]] @('\', '/')
    $activeFullPath = (Resolve-Path -LiteralPath $activeSysroot).ProviderPath.TrimEnd($trimCharacters)
    $resolvedCompiler = (Resolve-Path -LiteralPath $rustupCompiler).ProviderPath
    $rustupBin = [System.IO.Path]::GetDirectoryName($resolvedCompiler)
    $rustupSysroot = [System.IO.Path]::GetFullPath((Join-Path $rustupBin '..')).TrimEnd($trimCharacters)
    return [string]::Equals($activeFullPath, $rustupSysroot, [System.StringComparison]::OrdinalIgnoreCase)
}

function Install-RunBobRustupStable {
    param([Parameter(Mandatory)][string] $RustupPath)

    $result = Invoke-RunBobExternalProcess -FilePath $RustupPath -ArgumentList @(
        'toolchain', 'install', 'stable', '--profile', 'minimal'
    )
    if ($result.ExitCode -ne 0) {
        throw "rustup could not install the stable toolchain with the minimal profile: $($result.StdErr.Trim())"
    }
}

function Get-RunBobInstalledRustupPath {
    $cargoHome = $env:CARGO_HOME
    if (-not $cargoHome -and $env:USERPROFILE) {
        $cargoHome = Join-Path $env:USERPROFILE '.cargo'
    }
    if ($cargoHome) {
        $candidate = Join-Path $cargoHome 'bin\rustup.exe'
        if (Test-Path -LiteralPath $candidate -PathType Leaf) {
            return $candidate
        }
    }
    return Get-RunBobCommandPath -Name 'rustup'
}

function New-RunBobInstallerPath {
    return Join-Path ([System.IO.Path]::GetTempPath()) (
        'run-bob-rustup-{0}.exe' -f [guid]::NewGuid()
    )
}

function Install-RunBobOfficialRust {
    $architecture = Get-RunBobArchitecture
    $installerUri = switch ($architecture) {
        'X64' { 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe' }
        'Arm64' { 'https://static.rust-lang.org/rustup/dist/aarch64-pc-windows-msvc/rustup-init.exe' }
        default { throw "unsupported Windows architecture for automatic Rust installation: $architecture" }
    }

    $installerPath = New-RunBobInstallerPath
    try {
        Invoke-RunBobDownload -Uri $installerUri -OutFile $installerPath
        $result = Invoke-RunBobExternalProcess -FilePath $installerPath -ArgumentList @(
            '-y', '--profile', 'minimal', '--default-toolchain', 'stable', '--no-modify-path'
        )
        if ($result.ExitCode -ne 0) {
            $detail = (@($result.StdErr.Trim(), $result.StdOut.Trim()) | Where-Object { $_ }) -join ' '
            throw "official Rust installer failed. Install the Visual C++ Build Tools if requested, then retry. $detail"
        }
    }
    catch {
        if ($_.Exception.Message -match '^official Rust installer failed') {
            throw
        }
        throw "could not download or run the official Rust installer: $($_.Exception.Message)"
    }
    finally {
        if (Test-Path -LiteralPath $installerPath) {
            Remove-Item -LiteralPath $installerPath -Force -ErrorAction SilentlyContinue
        }
    }

    $rustupPath = Get-RunBobInstalledRustupPath
    if (-not $rustupPath) {
        throw 'official installer completed but rustup is unavailable'
    }
    return $rustupPath
}

function Assert-RunBobSelectedToolchain {
    param(
        [Parameter(Mandatory)][ValidateSet('direct', 'rustup')][string] $Mode,
        [Parameter(Mandatory)][version] $RequiredVersion,
        [Parameter()][string] $RustcPath,
        [Parameter()][string] $CargoPath,
        [Parameter()][string] $RustupPath
    )

    if ($Mode -eq 'direct') {
        $rustcVersion = Get-RunBobToolVersion -ToolPath $RustcPath -ToolName rustc
        $cargoVersion = Get-RunBobToolVersion -ToolPath $CargoPath -ToolName cargo
    }
    else {
        $rustcVersion = Get-RunBobToolVersion -ToolPath $RustupPath -ToolName rustc -RustupPath $RustupPath
        $cargoVersion = Get-RunBobToolVersion -ToolPath $RustupPath -ToolName cargo -RustupPath $RustupPath
    }

    if (-not (Test-RunBobVersionAtLeast -Actual $rustcVersion -Required $RequiredVersion)) {
        throw "bootstrapped rustc $($rustcVersion.Display) is older than required Rust $RequiredVersion"
    }
    if (-not (Test-RunBobVersionAtLeast -Actual $cargoVersion -Required $RequiredVersion)) {
        throw "bootstrapped cargo $($cargoVersion.Display) is older than required Rust $RequiredVersion"
    }
}

function Invoke-RunBobBootstrap {
    [CmdletBinding()]
    param(
        [Parameter()][AllowEmptyCollection()][string[]] $RunCargo,
        [Parameter()][switch] $RunCargoSpecified,
        [Parameter()][string] $ManifestPath = (Join-Path $PSScriptRoot '..\Cargo.toml')
    )

    if ($RunCargoSpecified -and
        ($null -eq $RunCargo -or $RunCargo.Count -eq 0 -or
            [string]::IsNullOrWhiteSpace($RunCargo[0]))) {
        throw 'Usage: bootstrap-rust.ps1 [-RunCargo <cargo arguments>]'
    }

    $manifest = Get-RunBobManifestRequirement -ManifestPath $ManifestPath
    $requiredVersion = $manifest.RequiredVersion
    $rustcPath = Get-RunBobCommandPath -Name 'rustc'
    $cargoPath = Get-RunBobCommandPath -Name 'cargo'
    $rustupPath = Get-RunBobCommandPath -Name 'rustup'
    $mode = $null

    if ($rustcPath -and $cargoPath) {
        $rustcVersion = Get-RunBobToolVersion -ToolPath $rustcPath -ToolName rustc
        $cargoVersion = Get-RunBobToolVersion -ToolPath $cargoPath -ToolName cargo
        if ((Test-RunBobVersionAtLeast -Actual $rustcVersion -Required $requiredVersion) -and
            (Test-RunBobVersionAtLeast -Actual $cargoVersion -Required $requiredVersion)) {
            $mode = 'direct'
        }
        else {
            $detected = "detected rustc $($rustcVersion.Display) and cargo $($cargoVersion.Display); requires rustc and cargo >= $requiredVersion"
            if (-not $rustupPath -or
                -not (Test-RunBobActiveCompilerRustupOwned -RustcPath $rustcPath -RustupPath $rustupPath)) {
                throw "$detected; active compiler is not rustup-owned; refusing to replace it"
            }
            Install-RunBobRustupStable -RustupPath $rustupPath
            $mode = 'rustup'
        }
    }
    elseif (-not $rustcPath -and -not $cargoPath) {
        if ($rustupPath) {
            Install-RunBobRustupStable -RustupPath $rustupPath
        }
        else {
            $rustupPath = Install-RunBobOfficialRust
        }
        $mode = 'rustup'
    }
    else {
        if ($rustcPath -and $rustupPath -and
            (Test-RunBobActiveCompilerRustupOwned -RustcPath $rustcPath -RustupPath $rustupPath)) {
            Install-RunBobRustupStable -RustupPath $rustupPath
            $mode = 'rustup'
        }
        else {
            throw 'a partial non-rustup Rust toolchain is installed; refusing to replace it automatically'
        }
    }

    Assert-RunBobSelectedToolchain -Mode $mode -RequiredVersion $requiredVersion `
        -RustcPath $rustcPath -CargoPath $cargoPath -RustupPath $rustupPath

    if ($RunCargoSpecified) {
        if ($mode -eq 'direct') {
            $cargoResult = Invoke-RunBobExternalProcess -FilePath $CargoPath -ArgumentList $RunCargo
        }
        else {
            $cargoArguments = @('run', 'stable', 'cargo') + $RunCargo
            $cargoResult = Invoke-RunBobExternalProcess -FilePath $rustupPath -ArgumentList $cargoArguments
        }
        if ($cargoResult.StdOut) { [Console]::Out.Write($cargoResult.StdOut) }
        if ($cargoResult.StdErr) { [Console]::Error.Write($cargoResult.StdErr) }
        if ($cargoResult.ExitCode -ne 0) {
            throw "cargo exited with status $($cargoResult.ExitCode)"
        }
    }
}

if ($MyInvocation.InvocationName -ne '.') {
    try {
        $specified = $PSBoundParameters.ContainsKey('RunCargo')
        Invoke-RunBobBootstrap -RunCargo $RunCargo -RunCargoSpecified:$specified
    }
    catch {
        [Console]::Error.WriteLine("error: $($_.Exception.Message)")
        exit 1
    }
}
