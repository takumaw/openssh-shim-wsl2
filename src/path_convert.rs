use crate::config::Config;
use crate::errors::{ShimError, ShimResult};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PathKind {
    WindowsDriveAbsolute,
    WindowsExtendedDriveAbsolute,
    WindowsRelative,
    WindowsUnc,
    RemoteSpec,
    LinuxOrPlain,
}

pub struct PathConverter {
    config: Config,
    wslpath_cache: HashMap<String, String>,
}

impl PathConverter {
    pub fn new(config: &Config, paths: &[String]) -> ShimResult<Self> {
        let mut wslpath_cache = HashMap::new();
        if config.use_wslpath {
            let mut to_convert = Vec::new();
            for p in paths {
                match classify(p) {
                    PathKind::WindowsDriveAbsolute
                    | PathKind::WindowsExtendedDriveAbsolute
                    | PathKind::WindowsRelative => {
                        if !to_convert.contains(p) {
                            to_convert.push(p.clone());
                        }
                    }
                    PathKind::WindowsUnc => {
                        return Err(ShimError::new(format!("UNC paths are not supported: {p}")));
                    }
                    _ => {}
                }
            }

            if !to_convert.is_empty() {
                let mut args = Vec::new();
                if let Some(distro) = &config.distribution {
                    args.push("--distribution".to_string());
                    args.push(distro.clone());
                }
                args.extend([
                    "--exec".to_string(),
                    "sh".to_string(),
                    "-c".to_string(),
                    r#"for p in "$@"; do wslpath -u "$p"; done"#.to_string(),
                    "--".to_string(),
                ]);
                args.extend(to_convert.clone());

                let output = std::process::Command::new(crate::wsl::WSL_EXE)
                    .args(&args)
                    .output()
                    .map_err(|err| ShimError::new(format!("failed to run wslpath batch: {err}")))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(ShimError::new(format!(
                        "wslpath batch conversion failed: {stderr}"
                    )));
                }

                let stdout = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = stdout.lines().collect();
                if lines.len() != to_convert.len() {
                    return Err(ShimError::new(format!(
                        "wslpath output count mismatch: expected {}, got {}",
                        to_convert.len(),
                        lines.len()
                    )));
                }

                for (win_path, linux_path) in to_convert.into_iter().zip(lines) {
                    wslpath_cache.insert(win_path, linux_path.trim().to_string());
                }
            }
        }
        Ok(Self {
            config: config.clone(),
            wslpath_cache,
        })
    }

    pub fn convert(&self, value: &str) -> ShimResult<String> {
        if self.config.use_wslpath {
            match classify(value) {
                PathKind::WindowsDriveAbsolute
                | PathKind::WindowsExtendedDriveAbsolute
                | PathKind::WindowsRelative => {
                    if let Some(cached) = self.wslpath_cache.get(value) {
                        return Ok(cached.clone());
                    }
                    let mut args = Vec::new();
                    if let Some(distro) = &self.config.distribution {
                        args.push("--distribution".to_string());
                        args.push(distro.clone());
                    }
                    args.extend([
                        "--exec".to_string(),
                        "wslpath".to_string(),
                        "-u".to_string(),
                        value.to_string(),
                    ]);
                    let output = std::process::Command::new(crate::wsl::WSL_EXE)
                        .args(&args)
                        .output()
                        .map_err(|err| {
                            ShimError::new(format!("failed to run wslpath for {value}: {err}"))
                        })?;
                    if !output.status.success() {
                        return Err(ShimError::new(format!("wslpath failed for {value}")));
                    }
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    return Ok(stdout.trim().to_string());
                }
                PathKind::WindowsUnc => {
                    return Err(ShimError::new(format!(
                        "UNC paths are not supported: {value}"
                    )));
                }
                PathKind::RemoteSpec | PathKind::LinuxOrPlain => {
                    return Ok(value.to_string());
                }
            }
        }

        match classify(value) {
            PathKind::WindowsDriveAbsolute => convert_drive_absolute(value),
            PathKind::WindowsExtendedDriveAbsolute => convert_extended_drive_absolute(value),
            PathKind::WindowsRelative => Ok(value.replace('\\', "/")),
            PathKind::WindowsUnc => Err(ShimError::new(format!(
                "UNC paths are not supported: {value}"
            ))),
            PathKind::RemoteSpec | PathKind::LinuxOrPlain => Ok(value.to_string()),
        }
    }
}

