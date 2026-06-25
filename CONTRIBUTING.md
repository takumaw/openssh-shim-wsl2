# CONTRIBUTING

## Development requirements

* Windows 10 x64 or Windows 11 x64
* Latest stable Rust toolchain
* PowerShell
* WSL2 for manual integration testing

This project uses Rust edition 2024 and does not declare an explicit MSRV for 0.1.0.

## Build

```powershell
cargo build --release
```

Expected release binaries:

```text
target\release\ssh.exe
target\release\scp.exe
target\release\sftp.exe
```

## Test

CI focuses on unit tests and release builds. End-to-end tests requiring a real SSH server or a configured WSL2 distribution are not required in CI.

### Unit tests

Most tests are Rust unit tests located in `src/`. You can run them using:

```powershell
cargo test
```

### Smoke tests

You can run the binary smoke test locally to verify that the built executables (`ssh.exe`, `scp.exe`, and `sftp.exe`) can launch successfully and exit with `255` (since WSL is not fully configured or invoked directly in this test environment):

```powershell
./tests/smoke_test.ps1
```

## Source layout

```text
src/
  bin/
    ssh.rs
    scp.rs
    sftp.rs
  lib.rs
  config.rs
  debug.rs
  env.rs
  errors.rs
  launcher.rs
  login_shell.rs
  openssh_args.rs
  path_convert.rs
  shell_quote.rs
  tool.rs
  wsl.rs
```

The binaries under `src/bin/` must remain thin entry points. Shared implementation belongs in the library crate.

## Implementation rules

* Do not implement SSH, SCP, or SFTP protocols.
* Do not call `cmd.exe` or PowerShell.
* Launch `C:\Windows\System32\wsl.exe` directly.
* Use `wsl.exe --cd <converted-cwd> --exec ...` by default.
* Do not use shell mode unless explicitly requested by `--wsl2-use-shell`, `--wsl2-shell`, or `--wsl2-use-login-shell`.
* Remove shim-specific `--wsl2-*` options before passing arguments to WSL OpenSSH.
* Preserve stdin, stdout, and stderr behavior.
* Return 255 for shim configuration, conversion, or launch errors.
* Return the `wsl.exe` exit code after `wsl.exe` has started.
* Keep UNC paths unsupported unless the design is explicitly revised.
* Do not add ssh-agent bridging in 0.1.0.
* Do not add a config file in 0.1.0.

## Documentation

Design changes must update `docs/DESIGN.md`. Release process changes must update `docs/RELEASE.md`.
