use crate::errors::{Category, ErrorCategory};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub type ConfigTable = toml::Table;

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Invalid config.toml file: {0}")]
    Invalid(String),
    #[error("Couldn't read the config.toml file")]
    Io(std::io::Error),
    #[error("Could not determine the system config directory")]
    NoConfigDir,
    #[error("Invalid value type for key: {0}")]
    InvalidValue(String),
    #[error("Unknown key: {0}")]
    UnknownKey(String),
    #[error("Invalid key `{0}`, expected <language>.<key> (e.g. python.formatter)")]
    MalformedKey(String),
}

impl ErrorCategory for ConfigError {
    fn category(&self) -> Category {
        match self {
            Self::Invalid(_) => Category::User,
            Self::Io(_) => Category::Environment,
            Self::NoConfigDir => Category::Environment,
            Self::InvalidValue(_) => Category::User,
            Self::UnknownKey(_) => Category::User,
            Self::MalformedKey(_) => Category::User,
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
    read(path)?
        .parse::<ConfigTable>()
        .map_err(|error| ConfigError::Invalid(error.to_string()))
}

pub fn set(path: &Path, section: &str, key: &str, value: &str) -> Result<(), ConfigError> {
    let mut document = read(path)?
        .parse::<toml_edit::DocumentMut>()
        .map_err(|error| ConfigError::Invalid(error.to_string()))?;
    document.entry(section).or_insert(toml_edit::table())[key] = toml_edit::value(value);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(ConfigError::Io)?;
    }
    std::fs::write(path, document.to_string()).map_err(ConfigError::Io)
}

fn read(path: &Path) -> Result<String, ConfigError> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(ConfigError::Io(error)),
    }
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

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!("dev-config-{name}-{}", std::process::id()))
            .join("config.toml")
    }

    #[test]
    fn set_creates_the_file_and_its_directory() {
        let path = temp_path("create");

        set(&path, "python", "formatter", "black").unwrap();

        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "[python]\nformatter = \"black\"\n"
        );
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn set_keeps_the_rest_of_the_file_untouched() {
        let path = temp_path("preserve");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            "# my settings\n[python]\nformatter = \"ruff\" # fast\nlinter = \"ruff\"\n",
        )
        .unwrap();

        set(&path, "python", "linter", "flake8").unwrap();

        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "# my settings\n[python]\nformatter = \"ruff\" # fast\nlinter = \"flake8\"\n"
        );
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
