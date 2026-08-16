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
}

Describe 'bootstrap-rust.ps1' {
    BeforeEach {
        $script:manifestPath = Join-Path $TestDrive 'Cargo.toml'
        @'
[package]
name = "run-bob"
version = "0.0.0"
rust-version = "1.75"
'@ | Set-Content -LiteralPath $script:manifestPath
        $script:processCalls = [System.Collections.Generic.List[object]]::new()
        $script:downloadPath = $null
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
        Mock Invoke-WebRequest { throw 'network must not be used' }
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
}
