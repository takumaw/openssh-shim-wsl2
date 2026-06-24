# openssh-shim-wsl2

WSL2 OpenSSH shims for Windows shells.

This project provides Windows-native `ssh.exe`, `scp.exe`, and `sftp.exe` shims that run OpenSSH inside WSL2. It is intended for use from Windows shells such as PowerShell and Command Prompt, and it can also be used as the SSH executable for VS Code Remote-SSH.

(C) 2026 Takuma WATANABE

## Requirements

* Windows 10 x64 or Windows 11 x64
* WSL2
* A recent `wsl.exe` with `--cd` support
* A WSL2 distribution with `ssh`, `scp`, and `sftp` available in `PATH`

The shims use the OpenSSH configuration inside WSL2. That means `~/.ssh/config`, `~/.ssh/known_hosts`, SSH keys, and `ssh-agent` are the WSL-side files and services, not the Windows OpenSSH files and services.

## Installation

### Manual installation

1. Open the latest release page:

   <https://github.com/takumaw/openssh-shim-wsl2/releases/latest>

2. Download the zip asset named like this:

   ```text
   openssh-shim-wsl2-vX.Y.Z-x86_64-pc-windows-msvc.zip
   ```

3. Extract it to:

   ```text
   %LOCALAPPDATA%\Programs\openssh-shim-wsl2\
   ```

   After extraction, the directory should contain:

   ```text
   ssh.exe
   scp.exe
   sftp.exe
   README.md
   LICENSE.txt
   ```

### Script installation

This script downloads the latest versioned zip asset from GitHub Releases and extracts it to the recommended per-user install directory.

```powershell
$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\openssh-shim-wsl2"
$TempZip = Join-Path $env:TEMP "openssh-shim-wsl2.zip"
$ApiUrl = "https://api.github.com/repos/takumaw/openssh-shim-wsl2/releases/latest"

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

$Release = Invoke-RestMethod -Uri $ApiUrl
$Asset = $Release.assets |
    Where-Object { $_.name -like "openssh-shim-wsl2-v*-x86_64-pc-windows-msvc.zip" } |
    Select-Object -First 1

if (-not $Asset) {
    throw "Release asset not found."
}

Invoke-WebRequest -Uri $Asset.browser_download_url -OutFile $TempZip
Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force
Remove-Item $TempZip -Force
```

## VS Code Remote-SSH

Configure VS Code to use the extracted `ssh.exe`.

```json
{
  "remote.SSH.path": "C:\\Users\\<you>\\AppData\\Local\\Programs\\openssh-shim-wsl2\\ssh.exe"
}
```

`scp.exe` must be installed in the same directory as `ssh.exe`. VS Code Remote-SSH may look for `scp.exe` next to the configured SSH executable when it needs to transfer server assets from the local client.

## Usage

Basic usage mirrors OpenSSH command names:

```powershell
ssh.exe host
scp.exe C:\Users\you\file.txt host:/tmp/file.txt
sftp.exe host
```

By default, the selected WSL2 distribution is the default WSL distribution. To select a distribution:

```powershell
ssh.exe --wsl2-distribution Ubuntu-24.04 host
```

Or use an environment variable:

```powershell
$env:OPENSSH_SHIM_WSL2_DISTRIBUTION = "Ubuntu-24.04"
ssh.exe host
```

Command-line options take precedence over environment variables.

## Shim options

Shim-specific options are removed before arguments are passed to WSL OpenSSH.

```text
--wsl2-distribution <name>
--wsl2-use-wslpath
--wsl2-use-shell
--wsl2-shell <shell>
--wsl2-use-login-shell
--wsl2-debug
```

Corresponding environment variables:

```text
OPENSSH_SHIM_WSL2_DISTRIBUTION
OPENSSH_SHIM_WSL2_USE_WSLPATH
OPENSSH_SHIM_WSL2_USE_SHELL
OPENSSH_SHIM_WSL2_SHELL
OPENSSH_SHIM_WSL2_USE_LOGIN_SHELL
OPENSSH_SHIM_WSL2_DEBUG
```

Boolean environment variables treat the following values as true:

```text
1, true, yes, on
```

They treat the following values as false:

```text
0, false, no, off, empty, unset
```

## Path conversion

The shims perform Windows-to-WSL path conversion for known path-valued arguments.

Examples:

```text
C:\Users\you\file.txt  ->  /mnt/c/Users/you/file.txt
C:/Users/you/file.txt   ->  /mnt/c/Users/you/file.txt
.\file.txt              ->  ./file.txt
..\file.txt             ->  ../file.txt
```

Paths beginning with `~` are not converted. They are interpreted by WSL OpenSSH as WSL/Linux paths. To refer to the Windows home directory, pass an expanded Windows path from PowerShell or Command Prompt.

UNC paths are not supported. Passing a UNC path such as `\\server\share\file.txt` exits with status 255.

Remote specs such as `host:/tmp/file.txt` and `user@host:/tmp/file.txt` are not converted.

## Shell mode

By default, the shims run OpenSSH directly through `wsl.exe --exec` without a shell.

Use shell mode only when you need shell initialization behavior.

```powershell
ssh.exe --wsl2-use-shell host
ssh.exe --wsl2-shell /bin/bash host
ssh.exe --wsl2-use-login-shell host
```

Shell mode is best-effort and may change argument semantics. The default shell for shell mode is `/bin/sh`. `--wsl2-shell` selects a specific shell. `--wsl2-use-login-shell` resolves the WSL user's login shell from `/etc/passwd`; if resolution fails, `/bin/sh` is used.

## WSL networking

VS Code Remote-SSH and OpenSSH options such as `-D` use local port forwarding. Because this shim runs OpenSSH inside WSL2, local forwarding ports are opened from the WSL2 environment. Make sure your WSL networking mode allows Windows applications to connect to services bound from WSL2.

## WSL startup cost

The first invocation may be slower if the selected WSL2 distribution is not already running. Starting the distribution once before connecting can reduce cold-start delay.

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Design

See [docs/DESIGN.md](docs/DESIGN.md).

## License

Apache-2.0. See [LICENSE.txt](LICENSE.txt).
