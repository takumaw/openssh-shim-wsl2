# RELEASE

## Overview

Releases are driven by Git tags. Release notes are maintained in `HISTORY.md`.

## Version format

```text
HISTORY.md heading:  ## X.Y.Z
Git tag:             vX.Y.Z
Zip asset:           openssh-shim-wsl2-vX.Y.Z-x86_64-pc-windows-msvc.zip
```

## Release assets

Each release contains:

```text
openssh-shim-wsl2-vX.Y.Z-x86_64-pc-windows-msvc.zip
SHA256SUMS.txt
```

The zip contains:

```text
ssh.exe
scp.exe
sftp.exe
README.md
LICENSE.txt
```

Standalone `ssh.exe`, `scp.exe`, and `sftp.exe` assets are not uploaded.

## Pre-release checklist

* `README.md` is up to date.
* `LICENSE.txt` is present.
* `HISTORY.md` contains the target version heading.
* `cargo test` passes.
* `cargo build --release` passes.

## Creating a release

To trigger a release (e.g., `v0.1.0`), push a signed Git tag representing the version:

```powershell
# 1. Update HISTORY.md, README.md, etc., commit changes
git add HISTORY.md README.md ...
git commit -m "Prepare v0.1.0 release"

# 2. Create a signed tag
git tag -s v0.1.0 -m "Release v0.1.0"

# 3. Push main branch and the tag
git push origin main
git push origin v0.1.0
```

## Workflow behavior

The release workflow:

1. Checks out the repository.
2. Extracts the version from the tag.
3. Extracts release notes from `HISTORY.md`.
4. Runs `cargo test`.
5. Runs `cargo build --release`.
6. Copies `ssh.exe`, `scp.exe`, `sftp.exe`, `README.md`, and `LICENSE.txt` into a staging directory.
7. Creates `openssh-shim-wsl2-vX.Y.Z-x86_64-pc-windows-msvc.zip`.
8. Generates `SHA256SUMS.txt`.
9. Creates a GitHub Release.
10. Uploads release assets.

## Failure cases

The release should fail if:

* The tag does not match `vX.Y.Z`.
* `HISTORY.md` does not contain `## X.Y.Z`.
* The release notes section is empty.
* Tests fail.
* Build fails.
* The GitHub Release already exists.
