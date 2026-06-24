mod config;
mod debug;
mod env;
mod errors;
mod launcher;
mod login_shell;
mod openssh_args;
mod path_convert;
mod shell_quote;
mod tool;
mod wsl;

pub use tool::Tool;

const EXIT_SHIM_ERROR: i32 = 255;

pub fn run(tool: Tool) -> i32 {
    match run_inner(tool) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("openssh-shim-wsl2: error: {err}");
            EXIT_SHIM_ERROR
        }
    }
}

fn run_inner(tool: Tool) -> errors::ShimResult<i32> {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();
    let parsed = config::parse(tool, raw_args)?;

    let cwd = std::env::current_dir()?;
    let cwd_string = cwd.to_string_lossy().to_string();

    let mut paths_to_collect = openssh_args::collect_paths(tool, &parsed.open_ssh_args)?;
    paths_to_collect.push(cwd_string);

    let converter = path_convert::PathConverter::new(&parsed.config, &paths_to_collect)?;

    let converted = openssh_args::convert_args(tool, &parsed.open_ssh_args, &converter)?;
    launcher::launch(tool, parsed.config, converted, &converter)
}
