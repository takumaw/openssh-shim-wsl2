# DESIGN

## Purpose

`openssh-shim-wsl2` provides Windows-native `ssh.exe`, `scp.exe`, and `sftp.exe` launchers that run OpenSSH inside WSL2.

The project is intended for Windows shells such as PowerShell and Command Prompt. VS Code Remote-SSH is supported by configuring `remote.SSH.path` to the installed `ssh.exe`.

## Executables

```text
ssh.exe  -> wsl.exe ... --exec ssh  ...
scp.exe  -> wsl.exe ... --exec scp  ...
sftp.exe -> wsl.exe ... --exec sftp ...
```

The Windows executables are shims. They do not implement SSH, SCP, or SFTP protocols.

## WSL distribution selection

By default, the shim uses the default WSL distribution.

Distribution can be selected using:

```text
--wsl2-distribution <name>
OPENSSH_SHIM_WSL2_DISTRIBUTION
```

Command-line options take precedence over environment variables.

## Option precedence

```text
1. command-line --wsl2-* options
2. OPENSSH_SHIM_WSL2_* environment variables
3. defaults
```

Shim-specific options are removed before arguments are passed to WSL OpenSSH.

## Boolean environment variables

True values:

```text
1, true, yes, on
```

False values:

```text
0, false, no, off, empty, unset
```

Invalid boolean values are shim errors and exit 255.

## Working directory

The shim converts the Windows current directory to a WSL path and passes it to `wsl.exe --cd`.

Example:

```text
C:\Users\you\project -> /mnt/c/Users/you/project
```

If current directory conversion fails, the shim exits 255.

## Path conversion

The shim converts known path-valued arguments from Windows syntax to WSL syntax.

Supported conversions:

```text
C:\a\b        -> /mnt/c/a/b
C:/a/b        -> /mnt/c/a/b
\\?\C:\a\b    -> /mnt/c/a/b
.\a\b         -> ./a/b
..\a\b        -> ../a/b
```

Paths beginning with `~` are not converted. They are WSL/Linux paths. Users who want to refer to their Windows home directory should pass an expanded Windows path from the Windows shell.

Remote specs are not converted:

```text
host:/path
user@host:/path
scp://...
sftp://...
```

UNC paths are unsupported and are shim errors:

```text
\\server\share\path
\\?\UNC\server\share\path
```

The exit code for this error is 255.

## Path conversion backend

Default path conversion is built in.

`--wsl2-use-wslpath` or `OPENSSH_SHIM_WSL2_USE_WSLPATH` enables a `wslpath`-based backend. The backend is opt-in because it requires additional WSL execution and may be slower.

## SSH argument parsing

The shim is not a complete OpenSSH option parser. It implements a focused allowlist for common path-valued options.

Path-valued options include:

```text
-F <configfile>
-i <identity_file>
-E <log_file>
-o CertificateFile=<path>
-o ControlPath=<path>
-o GlobalKnownHostsFile=<path>
-o IdentityFile=<path>
-o Include=<path>
-o UserKnownHostsFile=<path>
```

The following command-valued options are not rewritten:

```text
ProxyCommand
LocalCommand
KnownHostsCommand
RemoteCommand
```

Commands executed by WSL OpenSSH are Linux commands executed inside WSL2. Windows commands are not translated.

## scp.exe

`scp.exe` converts local source and destination paths where they are Windows paths. Remote specs are not converted.

`-S` is not converted. It is interpreted by WSL `scp` as the SSH program to run inside WSL2.

## sftp.exe

`sftp.exe` converts:

```text
-b <batchfile>
-F <configfile>
-i <identity_file>
-o <path-valued ssh option>
```

`sftp.exe` positional arguments are not converted. They are interpreted by WSL `sftp`.

The path to a batch file passed with `-b` is converted. Paths inside the batch file are not rewritten.

`-S` is not converted.

## Shell mode

Default mode does not use a shell:

```text
wsl.exe [--distribution D] --cd <cwd> --exec <tool> <args...>
```

Shell mode is enabled by any of:

```text
--wsl2-use-shell
--wsl2-shell <shell>
--wsl2-use-login-shell
OPENSSH_SHIM_WSL2_USE_SHELL
OPENSSH_SHIM_WSL2_SHELL
OPENSSH_SHIM_WSL2_USE_LOGIN_SHELL
```

Shell mode executes:

```text
wsl.exe [--distribution D] --cd <cwd> --exec <shell> -lc 'exec <tool> ...'
```

Default shell for shell mode is `/bin/sh`.

`--wsl2-shell <shell>` uses the specified shell and implies shell mode.

`--wsl2-use-login-shell` resolves the WSL user's login shell from `/etc/passwd` and implies shell mode. If resolution fails, `/bin/sh` is used.

Conflict policy:

```text
CLI:
  --wsl2-shell and --wsl2-use-login-shell together are an error, exit 255.

Env:
  OPENSSH_SHIM_WSL2_SHELL wins over OPENSSH_SHIM_WSL2_USE_LOGIN_SHELL.

CLI > env > default.
```

Shell mode is best-effort because shell option semantics differ across shells.

## SSH config, keys, known_hosts, and agent

The shim uses WSL OpenSSH. Therefore, WSL-side files and services are used:

```text
~/.ssh/config
~/.ssh/known_hosts
~/.ssh/id_*
WSL-side ssh-agent
```

Windows OpenSSH config, keys, known_hosts, and Windows ssh-agent are not used automatically.

SSH agent bridging is out of scope for 0.1.0.

## Debug output

`--wsl2-debug` and `OPENSSH_SHIM_WSL2_DEBUG` enable debug output to stderr.

The format intentionally resembles OpenSSH verbose output while clearly identifying the shim:

```text
debug1: openssh-shim-wsl2: tool: ssh
debug1: openssh-shim-wsl2: distribution: Ubuntu-24.04
debug1: openssh-shim-wsl2: cwd: C:\Users\me -> /mnt/c/Users/me
debug1: openssh-shim-wsl2: path conversion: builtin
debug1: openssh-shim-wsl2: shell mode: disabled
debug1: openssh-shim-wsl2: launching wsl.exe with 9 arguments
debug1: openssh-shim-wsl2: argv[0]: C:\Windows\System32\wsl.exe
```

Debug output includes the generated command and full argv. Users enable this intentionally; host names, paths, and command-line arguments may appear in debug output.

Only debug level 1 exists in 0.1.0.

## Errors and exit codes

Shim configuration, conversion, and launch errors are written to stderr and exit 255.

After `wsl.exe` has started, the shim returns the `wsl.exe` exit code.

## WSL networking

OpenSSH local forwarding options, such as `-D`, bind inside the WSL2 environment. Windows applications must be able to connect to services bound from WSL2. This depends on WSL networking configuration.

## Non-goals for 0.1.0

* SSH protocol implementation.
* Complete OpenSSH option parser.
* Windows ssh-agent bridging.
* Windows OpenSSH configuration reuse.
* UNC path support.
* Rewriting paths inside `sftp -b` batch files.
* Additional utilities such as `ssh-add`, `ssh-agent`, `ssh-keygen`, or `ssh-keyscan`.
* Config file support.