#[allow(dead_code)]
pub fn convert_path(value: &str, config: &Config) -> ShimResult<String> {
    let converter = PathConverter::new(config, &[value.to_string()])?;
    converter.convert(value)
}

pub fn classify(value: &str) -> PathKind {
    if is_unc(value) {
        return PathKind::WindowsUnc;
    }
    if is_extended_drive_absolute(value) {
        return PathKind::WindowsExtendedDriveAbsolute;
    }
    if is_drive_absolute(value) {
        return PathKind::WindowsDriveAbsolute;
    }
    if is_remote_spec(value) {
        return PathKind::RemoteSpec;
    }
    if is_windows_relative(value) {
        return PathKind::WindowsRelative;
    }
    PathKind::LinuxOrPlain
}

fn is_drive_absolute(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
}

fn is_extended_drive_absolute(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with(r"\\?\") && is_drive_absolute(&value[4..])
}

fn is_unc(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    (lower.starts_with(r"\\") && !lower.starts_with(r"\\?\")) || lower.starts_with(r"\\?\unc\")
}

fn is_windows_relative(value: &str) -> bool {
    value.contains('\\') || value.starts_with(r".") || value.starts_with(r"..")
}

fn is_remote_spec(value: &str) -> bool {
    if is_drive_absolute(value) || is_extended_drive_absolute(value) || is_unc(value) {
        return false;
    }
    if value.starts_with("scp://") || value.starts_with("sftp://") {
        return true;
    }
    let Some(idx) = value.find(':') else {
        return false;
    };
    idx > 0 && !value[..idx].contains('/') && !value[..idx].contains('\\')
}

fn convert_drive_absolute(value: &str) -> ShimResult<String> {
    let mut chars = value.chars();
    let drive = chars
        .next()
        .ok_or_else(|| ShimError::new("empty path"))?
        .to_ascii_lowercase();
    let rest = &value[2..];
    let rest = rest.trim_start_matches(['\\', '/']).replace('\\', "/");
    if rest.is_empty() {
        Ok(format!("/mnt/{drive}"))
    } else {
        Ok(format!("/mnt/{drive}/{rest}"))
    }
}

fn convert_extended_drive_absolute(value: &str) -> ShimResult<String> {
    convert_drive_absolute(&value[4..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn classifies_paths() {
        assert_eq!(
            classify(r"C:\Users\me\a.txt"),
            PathKind::WindowsDriveAbsolute
        );
        assert_eq!(
            classify(r"C:/Users/me/a.txt"),
            PathKind::WindowsDriveAbsolute
        );
        assert_eq!(
            classify(r"\\?\C:\Users\me\a.txt"),
            PathKind::WindowsExtendedDriveAbsolute
        );
        assert_eq!(classify(r"\\server\share\a.txt"), PathKind::WindowsUnc);
        assert_eq!(
            classify(r"\\?\UNC\server\share\a.txt"),
            PathKind::WindowsUnc
        );
        assert_eq!(classify("host:/tmp/a"), PathKind::RemoteSpec);
        assert_eq!(classify("user@host:/tmp/a"), PathKind::RemoteSpec);
        assert_eq!(classify(r".\a.txt"), PathKind::WindowsRelative);
        assert_eq!(classify(r"..\a.txt"), PathKind::WindowsRelative);
    }

    #[test]
    fn converts_drive_absolute() {
        let cfg = Config::default();
        assert_eq!(
            convert_path(r"C:\Users\me\a.txt", &cfg).unwrap(),
            "/mnt/c/Users/me/a.txt"
        );
        assert_eq!(
            convert_path(r"D:/tmp/a.txt", &cfg).unwrap(),
            "/mnt/d/tmp/a.txt"
        );
    }

    #[test]
    fn converts_relative_separators() {
        let cfg = Config::default();
        assert_eq!(convert_path(r".\a\b", &cfg).unwrap(), "./a/b");
        assert_eq!(convert_path(r"..\a\b", &cfg).unwrap(), "../a/b");
    }

    #[test]
    fn rejects_unc() {
        let cfg = Config::default();
        assert!(convert_path(r"\\server\share\a", &cfg).is_err());
        assert!(convert_path(r"\\?\UNC\server\share\a", &cfg).is_err());
    }
}
