use std::process::Command;

use crate::config::Config;
use crate::debug;
use crate::wsl::WSL_EXE;

pub fn resolve_login_shell(config: &Config) -> Option<String> {
    let mut args = Vec::new();
    if let Some(distro) = &config.distribution {
        args.push("--distribution".to_string());
        args.push(distro.clone());
    }
    args.extend([
        "--exec".to_string(),
        "sh".to_string(),
        "-c".to_string(),
        r#"awk -F: -v u="$(id -un)" '$1 == u { print $7; exit }' /etc/passwd"#.to_string(),
    ]);

    let output = Command::new(WSL_EXE).args(&args).output().ok()?;
    if !output.status.success() {
        debug::line(
            config,
            "login shell resolution failed; falling back to /bin/sh",
        );
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let shell = stdout.lines().next().unwrap_or("").trim();
    if shell.is_empty() || !shell.starts_with('/') {
        debug::line(
            config,
            "login shell resolution produced no absolute shell path; falling back to /bin/sh",
        );
        return None;
    }
    Some(shell.to_string())
}
