use crate::config::Config;

pub fn line(config: &Config, message: impl AsRef<str>) {
    if config.debug {
        eprintln!("debug1: openssh-shim-wsl2: {}", message.as_ref());
    }
}

pub fn argv(config: &Config, argv0: &str, args: &[String]) {
    if !config.debug {
        return;
    }
    eprintln!(
        "debug1: openssh-shim-wsl2: launching wsl.exe with {} arguments",
        args.len() + 1
    );
    eprintln!("debug1: openssh-shim-wsl2: argv[0]: {argv0}");
    for (i, arg) in args.iter().enumerate() {
        eprintln!("debug1: openssh-shim-wsl2: argv[{}]: {arg}", i + 1);
    }
}
