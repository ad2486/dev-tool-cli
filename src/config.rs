use crate::errors::{Category, ErrorCategory};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub type ConfigTable = toml::Table;

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Invalid config.toml file")]
    Invalid(toml::de::Error),
    #[error("Couldn't read the config.toml file")]
    Io(std::io::Error),
    #[error("Could not determine the system config directory")]
    NoConfigDir,
    #[error("Invalid value type for key: {0}")]
    InvalidValue(String),
    #[error("Unknown key: {0}")]
    UnknownKey(String),
}

impl ErrorCategory for ConfigError {
    fn category(&self) -> Category {
        match self {
            Self::Invalid(_) => Category::User,
            Self::Io(_) => Category::Environment,
            Self::NoConfigDir => Category::Environment,
            Self::InvalidValue(_) => Category::User,
            Self::UnknownKey(_) => Category::User,
        }
    }
}

pub fn default_path() -> Result<PathBuf, ConfigError> {
    Ok(directories::ProjectDirs::from("", "", "dev")
        .ok_or(ConfigError::NoConfigDir)?
        .config_dir()
        .join("config.toml"))
}

pub fn load(path: &Path) -> Result<ConfigTable, ConfigError> {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(ConfigTable::new()),
        Err(error) => return Err(ConfigError::Io(error)),
    };
    content.parse::<ConfigTable>().map_err(ConfigError::Invalid)
}

pub fn resolve(
    config: &ConfigTable,
    provider_name: &str,
    defaults: &HashMap<String, String>,
) -> Result<HashMap<String, String>, ConfigError> {
    let mut result = defaults.clone();
    let section = config.get(provider_name).and_then(|valor| valor.as_table());
    if let Some(secao) = section {
        for (key, value) in secao {
            if defaults.contains_key(key) {
                let str_value = value
                    .as_str()
                    .ok_or(ConfigError::InvalidValue(key.clone()))?;
                result.insert(key.clone(), str_value.to_string());
            } else {
                return Err(ConfigError::UnknownKey(key.clone()));
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_config_overrides_manifest_default() {
        let config: ConfigTable = r#"
            [python]
            formatter = "black"
        "#
        .parse()
        .unwrap();

        let mut defaults = HashMap::new();
        defaults.insert("formatter".to_string(), "ruff".to_string());
        defaults.insert("linter".to_string(), "ruff".to_string());
        let result = resolve(&config, "python", &defaults).unwrap();

        assert_eq!(result.get("formatter"), Some(&"black".to_string()));
        assert_eq!(result.get("linter"), Some(&"ruff".to_string()));
    }

    #[test]
    fn unknown_key_returns_error() {
        let config: ConfigTable = r#"
            [python]
            formattr = "black"
        "#
        .parse()
        .unwrap();
        let mut defaults = HashMap::new();
        defaults.insert("formatter".to_string(), "ruff".to_string());
        let result = resolve(&config, "python", &defaults);

        assert!(matches!(result, Err(ConfigError::UnknownKey(_))));
    }
}
