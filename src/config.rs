use crate::env;
use crate::errors::{ShimError, ShimResult};
use crate::tool::Tool;

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub distribution: Option<String>,
    pub use_wslpath: bool,
    pub use_shell: bool,
    pub shell: Option<String>,
    pub use_login_shell: bool,
    pub debug: bool,
}

#[derive(Clone, Debug)]
pub struct ParsedInvocation {
    pub config: Config,
    pub open_ssh_args: Vec<String>,
}

pub fn parse(_tool: Tool, args: Vec<String>) -> ShimResult<ParsedInvocation> {
    let mut config = Config {
        distribution: env::optional("OPENSSH_SHIM_WSL2_DISTRIBUTION"),
        use_wslpath: env::bool_env("OPENSSH_SHIM_WSL2_USE_WSLPATH")?.unwrap_or(false),
        use_shell: env::bool_env("OPENSSH_SHIM_WSL2_USE_SHELL")?.unwrap_or(false),
        shell: env::optional("OPENSSH_SHIM_WSL2_SHELL"),
        use_login_shell: env::bool_env("OPENSSH_SHIM_WSL2_USE_LOGIN_SHELL")?.unwrap_or(false),
        debug: env::bool_env("OPENSSH_SHIM_WSL2_DEBUG")?.unwrap_or(false),
    };

    if config.shell.is_some() && config.use_login_shell {
        // Environment conflict policy: explicit shell wins.
        config.use_login_shell = false;
    }

    let mut cli_shell_set = false;
    let mut cli_login_shell_set = false;
    let mut out = Vec::new();
    let mut i = 0;
    let mut stop_shim_option_parsing = false;

    while i < args.len() {
        let arg = &args[i];
        if stop_shim_option_parsing {
            out.push(arg.clone());
            i += 1;
            continue;
        }

        match arg.as_str() {
            "--" => {
                stop_shim_option_parsing = true;
                out.push(arg.clone());
                i += 1;
            }
            "--wsl2-distribution" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| ShimError::new("--wsl2-distribution requires a value"))?;
                config.distribution = Some(value.clone());
                i += 1;
            }
            "--wsl2-use-wslpath" => {
                config.use_wslpath = true;
                i += 1;
            }
            "--wsl2-use-shell" => {
                config.use_shell = true;
                i += 1;
            }
            "--wsl2-shell" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| ShimError::new("--wsl2-shell requires a value"))?;
                config.shell = Some(value.clone());
                config.use_shell = true;
                config.use_login_shell = false;
                cli_shell_set = true;
                i += 1;
            }
            "--wsl2-use-login-shell" => {
                config.use_login_shell = true;
                config.use_shell = true;
                config.shell = None;
                cli_login_shell_set = true;
                i += 1;
            }
            "--wsl2-debug" => {
                config.debug = true;
                i += 1;
            }
            _ => {
                out.push(arg.clone());
                i += 1;
            }
        }
    }

    if cli_shell_set && cli_login_shell_set {
        return Err(ShimError::new(
            "--wsl2-shell and --wsl2-use-login-shell are mutually exclusive",
        ));
    }

    if config.shell.is_some() || config.use_login_shell {
        config.use_shell = true;
    }

    Ok(ParsedInvocation {
        config,
        open_ssh_args: out,
    })
}
