use std::collections::HashMap;
use crate::errors::{ Category, ErrorCategory };

const SUPPORTED_SCHEMA: i64 = 1; 

#[derive(serde::Deserialize)]
pub struct Manifest {
    pub schema: u32,
    pub name: String,
    pub tools: HashMap<String, String>,
    pub packages: HashMap<String, PackageEntry>,
    pub commands: HashMap<String, String>,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
pub enum PackageEntry {
    Name(String),
    ByManager(HashMap<String, String>),
}

#[derive(thiserror::Error, Debug)]
pub enum ManifestError {
    #[error("Invalid manifest.toml file")]
    Invalid(toml::de::Error),
    #[error("Schema version not supported")]
    UnsupportedSchema(i64),
    #[error("Manifest is missing a valid schema field")]
    MissingSchema,
}

impl ErrorCategory for ManifestError {
    fn category (&self) -> Category {
        match self {
            Self::Invalid(_) => Category::Internal,
            Self::UnsupportedSchema(_) => Category::Internal,
            Self::MissingSchema => Category::Internal,
        }
    }
}

pub fn parse(content: &str) -> Result<Manifest, ManifestError> {
    let table = content
        .parse::<toml::Table>()
        .map_err(ManifestError::Invalid)?;

    let schema = table
        .get("schema")
        .and_then(|valor| valor.as_integer())
        .ok_or(ManifestError::MissingSchema)?;

    if schema != SUPPORTED_SCHEMA {
        return Err(ManifestError::UnsupportedSchema(schema));
    }

    toml::from_str::<Manifest>(content).map_err(ManifestError::Invalid)
}