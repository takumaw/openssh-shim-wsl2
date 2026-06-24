use std::process::Command;

use crate::config::Config;
use crate::debug;
use crate::errors::{ShimError, ShimResult};
use crate::login_shell;
use crate::path_convert;
use crate::shell_quote;
use crate::tool::Tool;
use crate::wsl::WSL_EXE;

pub fn launch(
    tool: Tool,
    config: Config,
    args: Vec<String>,
    converter: &path_convert::PathConverter,
) -> ShimResult<i32> {
    let cwd = std::env::current_dir()?;
    let cwd_string = cwd.to_string_lossy().to_string();
    let converted_cwd = converter.convert(&cwd_string)?;

    debug::line(&config, format!("tool: {}", tool.command()));
    debug::line(
        &config,
        format!(
            "distribution: {}",
            config.distribution.as_deref().unwrap_or("<default>")
        ),
    );
    debug::line(&config, format!("cwd: {cwd_string} -> {converted_cwd}"));
    debug::line(
        &config,
        format!(
            "path conversion: {}",
            if config.use_wslpath {
                "wslpath"
            } else {
                "builtin"
            }
        ),
    );

    let mut wsl_args = Vec::new();
    if let Some(distro) = &config.distribution {
        wsl_args.push("--distribution".to_string());
        wsl_args.push(distro.clone());
    }
    wsl_args.push("--cd".to_string());
    wsl_args.push(converted_cwd);

    if config.use_shell {
        let shell = select_shell(&config);
        let command = shell_quote::shell_command(tool.command(), &args);
        debug::line(&config, "shell mode: enabled");
        debug::line(&config, format!("resolved shell: {shell}"));
        debug::line(&config, format!("command: {command}"));
        wsl_args.push("--exec".to_string());
        wsl_args.push(shell);
        wsl_args.push("-lc".to_string());
        wsl_args.push(command);
    } else {
        debug::line(&config, "shell mode: disabled");
        wsl_args.push("--exec".to_string());
        wsl_args.push(tool.command().to_string());
        wsl_args.extend(args);
    }

    debug::argv(&config, WSL_EXE, &wsl_args);

    let status = Command::new(WSL_EXE)
        .args(&wsl_args)
        .status()
        .map_err(|err| ShimError::new(format!("failed to launch {WSL_EXE}: {err}")))?;

    Ok(status.code().unwrap_or(255))
}

fn select_shell(config: &Config) -> String {
    if let Some(shell) = &config.shell {
        debug::line(config, "shell selection: explicit");
        return shell.clone();
    }
    if config.use_login_shell {
        debug::line(config, "shell selection: login shell");
        return login_shell::resolve_login_shell(config).unwrap_or_else(|| "/bin/sh".to_string());
    }
    debug::line(config, "shell selection: default");
    "/bin/sh".to_string()
}
