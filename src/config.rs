use std::path::{ Path, PathBuf };

use crate::errors::{ Category, ErrorCategory };

pub type ConfigTable = toml::Table;

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Invalid config.toml file")]
    Invalid(toml::de::Error),
    #[error("Couldn't read the config.toml file")]
    Io(std::io::Error),
    #[error("Could not determine the system config directory")]
    NoConfigDir,
}

impl ErrorCategory for ConfigError {
    fn category(&self) -> Category {
        match self {
            Self::Invalid(_) => Category::User,
            Self::Io(_) => Category::Environment,
            Self::NoConfigDir => Category::Environment,
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

// Ok(conteudo) → seguir com o conteúdo (é uma String)
// Err de "não existe" → não é erro, quer sair da função inteira com Ok(ConfigTable::new())
// Err de qualquer outra coisa → sair da função com Err(ConfigError::Io(...))
