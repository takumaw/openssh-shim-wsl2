use crate::errors::{ShimError, ShimResult};

pub fn optional(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

pub fn bool_env(name: &str) -> ShimResult<Option<bool>> {
    let Some(value) = std::env::var(name).ok() else {
        return Ok(None);
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(Some(false));
    }
    match trimmed.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(Some(true)),
        "0" | "false" | "no" | "off" => Ok(Some(false)),
        _ => Err(ShimError::new(format!(
            "invalid boolean value for {name}: {value}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    fn parse_bool_value(value: &str) -> Result<bool, String> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Ok(false);
        }
        match trimmed.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Ok(true),
            "0" | "false" | "no" | "off" => Ok(false),
            _ => Err(value.to_string()),
        }
    }

    #[test]
    fn parses_boolean_values() {
        for v in ["1", "true", "yes", "on", "TRUE"] {
            assert!(parse_bool_value(v).unwrap());
        }
        for v in ["0", "false", "no", "off", ""] {
            assert!(!parse_bool_value(v).unwrap());
        }
        assert!(parse_bool_value("maybe").is_err());
    }
}
