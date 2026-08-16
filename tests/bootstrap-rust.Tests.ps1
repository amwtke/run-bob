BeforeAll {
    . (Join-Path $PSScriptRoot '..\scripts\bootstrap-rust.ps1')

    function New-TestProcessResult {
        param(
            [int] $ExitCode = 0,
            [string] $StdOut = '',
            [string] $StdErr = ''
        )
        return [pscustomobject]@{
            ExitCode = $ExitCode
            StdOut = $StdOut
            StdErr = $StdErr
        }
    }

    function New-TestCargoHomeTools {
        param(
            [Parameter(Mandatory)][string] $CargoHome,
            [switch] $Rustc,
            [switch] $Cargo,
            [switch] $Rustup
        )

        $bin = Join-Path $CargoHome 'bin'
        New-Item -ItemType Directory -Path $bin -Force | Out-Null
        if ($Rustc) { Set-Content -LiteralPath (Join-Path $bin 'rustc.exe') -Value 'proxy' }
        if ($Cargo) { Set-Content -LiteralPath (Join-Path $bin 'cargo.exe') -Value 'proxy' }
        if ($Rustup) { Set-Content -LiteralPath (Join-Path $bin 'rustup.exe') -Value 'proxy' }
        return [pscustomobject]@{
            Bin = $bin
            Rustc = Join-Path $bin 'rustc.exe'
            Cargo = Join-Path $bin 'cargo.exe'
            Rustup = Join-Path $bin 'rustup.exe'
        }
    }
}

