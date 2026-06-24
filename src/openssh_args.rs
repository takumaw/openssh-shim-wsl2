use crate::errors::ShimResult;
use crate::path_convert::PathConverter;
use crate::tool::Tool;

pub fn collect_paths(tool: Tool, args: &[String]) -> ShimResult<Vec<String>> {
    match tool {
        Tool::Ssh => collect_ssh_like(args, false),
        Tool::Scp => collect_scp(args),
        Tool::Sftp => collect_sftp(args),
    }
}

fn collect_ssh_like(args: &[String], include_sftp_batch: bool) -> ShimResult<Vec<String>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--" {
            i += 1;
            continue;
        }

        if matches!(arg.as_str(), "-F" | "-i" | "-E") || (include_sftp_batch && arg == "-b") {
            i += 1;
            if let Some(value) = args.get(i) {
                if !(include_sftp_batch && arg == "-b" && value == "-") {
                    out.push(value.clone());
                }
                i += 1;
            }
            continue;
        }

        if let Some(prefix) = joined_path_option_prefix(arg, include_sftp_batch) {
            let value = &arg[prefix.len()..];
            out.push(value.to_string());
            i += 1;
            continue;
        }

        if arg == "-o" {
            i += 1;
            if let Some(value) = args.get(i) {
                if let Some(path_val) = extract_o_option_path(value) {
                    out.push(path_val);
                }
                i += 1;
            }
            continue;
        }

        if let Some(value) = arg.strip_prefix("-o") {
            if let Some(path_val) = extract_o_option_path(value) {
                out.push(path_val);
            }
            i += 1;
            continue;
        }

        i += 1;
    }
    Ok(out)
}

fn extract_o_option_path(value: &str) -> Option<String> {
    let eq = value.find('=')?;
    let key = &value[..eq];
    let val = &value[eq + 1..];
    if is_path_valued_ssh_config_key(key) {
        Some(val.to_string())
    } else {
        None
    }
}

fn collect_scp(args: &[String]) -> ShimResult<Vec<String>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if matches!(arg.as_str(), "-F" | "-i" | "-E") {
            i += 1;
            if let Some(value) = args.get(i) {
                out.push(value.clone());
                i += 1;
            }
            continue;
        }
        if matches!(arg.as_str(), "-S") {
            i += 2;
            continue;
        }
        if let Some(prefix) = joined_path_option_prefix(arg, false) {
            let value = &arg[prefix.len()..];
            out.push(value.to_string());
            i += 1;
            continue;
        }
        if arg == "-o" {
            i += 1;
            if let Some(value) = args.get(i) {
                if let Some(path_val) = extract_o_option_path(value) {
                    out.push(path_val);
                }
                i += 1;
            }
            continue;
        }
        if let Some(value) = arg.strip_prefix("-o") {
            if let Some(path_val) = extract_o_option_path(value) {
                out.push(path_val);
            }
            i += 1;
            continue;
        }
        out.push(arg.clone());
        i += 1;
    }
    Ok(out)
}

fn collect_sftp(args: &[String]) -> ShimResult<Vec<String>> {
    collect_ssh_like(args, true)
}

pub fn convert_args(
    tool: Tool,
    args: &[String],
    converter: &PathConverter,
) -> ShimResult<Vec<String>> {
    match tool {
        Tool::Ssh => convert_ssh_like(args, converter, false),
        Tool::Scp => convert_scp(args, converter),
        Tool::Sftp => convert_sftp(args, converter),
    }
}

fn convert_ssh_like(
    args: &[String],
    converter: &PathConverter,
    include_sftp_batch: bool,
) -> ShimResult<Vec<String>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--" {
            out.push(arg.clone());
            i += 1;
            continue;
        }

        if matches!(arg.as_str(), "-F" | "-i" | "-E") || (include_sftp_batch && arg == "-b") {
            out.push(arg.clone());
            i += 1;
            if let Some(value) = args.get(i) {
                if include_sftp_batch && arg == "-b" && value == "-" {
                    out.push(value.clone());
                } else {
                    out.push(converter.convert(value)?);
                }
                i += 1;
            }
            continue;
        }

        if let Some(prefix) = joined_path_option_prefix(arg, include_sftp_batch) {
            let value = &arg[prefix.len()..];
            out.push(format!("{prefix}{}", converter.convert(value)?));
            i += 1;
            continue;
        }

        if arg == "-o" {
            out.push(arg.clone());
            i += 1;
            if let Some(value) = args.get(i) {
                out.push(convert_o_option(value, converter)?);
                i += 1;
            }
            continue;
        }

        if let Some(value) = arg.strip_prefix("-o") {
            out.push(format!("-o{}", convert_o_option(value, converter)?));
            i += 1;
            continue;
        }

        out.push(arg.clone());
        i += 1;
    }
    Ok(out)
}

