use crate::errors::{Category, ErrorCategory};
use std::collections::HashMap;

const SUPPORTED_SCHEMA: i64 = 1;

#[derive(serde::Deserialize, Default)]
pub struct Manifest {
    pub schema: u32,
    pub name: String,
    pub tools: HashMap<String, String>,
    pub packages: HashMap<String, PackageEntry>,
    pub commands: HashMap<String, String>,
}

#[derive(serde::Deserialize, Debug, PartialEq)]
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
    fn category(&self) -> Category {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_manifest() {
        let toml = r#"
            schema = 1
            name = "python"

            [tools]
            formatter = "ruff"
            linter = "ruff"
            tester = "pytest"

            [packages]
            apt = "python3"
            brew = "python"

            [packages.uv]
            apt = "uv"
            brew = "uv"

            [commands]
            init = "uv init"
        "#;

        let manifest = parse(toml).unwrap();
        let mut uv_packages = HashMap::new();
        uv_packages.insert("apt".to_string(), "uv".to_string());
        uv_packages.insert("brew".to_string(), "uv".to_string());

        assert_eq!(manifest.schema, 1);
        assert_eq!(
            manifest.packages.get("uv"),
            Some(&PackageEntry::ByManager(uv_packages))
        );
    }
}