Describe 'bootstrap-rust.ps1' {
    BeforeEach {
        $script:originalCargoHome = $env:CARGO_HOME
        $script:originalUserProfile = $env:USERPROFILE
        $script:caseRoot = Join-Path $TestDrive ([guid]::NewGuid().ToString('N'))
        New-Item -ItemType Directory -Path $script:caseRoot -Force | Out-Null
        $env:CARGO_HOME = Join-Path $script:caseRoot 'isolated empty cargo home'
        $env:USERPROFILE = Join-Path $script:caseRoot 'isolated empty profile'
        $script:manifestPath = Join-Path $script:caseRoot 'Cargo.toml'
        @'
[package]
name = "run-bob"
version = "0.0.0"
rust-version = "1.75"
'@ | Set-Content -LiteralPath $script:manifestPath
        $script:processCalls = [System.Collections.Generic.List[object]]::new()
        $script:downloadPath = $null
        Mock Invoke-WebRequest { throw 'network must not be used' }
    }

    AfterEach {
        $env:CARGO_HOME = $script:originalCargoHome
        $env:USERPROFILE = $script:originalUserProfile
    }

    It 'uses a supported direct toolchain and forwards exact cargo arguments' {
        Mock Get-RunBobCommandPath {
            switch ($Name) {
                'rustc' { 'C:\mock\rustc.exe' }
                'cargo' { 'C:\mock\cargo.exe' }
                'rustup' { $null }
            }
        }
        Mock Invoke-RunBobExternalProcess {
            [void] $script:processCalls.Add([pscustomobject]@{
                FilePath = $FilePath
                ArgumentList = @($ArgumentList)
            })
            if ($ArgumentList.Count -eq 1 -and $ArgumentList[0] -eq '--version') {
                if ($FilePath -like '*rustc.exe') {
                    return New-TestProcessResult -StdOut 'rustc 1.75.0 (mock)'
                }
                return New-TestProcessResult -StdOut 'cargo 1.75.0 (mock)'
            }
            return New-TestProcessResult
        }

        $arguments = @('test', '--locked', '--', 'case with spaces')
        Invoke-RunBobBootstrap -ManifestPath $script:manifestPath `
            -RunCargoSpecified -RunCargo $arguments

        $cargoCalls = @($script:processCalls | Where-Object {
            $_.FilePath -eq 'C:\mock\cargo.exe' -and $_.ArgumentList.Count -eq 4
        })
        $cargoCalls.Count | Should -Be 1
        ($cargoCalls[0].ArgumentList -join "`u{1f}") |
            Should -Be ($arguments -join "`u{1f}")
        Should -Invoke Invoke-WebRequest -Times 0 -Exactly
    }

    It 'accepts newer nightly and beta cores but rejects an equal-core prerelease' {
        Mock Get-RunBobCommandPath {
            switch ($Name) {
                'rustc' { 'C:\mock\rustc.exe' }
                'cargo' { 'C:\mock\cargo.exe' }
                'rustup' { $null }
            }
        }
        Mock Invoke-RunBobExternalProcess {
            if ($FilePath -like '*rustc.exe') {
                return New-TestProcessResult -StdOut 'rustc 1.90.0-nightly-2026-01-01 (mock)'
            }
            return New-TestProcessResult -StdOut 'cargo 1.90.0-beta.1 (mock)'
        }
        { Invoke-RunBobBootstrap -ManifestPath $script:manifestPath } | Should -Not -Throw

        Mock Invoke-RunBobExternalProcess {
            if ($FilePath -like '*rustc.exe') {
                return New-TestProcessResult -StdOut 'rustc 1.75.0-nightly (mock)'
            }
            return New-TestProcessResult -StdOut 'cargo 1.75.0-beta.1 (mock)'
        }
        { Invoke-RunBobBootstrap -ManifestPath $script:manifestPath } |
            Should -Throw '*requires rustc and cargo >= 1.75.0*'
    }

    It 'takes the required version strictly from Cargo.toml' {
        (Get-Content -LiteralPath $script:manifestPath) -replace '1.75', '1.76' |
            Set-Content -LiteralPath $script:manifestPath
        Mock Get-RunBobCommandPath {
            switch ($Name) {
                'rustc' { 'C:\mock\rustc.exe' }
                'cargo' { 'C:\mock\cargo.exe' }
                'rustup' { $null }
            }
        }
        Mock Invoke-RunBobExternalProcess {
            if ($FilePath -like '*rustc.exe') {
                return New-TestProcessResult -StdOut 'rustc 1.75.0 (mock)'
            }
            return New-TestProcessResult -StdOut 'cargo 1.75.0 (mock)'
        }

        { Invoke-RunBobBootstrap -ManifestPath $script:manifestPath } |
            Should -Throw '*requires rustc and cargo >= 1.76.0*'
    }

    Context 'when Rust and rustup are absent' {
        BeforeEach {
            Mock Get-RunBobCommandPath { $null }
            Mock Get-RunBobArchitecture { 'X64' }
            Mock Get-RunBobInstalledRustupPath { 'C:\installed\rustup.exe' }
            Mock New-RunBobInstallerPath {
                Join-Path $TestDrive ('run-bob-rustup-{0}.exe' -f [guid]::NewGuid())
            }
            Mock Invoke-WebRequest {
                $script:downloadPath = $OutFile
                Set-Content -LiteralPath $OutFile -Value 'mock installer'
            }
            Mock Invoke-RunBobExternalProcess {
                [void] $script:processCalls.Add([pscustomobject]@{
                    FilePath = $FilePath
                    ArgumentList = @($ArgumentList)
                })
                if ($ArgumentList -contains '--version') {
                    if ($ArgumentList -contains 'rustc') {
                        return New-TestProcessResult -StdOut 'rustc 1.76.0 (mock)'
                    }
                    return New-TestProcessResult -StdOut 'cargo 1.76.0 (mock)'
                }
                return New-TestProcessResult
            }
        }

        It 'downloads the fixed X64 installer, uses safe flags, verifies, and cleans up' {
            Invoke-RunBobBootstrap -ManifestPath $script:manifestPath

            Should -Invoke Invoke-WebRequest -Times 1 -Exactly -ParameterFilter {
                $Uri -eq 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe'
            }
            $script:downloadPath | Should -Match 'run-bob-rustup-[0-9a-f-]+\.exe$'
            (Test-Path -LiteralPath $script:downloadPath) | Should -BeFalse
            $installerCall = @($script:processCalls | Where-Object {
                $_.FilePath -match 'run-bob-rustup-[0-9a-f-]+\.exe$'
            })
            $installerCall.Count | Should -Be 1
            ($installerCall[0].ArgumentList -join ' ') |
                Should -Be '-y --profile minimal --default-toolchain stable --no-modify-path'
            @($script:processCalls | Where-Object {
                $_.ArgumentList.Count -gt 0 -and $_.ArgumentList[0] -eq 'default'
            }).Count | Should -Be 0
            @($script:processCalls | Where-Object {
                ($_.ArgumentList -join ' ') -eq 'run stable rustc --version'
            }).Count | Should -Be 1
            @($script:processCalls | Where-Object {
                ($_.ArgumentList -join ' ') -eq 'run stable cargo --version'
            }).Count | Should -Be 1
        }

        It 'uses the fixed Arm64 installer URL' {
            Mock Get-RunBobArchitecture { 'Arm64' }

            Invoke-RunBobBootstrap -ManifestPath $script:manifestPath

            Should -Invoke Invoke-WebRequest -Times 1 -Exactly -ParameterFilter {
                $Uri -eq 'https://static.rust-lang.org/rustup/dist/aarch64-pc-windows-msvc/rustup-init.exe'
            }
        }

        It 'rejects an unsupported architecture before download' {
            Mock Get-RunBobArchitecture { 'X86' }

            { Invoke-RunBobBootstrap -ManifestPath $script:manifestPath } |
                Should -Throw '*unsupported Windows architecture*X86*'
            Should -Invoke Invoke-WebRequest -Times 0 -Exactly
            Should -Invoke Invoke-RunBobExternalProcess -Times 0 -Exactly
        }

        It 'surfaces installer and Visual C++ guidance and still cleans up' {
            Mock Invoke-RunBobExternalProcess {
                return New-TestProcessResult -ExitCode 31 -StdErr 'link.exe is unavailable'
            }

            { Invoke-RunBobBootstrap -ManifestPath $script:manifestPath } |
                Should -Throw '*official Rust installer failed*Visual C++ Build Tools*link.exe is unavailable*'
            $script:downloadPath | Should -Not -BeNullOrEmpty
            (Test-Path -LiteralPath $script:downloadPath) | Should -BeFalse
        }

        It 'rejects an incomplete post-install toolchain' {
            Mock Invoke-RunBobExternalProcess {
                if ($ArgumentList -contains 'rustc' -and $ArgumentList -contains '--version') {
                    return New-TestProcessResult -StdOut 'rustc 1.76.0 (mock)'
                }
                if ($ArgumentList -contains 'cargo' -and $ArgumentList -contains '--version') {
                    return New-TestProcessResult -ExitCode 44 -StdErr 'cargo missing'
                }
                return New-TestProcessResult
            }

            { Invoke-RunBobBootstrap -ManifestPath $script:manifestPath } |
                Should -Throw '*complete Rust toolchain*cargo unavailable*cargo missing*'
        }

        It 'forwards exact cargo arguments through rustup stable' {
            $arguments = @('metadata', '--format-version', '1', '--filter-platform', 'target with spaces')

            Invoke-RunBobBootstrap -ManifestPath $script:manifestPath `
                -RunCargoSpecified -RunCargo $arguments

            $cargoCall = @($script:processCalls | Where-Object {
                $_.ArgumentList.Count -eq 8 -and
                ($_.ArgumentList[0..2] -join ' ') -eq 'run stable cargo'
            })
            $cargoCall.Count | Should -Be 1
            ($cargoCall[0].ArgumentList[3..7] -join "`u{1f}") |
                Should -Be ($arguments -join "`u{1f}")
        }
    }

    It 'installs stable through an existing rustup when rustc and cargo are absent' {
        Mock Get-RunBobCommandPath {
            if ($Name -eq 'rustup') { return 'C:\mock\rustup.exe' }
            return $null
        }
        Mock Invoke-RunBobExternalProcess {
            [void] $script:processCalls.Add([pscustomobject]@{
                FilePath = $FilePath
                ArgumentList = @($ArgumentList)
            })
            if (($ArgumentList -join ' ') -eq 'run stable rustc --version') {
                return New-TestProcessResult -StdOut 'rustc 1.76.0 (mock)'
            }
            if (($ArgumentList -join ' ') -eq 'run stable cargo --version') {
                return New-TestProcessResult -StdOut 'cargo 1.76.0 (mock)'
            }
            return New-TestProcessResult
        }

        Invoke-RunBobBootstrap -ManifestPath $script:manifestPath

        @($script:processCalls | Where-Object {
            ($_.ArgumentList -join ' ') -eq 'toolchain install stable --profile minimal'
        }).Count | Should -Be 1
        @($script:processCalls | Where-Object {
            ($_.ArgumentList -join ' ') -eq 'run stable rustc --version'
        }).Count | Should -Be 1
        @($script:processCalls | Where-Object {
            ($_.ArgumentList -join ' ') -eq 'run stable cargo --version'
        }).Count | Should -Be 1
        Should -Invoke Invoke-WebRequest -Times 0 -Exactly
    }

    It 'surfaces an existing rustup installation failure' {
        Mock Get-RunBobCommandPath {
            if ($Name -eq 'rustup') { return 'C:\mock\rustup.exe' }
            return $null
        }
        Mock Invoke-RunBobExternalProcess {
            return New-TestProcessResult -ExitCode 23 -StdErr 'toolchain download denied'
        }

        { Invoke-RunBobBootstrap -ManifestPath $script:manifestPath } |
            Should -Throw '*rustup could not install*toolchain download denied*'
    }

    It 'uses stable without changing the default for an old active rustup compiler' {
        $sysroot = Join-Path $TestDrive 'toolchains\old'
        $bin = Join-Path $sysroot 'bin'
        New-Item -ItemType Directory -Path $bin -Force | Out-Null
        $ownedRustc = Join-Path $bin 'rustc.exe'
        Set-Content -LiteralPath $ownedRustc -Value 'marker'
        Mock Get-RunBobCommandPath {
            switch ($Name) {
                'rustc' { 'C:\mock\rustc.exe' }
                'cargo' { 'C:\mock\cargo.exe' }
                'rustup' { 'C:\mock\rustup.exe' }
            }
        }
        Mock Invoke-RunBobExternalProcess {
            [void] $script:processCalls.Add([pscustomobject]@{
                FilePath = $FilePath
                ArgumentList = @($ArgumentList)
            })
            $joined = $ArgumentList -join ' '
            switch ($joined) {
                '--version' {
                    if ($FilePath -like '*rustc.exe') {
                        return New-TestProcessResult -StdOut 'rustc 1.70.0-nightly (mock)'
                    }
                    return New-TestProcessResult -StdOut 'cargo 1.70.0-nightly (mock)'
                }
                '--print sysroot' { return New-TestProcessResult -StdOut $sysroot }
                'which rustc' { return New-TestProcessResult -StdOut $ownedRustc }
                'run stable rustc --version' {
                    return New-TestProcessResult -StdOut 'rustc 1.76.0 (mock)'
                }
                'run stable cargo --version' {
                    return New-TestProcessResult -StdOut 'cargo 1.76.0 (mock)'
                }
                default { return New-TestProcessResult }
            }
        }

        Invoke-RunBobBootstrap -ManifestPath $script:manifestPath `
            -RunCargoSpecified -RunCargo @('test', '--locked')

        @($script:processCalls | Where-Object {
            ($_.ArgumentList -join ' ') -eq 'toolchain install stable --profile minimal'
        }).Count | Should -Be 1
        @($script:processCalls | Where-Object {
            ($_.ArgumentList -join ' ') -eq 'run stable cargo test --locked'
        }).Count | Should -Be 1
        @($script:processCalls | Where-Object {
            $_.ArgumentList.Count -gt 0 -and $_.ArgumentList[0] -eq 'default'
        }).Count | Should -Be 0
    }

    It 'refuses an old system compiler even when unrelated rustup is present' {
        $systemSysroot = Join-Path $TestDrive 'system-rust'
        $otherSysroot = Join-Path $TestDrive 'toolchains\unrelated'
        $otherBin = Join-Path $otherSysroot 'bin'
        New-Item -ItemType Directory -Path $systemSysroot, $otherBin -Force | Out-Null
        $otherRustc = Join-Path $otherBin 'rustc.exe'
        Set-Content -LiteralPath $otherRustc -Value 'marker'
        Mock Get-RunBobCommandPath {
            switch ($Name) {
                'rustc' { 'C:\system\rustc.exe' }
                'cargo' { 'C:\system\cargo.exe' }
                'rustup' { 'C:\other\rustup.exe' }
            }
        }
        Mock Invoke-RunBobExternalProcess {
            [void] $script:processCalls.Add([pscustomobject]@{
                FilePath = $FilePath
                ArgumentList = @($ArgumentList)
            })
            $joined = $ArgumentList -join ' '
            if ($joined -eq '--version') {
                if ($FilePath -like '*rustc.exe') {
                    return New-TestProcessResult -StdOut 'rustc 1.74.0 (mock)'
                }
                return New-TestProcessResult -StdOut 'cargo 1.74.0 (mock)'
            }
            if ($joined -eq '--print sysroot') {
                return New-TestProcessResult -StdOut $systemSysroot
            }
            if ($joined -eq 'which rustc') {
                return New-TestProcessResult -StdOut $otherRustc
            }
            return New-TestProcessResult
        }

        { Invoke-RunBobBootstrap -ManifestPath $script:manifestPath } |
            Should -Throw '*active compiler is not rustup-owned*'
        @($script:processCalls | Where-Object {
            $_.ArgumentList -contains 'toolchain'
        }).Count | Should -Be 0
    }

    It 'refuses a partial unrelated toolchain without invoking installation' {
        Mock Get-RunBobCommandPath {
            switch ($Name) {
                'rustc' { 'C:\system\rustc.exe' }
                default { $null }
            }
        }
        Mock Invoke-RunBobExternalProcess { return New-TestProcessResult }

        { Invoke-RunBobBootstrap -ManifestPath $script:manifestPath } |
            Should -Throw '*partial non-rustup Rust toolchain*'
        Should -Invoke Invoke-RunBobExternalProcess -Times 0 -Exactly
    }

    It 'rejects an explicitly empty RunCargo request' {
        Mock Get-RunBobCommandPath { throw 'command discovery must not run' }

        { Invoke-RunBobBootstrap -ManifestPath $script:manifestPath `
                -RunCargoSpecified -RunCargo @() } | Should -Throw '*Usage:*'
        { Invoke-RunBobBootstrap -ManifestPath $script:manifestPath `
                -RunCargoSpecified -RunCargo @('') } | Should -Throw '*Usage:*'
        Should -Invoke Get-RunBobCommandPath -Times 0 -Exactly
    }

    It 'rejects a bare positional argument at script binding before main' {
        $entryScript = Join-Path $PSScriptRoot '..\scripts\bootstrap-rust.ps1'
        Mock Get-RunBobCommandPath { throw 'command discovery must not run' }
        Mock Invoke-RunBobBootstrap { throw 'bootstrap main must not run' }

        # Dot-sourcing still exercises the script entry parameter binder while
        # containing a regression safely: accepted input cannot run the guarded main.
        { . $entryScript build } | Should -Throw '*positional parameter*'
        Should -Invoke Get-RunBobCommandPath -Times 0 -Exactly
        Should -Invoke Invoke-RunBobBootstrap -Times 0 -Exactly
    }

    Context 'Cargo-home tool reuse' {
        BeforeEach {
            $env:CARGO_HOME = Join-Path $script:caseRoot 'cargo home with spaces'
            $env:USERPROFILE = Join-Path $script:caseRoot 'profile that must not win'
        }

        It 'uses a complete Cargo-home pair directly when command discovery is absent' {
            $tools = New-TestCargoHomeTools -CargoHome $env:CARGO_HOME -Rustc -Cargo -Rustup
            Mock Get-RunBobCommandPath { $null }
            Mock Invoke-RunBobExternalProcess {
                [void] $script:processCalls.Add([pscustomobject]@{
                    FilePath = $FilePath
                    ArgumentList = @($ArgumentList)
                })
                if ($ArgumentList.Count -eq 1 -and $ArgumentList[0] -eq '--version') {
                    if ($FilePath -eq $tools.Rustc) {
                        return New-TestProcessResult -StdOut 'rustc 1.75.0 (mock)'
                    }
                    return New-TestProcessResult -StdOut 'cargo 1.75.0 (mock)'
                }
                return New-TestProcessResult
            }

            $arguments = @('check', '--locked', '--message-format', 'json with spaces')
            Invoke-RunBobBootstrap -ManifestPath $script:manifestPath `
                -RunCargoSpecified -RunCargo $arguments

            $cargoCalls = @($script:processCalls | Where-Object {
                $_.FilePath -eq $tools.Cargo -and $_.ArgumentList.Count -eq 4
            })
            $cargoCalls.Count | Should -Be 1
            ($cargoCalls[0].ArgumentList -join "`u{1f}") |
                Should -Be ($arguments -join "`u{1f}")
            @($script:processCalls | Where-Object {
                $_.ArgumentList -contains 'toolchain'
            }).Count | Should -Be 0
            Should -Invoke Invoke-WebRequest -Times 0 -Exactly
        }

        It 'falls back explicitly to USERPROFILE dot-cargo when CARGO_HOME is empty' {
            $env:CARGO_HOME = ''
            $profileCargoHome = Join-Path $env:USERPROFILE '.cargo'
            $tools = New-TestCargoHomeTools -CargoHome $profileCargoHome -Rustc -Cargo
            Mock Get-RunBobCommandPath { $null }
            Mock Invoke-RunBobExternalProcess {
                [void] $script:processCalls.Add([pscustomobject]@{
                    FilePath = $FilePath
                    ArgumentList = @($ArgumentList)
                })
                if ($FilePath -eq $tools.Rustc) {
                    return New-TestProcessResult -StdOut 'rustc 1.75.0 (mock)'
                }
                return New-TestProcessResult -StdOut 'cargo 1.75.0 (mock)'
            }

            Invoke-RunBobBootstrap -ManifestPath $script:manifestPath

            @($script:processCalls | Where-Object {
                $_.FilePath -eq $tools.Rustc -and
                ($_.ArgumentList -join ' ') -eq '--version'
            }).Count | Should -Be 2
            @($script:processCalls | Where-Object {
                $_.FilePath -eq $tools.Cargo -and
                ($_.ArgumentList -join ' ') -eq '--version'
            }).Count | Should -Be 2
            Should -Invoke Invoke-WebRequest -Times 0 -Exactly
        }

        It 'uses Cargo-home rustup when it is the only installed Rust command' {
            $tools = New-TestCargoHomeTools -CargoHome $env:CARGO_HOME -Rustup
            Mock Get-RunBobCommandPath { $null }
            Mock Invoke-RunBobExternalProcess {
                [void] $script:processCalls.Add([pscustomobject]@{
                    FilePath = $FilePath
                    ArgumentList = @($ArgumentList)
                })
                if (($ArgumentList -join ' ') -eq 'run stable rustc --version') {
                    return New-TestProcessResult -StdOut 'rustc 1.76.0 (mock)'
                }
                if (($ArgumentList -join ' ') -eq 'run stable cargo --version') {
                    return New-TestProcessResult -StdOut 'cargo 1.76.0 (mock)'
                }
                return New-TestProcessResult
            }

            Invoke-RunBobBootstrap -ManifestPath $script:manifestPath

            @($script:processCalls | Where-Object {
                $_.FilePath -eq $tools.Rustup -and
                ($_.ArgumentList -join ' ') -eq 'toolchain install stable --profile minimal'
            }).Count | Should -Be 1
            Should -Invoke Invoke-WebRequest -Times 0 -Exactly
        }

        It 'repairs an empty Cargo-home rustup toolchain through its matching proxies' {
            $tools = New-TestCargoHomeTools -CargoHome $env:CARGO_HOME -Rustc -Cargo -Rustup
            Mock Get-RunBobCommandPath { $null }
            Mock Invoke-RunBobExternalProcess {
                [void] $script:processCalls.Add([pscustomobject]@{
                    FilePath = $FilePath
                    ArgumentList = @($ArgumentList)
                })
                $joined = $ArgumentList -join ' '
                if ($joined -eq '--version') {
                    return New-TestProcessResult -ExitCode 1 -StdErr 'no default toolchain configured'
                }
                if ($joined -eq '--print sysroot' -or $joined -eq 'which rustc') {
                    return New-TestProcessResult -ExitCode 1 -StdErr 'no active toolchain'
                }
                if ($joined -eq 'run stable rustc --version') {
                    return New-TestProcessResult -StdOut 'rustc 1.80.0 (mock)'
                }
                if ($joined -eq 'run stable cargo --version') {
                    return New-TestProcessResult -StdOut 'cargo 1.80.0 (mock)'
                }
                return New-TestProcessResult
            }

            Invoke-RunBobBootstrap -ManifestPath $script:manifestPath `
                -RunCargoSpecified -RunCargo @('check', '--locked')

            @($script:processCalls | Where-Object {
                $_.FilePath -eq $tools.Rustup -and
                ($_.ArgumentList -join ' ') -eq 'toolchain install stable --profile minimal'
            }).Count | Should -Be 1
            @($script:processCalls | Where-Object {
                $_.FilePath -eq $tools.Rustup -and
                ($_.ArgumentList -join ' ') -eq 'run stable cargo check --locked'
            }).Count | Should -Be 1
            Should -Invoke Invoke-WebRequest -Times 0 -Exactly
        }

        It 'rejects a broken Cargo-home pair that does not match its rustup executable' {
            $tools = New-TestCargoHomeTools -CargoHome $env:CARGO_HOME -Rustc -Cargo -Rustup
            Set-Content -LiteralPath $tools.Rustc -Value 'unrelated rustc executable'
            Set-Content -LiteralPath $tools.Cargo -Value 'unrelated cargo executable'
            Set-Content -LiteralPath $tools.Rustup -Value 'rustup executable'
            Mock Get-RunBobCommandPath { $null }
            Mock Invoke-RunBobExternalProcess {
                return New-TestProcessResult -ExitCode 1 -StdErr 'broken tool'
            }

            { Invoke-RunBobBootstrap -ManifestPath $script:manifestPath } |
                Should -Throw '*rustc unavailable*broken tool*'
            Should -Invoke Invoke-RunBobExternalProcess -ParameterFilter {
                $ArgumentList -contains 'toolchain'
            } -Times 0 -Exactly
            Should -Invoke Invoke-WebRequest -Times 0 -Exactly
        }

        It 'reuses files from a first official install on the second invocation' {
            Mock Get-RunBobCommandPath { $null }
            Mock Get-RunBobArchitecture { 'X64' }
            Mock New-RunBobInstallerPath {
                Join-Path $TestDrive ('run-bob-rustup-{0}.exe' -f [guid]::NewGuid())
            }
            Mock Invoke-WebRequest {
                $script:downloadPath = $OutFile
                Set-Content -LiteralPath $OutFile -Value 'mock installer'
            }
            Mock Invoke-RunBobExternalProcess {
                [void] $script:processCalls.Add([pscustomobject]@{
                    FilePath = $FilePath
                    ArgumentList = @($ArgumentList)
                })
                if ($FilePath -match 'run-bob-rustup-[0-9a-f-]+\.exe$') {
                    [void] (New-TestCargoHomeTools -CargoHome $env:CARGO_HOME -Rustc -Cargo -Rustup)
                    return New-TestProcessResult
                }
                if ($ArgumentList -contains '--version') {
                    if ($FilePath -like '*rustc.exe' -or $ArgumentList -contains 'rustc') {
                        return New-TestProcessResult -StdOut 'rustc 1.76.0 (mock)'
                    }
                    return New-TestProcessResult -StdOut 'cargo 1.76.0 (mock)'
                }
                return New-TestProcessResult
            }

            Invoke-RunBobBootstrap -ManifestPath $script:manifestPath
            Invoke-RunBobBootstrap -ManifestPath $script:manifestPath `
                -RunCargoSpecified -RunCargo @('metadata', '--locked')

            Should -Invoke Invoke-WebRequest -Times 1 -Exactly
            @($script:processCalls | Where-Object {
                $_.FilePath -match 'run-bob-rustup-[0-9a-f-]+\.exe$'
            }).Count | Should -Be 1
            @($script:processCalls | Where-Object {
                $_.ArgumentList -contains 'toolchain'
            }).Count | Should -Be 0
            @($script:processCalls | Where-Object {
                $_.FilePath -like '*cargo home with spaces*' -and
                ($_.ArgumentList -join ' ') -eq 'metadata --locked'
            }).Count | Should -Be 1
        }

        It 'rejects mixed single tools from command discovery and Cargo home' {
            $tools = New-TestCargoHomeTools -CargoHome $env:CARGO_HOME -Cargo
            Mock Get-RunBobCommandPath {
                if ($Name -eq 'rustc') { return 'C:\command tools\rustc.exe' }
                return $null
            }
            Mock Invoke-RunBobExternalProcess { return New-TestProcessResult }

            { Invoke-RunBobBootstrap -ManifestPath $script:manifestPath } |
                Should -Throw '*partial non-rustup Rust toolchain*'
            $tools.Cargo | Should -Exist
            Should -Invoke Invoke-RunBobExternalProcess -Times 0 -Exactly
            Should -Invoke Invoke-WebRequest -Times 0 -Exactly
        }

        It 'prefers a complete Cargo-home pair over a partial command pair' {
            $tools = New-TestCargoHomeTools -CargoHome $env:CARGO_HOME -Rustc -Cargo
            Mock Get-RunBobCommandPath {
                if ($Name -eq 'rustc') { return 'C:\unrelated\rustc.exe' }
                return $null
            }
            Mock Invoke-RunBobExternalProcess {
                [void] $script:processCalls.Add([pscustomobject]@{
                    FilePath = $FilePath
                    ArgumentList = @($ArgumentList)
                })
                if ($FilePath -eq $tools.Rustc) {
                    return New-TestProcessResult -StdOut 'rustc 1.75.0 (mock)'
                }
                return New-TestProcessResult -StdOut 'cargo 1.75.0 (mock)'
            }

            Invoke-RunBobBootstrap -ManifestPath $script:manifestPath `
                -RunCargoSpecified -RunCargo @('check', '--locked')

            @($script:processCalls | Where-Object {
                $_.FilePath -eq 'C:\unrelated\rustc.exe'
            }).Count | Should -Be 0
            @($script:processCalls | Where-Object {
                $_.FilePath -eq $tools.Cargo -and
                ($_.ArgumentList -join ' ') -eq 'check --locked'
            }).Count | Should -Be 1
        }

        It 'repairs a Cargo-home rustc proxy with owned rustup when cargo is missing' {
            $tools = New-TestCargoHomeTools -CargoHome $env:CARGO_HOME -Rustc -Rustup
            $sysroot = Join-Path $TestDrive 'toolchain sysroot with spaces'
            $sysrootBin = Join-Path $sysroot 'bin'
            New-Item -ItemType Directory -Path $sysrootBin -Force | Out-Null
            $ownedRustc = Join-Path $sysrootBin 'rustc.exe'
            Set-Content -LiteralPath $ownedRustc -Value 'compiler'
            Mock Get-RunBobCommandPath { $null }
            Mock Invoke-RunBobExternalProcess {
                [void] $script:processCalls.Add([pscustomobject]@{
                    FilePath = $FilePath
                    ArgumentList = @($ArgumentList)
                })
                switch ($ArgumentList -join ' ') {
                    '--print sysroot' { return New-TestProcessResult -StdOut $sysroot }
                    'which rustc' { return New-TestProcessResult -StdOut $ownedRustc }
                    'run stable rustc --version' {
                        return New-TestProcessResult -StdOut 'rustc 1.76.0 (mock)'
                    }
                    'run stable cargo --version' {
                        return New-TestProcessResult -StdOut 'cargo 1.76.0 (mock)'
                    }
                    default { return New-TestProcessResult }
                }
            }

            Invoke-RunBobBootstrap -ManifestPath $script:manifestPath

            @($script:processCalls | Where-Object {
                $_.FilePath -eq $tools.Rustc -and
                ($_.ArgumentList -join ' ') -eq '--print sysroot'
            }).Count | Should -Be 1
            @($script:processCalls | Where-Object {
                $_.FilePath -eq $tools.Rustup -and
                ($_.ArgumentList -join ' ') -eq 'toolchain install stable --profile minimal'
            }).Count | Should -Be 1
        }

        It 'uses the selected home compiler for ownership instead of an unrelated command rustc' {
            $tools = New-TestCargoHomeTools -CargoHome $env:CARGO_HOME -Rustc -Cargo -Rustup
            $sysroot = Join-Path $TestDrive 'selected home sysroot'
            $sysrootBin = Join-Path $sysroot 'bin'
            New-Item -ItemType Directory -Path $sysrootBin -Force | Out-Null
            $ownedRustc = Join-Path $sysrootBin 'rustc.exe'
            Set-Content -LiteralPath $ownedRustc -Value 'compiler'
            Mock Get-RunBobCommandPath {
                if ($Name -eq 'rustc') { return 'C:\unrelated mirror\rustc.exe' }
                return $null
            }
            Mock Invoke-RunBobExternalProcess {
                [void] $script:processCalls.Add([pscustomobject]@{
                    FilePath = $FilePath
                    ArgumentList = @($ArgumentList)
                })
                $joined = $ArgumentList -join ' '
                if ($joined -eq '--version') {
                    if ($FilePath -eq $tools.Rustc) {
                        return New-TestProcessResult -StdOut 'rustc 1.74.0 (mock)'
                    }
                    return New-TestProcessResult -StdOut 'cargo 1.74.0 (mock)'
                }
                switch ($joined) {
                    '--print sysroot' { return New-TestProcessResult -StdOut $sysroot }
                    'which rustc' { return New-TestProcessResult -StdOut $ownedRustc }
                    'run stable rustc --version' {
                        return New-TestProcessResult -StdOut 'rustc 1.76.0 (mock)'
                    }
                    'run stable cargo --version' {
                        return New-TestProcessResult -StdOut 'cargo 1.76.0 (mock)'
                    }
                    default { return New-TestProcessResult }
                }
            }

            Invoke-RunBobBootstrap -ManifestPath $script:manifestPath

            @($script:processCalls | Where-Object {
                $_.FilePath -eq 'C:\unrelated mirror\rustc.exe'
            }).Count | Should -Be 0
            @($script:processCalls | Where-Object {
                $_.FilePath -eq $tools.Rustc -and
                ($_.ArgumentList -join ' ') -eq '--print sysroot'
            }).Count | Should -Be 1
            @($script:processCalls | Where-Object {
                $_.FilePath -eq $tools.Rustup -and
                ($_.ArgumentList -join ' ') -eq 'toolchain install stable --profile minimal'
            }).Count | Should -Be 1
        }

        It 'rejects an unowned selected home pair despite an owned unrelated command rustc' {
            $tools = New-TestCargoHomeTools -CargoHome $env:CARGO_HOME -Rustc -Cargo -Rustup
            $ownedSysroot = Join-Path $TestDrive 'unrelated command sysroot'
            $ownedBin = Join-Path $ownedSysroot 'bin'
            $unownedSysroot = Join-Path $TestDrive 'selected but unowned sysroot'
            New-Item -ItemType Directory -Path $ownedBin, $unownedSysroot -Force | Out-Null
            $rustupCompiler = Join-Path $ownedBin 'rustc.exe'
            Set-Content -LiteralPath $rustupCompiler -Value 'compiler'
            Mock Get-RunBobCommandPath {
                if ($Name -eq 'rustc') { return 'C:\unrelated mirror\rustc.exe' }
                return $null
            }
            Mock Invoke-RunBobExternalProcess {
                [void] $script:processCalls.Add([pscustomobject]@{
                    FilePath = $FilePath
                    ArgumentList = @($ArgumentList)
                })
                $joined = $ArgumentList -join ' '
                if ($joined -eq '--version') {
                    if ($FilePath -eq $tools.Rustc) {
                        return New-TestProcessResult -StdOut 'rustc 1.74.0 (mock)'
                    }
                    return New-TestProcessResult -StdOut 'cargo 1.74.0 (mock)'
                }
                if ($joined -eq '--print sysroot') {
                    if ($FilePath -eq $tools.Rustc) {
                        return New-TestProcessResult -StdOut $unownedSysroot
                    }
                    return New-TestProcessResult -StdOut $ownedSysroot
                }
                if ($joined -eq 'which rustc') {
                    return New-TestProcessResult -StdOut $rustupCompiler
                }
                return New-TestProcessResult
            }

            { Invoke-RunBobBootstrap -ManifestPath $script:manifestPath } |
                Should -Throw '*active compiler is not rustup-owned*'

            @($script:processCalls | Where-Object {
                $_.FilePath -eq 'C:\unrelated mirror\rustc.exe'
            }).Count | Should -Be 0
            @($script:processCalls | Where-Object {
                $_.FilePath -eq $tools.Rustc -and
                ($_.ArgumentList -join ' ') -eq '--print sysroot'
            }).Count | Should -Be 1
            @($script:processCalls | Where-Object {
                $_.ArgumentList -contains 'toolchain'
            }).Count | Should -Be 0
        }
    }
}

Describe 'bootstrap-rust.ps1 process exit boundary' -Skip:(-not $IsWindows) {
    It 'returns zero after success even when the caller starts with a nonzero LASTEXITCODE' {
        $entryScript = (Resolve-Path (Join-Path $PSScriptRoot '..\scripts\bootstrap-rust.ps1')).ProviderPath
        $shell = (Get-Process -Id $PID).Path
        $isolatedCargoHome = Join-Path $TestDrive 'isolated process cargo home'
        $isolatedBin = Join-Path $isolatedCargoHome 'bin'
        New-Item -ItemType Directory -Path $isolatedBin -Force | Out-Null
        foreach ($toolName in @('rustc', 'cargo')) {
            $tool = Get-Command -Name $toolName -CommandType Application -ErrorAction Stop |
                Select-Object -First 1
            Copy-Item -LiteralPath $tool.Source -Destination (Join-Path $isolatedBin "$toolName.exe")
        }
        $quotedCargoHome = $isolatedCargoHome.Replace("'", "''")
        $quotedBin = $isolatedBin.Replace("'", "''")
        $quotedEntryScript = $entryScript.Replace("'", "''")
        $wrapperPath = Join-Path $TestDrive 'exit-boundary.ps1'
        @"
`$env:CARGO_HOME = '$quotedCargoHome'
`$env:Path = '$quotedBin;' + `$env:SystemRoot + '\System32;' + `$env:SystemRoot
& cmd.exe /c exit 23
& '$quotedEntryScript'
exit `$LASTEXITCODE
"@ | Set-Content -LiteralPath $wrapperPath

        $process = Start-Process -FilePath $shell -ArgumentList @(
            '-NoLogo', '-NoProfile', '-NonInteractive', '-File', $wrapperPath
        ) -Wait -PassThru -NoNewWindow

        $process.ExitCode | Should -Be 0
    }
}