fn joined_path_option_prefix(arg: &str, include_sftp_batch: bool) -> Option<&'static str> {
    for prefix in ["-F", "-i", "-E"] {
        if arg.starts_with(prefix) && arg.len() > prefix.len() {
            return Some(prefix);
        }
    }
    if include_sftp_batch && arg.starts_with("-b") && arg.len() > 2 {
        return Some("-b");
    }
    None
}

fn convert_o_option(value: &str, converter: &PathConverter) -> ShimResult<String> {
    let Some(eq) = value.find('=') else {
        return Ok(value.to_string());
    };
    let key = &value[..eq];
    let val = &value[eq + 1..];
    if is_path_valued_ssh_config_key(key) {
        Ok(format!("{key}={}", converter.convert(val)?))
    } else {
        Ok(value.to_string())
    }
}

fn is_path_valued_ssh_config_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "certificatefile"
            | "controlpath"
            | "globalknownhostsfile"
            | "identityfile"
            | "include"
            | "userknownhostsfile"
    )
}

fn convert_scp(args: &[String], converter: &PathConverter) -> ShimResult<Vec<String>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if matches!(arg.as_str(), "-F" | "-i" | "-E") {
            out.push(arg.clone());
            i += 1;
            if let Some(value) = args.get(i) {
                out.push(converter.convert(value)?);
                i += 1;
            }
            continue;
        }
        if matches!(arg.as_str(), "-S") {
            out.push(arg.clone());
            i += 1;
            if let Some(value) = args.get(i) {
                out.push(value.clone());
                i += 1;
            }
            continue;
        }
        if let Some(prefix) = joined_path_option_prefix(arg, false) {
            let value = &arg[prefix.len()..];
            out.push(format!("{prefix}{}", converter.convert(value)?));
            i += 1;
            continue;
        }
        if arg == "-o" {
            out.push(arg.clone());
            i += 1;
            if let Some(value) = args.get(i) {
                out.push(convert_o_option(value, converter)?);
                i += 1;
            }
            continue;
        }
        if let Some(value) = arg.strip_prefix("-o") {
            out.push(format!("-o{}", convert_o_option(value, converter)?));
            i += 1;
            continue;
        }
        out.push(converter.convert(arg)?);
        i += 1;
    }
    Ok(out)
}

fn convert_sftp(args: &[String], converter: &PathConverter) -> ShimResult<Vec<String>> {
    convert_ssh_like(args, converter, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn converts_ssh_path_options() {
        let cfg = Config::default();
        let args = vec![
            "-F".to_string(),
            r"C:\Users\me\.ssh\config".to_string(),
            "host".to_string(),
        ];
        let converter = PathConverter::new(&cfg, &args).unwrap();
        let out = convert_args(Tool::Ssh, &args, &converter).unwrap();
        assert_eq!(out[1], "/mnt/c/Users/me/.ssh/config");
    }

    #[test]
    fn converts_scp_local_paths_but_not_remote_specs() {
        let cfg = Config::default();
        let args = vec![r"C:\tmp\a.txt".to_string(), "host:/tmp/a.txt".to_string()];
        let converter = PathConverter::new(&cfg, &args).unwrap();
        let out = convert_args(Tool::Scp, &args, &converter).unwrap();
        assert_eq!(out[0], "/mnt/c/tmp/a.txt");
        assert_eq!(out[1], "host:/tmp/a.txt");
    }

    #[test]
    fn does_not_convert_scp_capital_s() {
        let cfg = Config::default();
        let args = vec![
            "-S".to_string(),
            r"C:\ssh.exe".to_string(),
            "host:/tmp/a".to_string(),
        ];
        let converter = PathConverter::new(&cfg, &args).unwrap();
        let out = convert_args(Tool::Scp, &args, &converter).unwrap();
        assert_eq!(out[1], r"C:\ssh.exe");
    }

    #[test]
    fn converts_sftp_batch_file_but_not_positional() {
        let cfg = Config::default();
        let args = vec![
            "-b".to_string(),
            r"C:\batch.txt".to_string(),
            "host:/tmp".to_string(),
        ];
        let converter = PathConverter::new(&cfg, &args).unwrap();
        let out = convert_args(Tool::Sftp, &args, &converter).unwrap();
        assert_eq!(out[1], "/mnt/c/batch.txt");
        assert_eq!(out[2], "host:/tmp");
    }

    #[test]
    fn collects_paths_properly() {
        let args = vec![
            "-F".to_string(),
            r"C:\Users\me\.ssh\config".to_string(),
            "host".to_string(),
        ];
        let paths = collect_paths(Tool::Ssh, &args).unwrap();
        assert_eq!(paths, vec![r"C:\Users\me\.ssh\config".to_string()]);
    }
}
