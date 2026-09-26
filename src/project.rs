use crate::errors::{Category, ErrorCategory};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const FILE_NAME: &str = "dev.toml";
const SUPPORTED_SCHEMA: i64 = 1;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, PartialEq)]
pub struct Project {
    pub schema: u32,
    pub components: Vec<String>,
    pub dependencies: BTreeMap<String, String>,
}

#[derive(thiserror::Error, Debug)]
pub enum ProjectError {
    #[error("Not a dev project: no {FILE_NAME} found. Run `dev init <language>` first")]
    NotAProject,
    #[error("Invalid {FILE_NAME} file")]
    Invalid(toml::de::Error),
    #[error("Missing `schema` field in {FILE_NAME}")]
    MissingSchema,
    #[error("Unsupported {FILE_NAME} schema {0}; this project needs a newer `dev`")]
    UnsupportedSchema(i64),
    #[error("Couldn't read or write the {FILE_NAME} file")]
    Io(std::io::Error),
    #[error("Couldn't serialize the {FILE_NAME} file")]
    Serialize(toml::ser::Error),
}

impl ErrorCategory for ProjectError {
    fn category(&self) -> Category {
        match self {
            Self::NotAProject => Category::User,
            Self::Invalid(_) => Category::User,
            Self::MissingSchema => Category::User,
            Self::UnsupportedSchema(_) => Category::User,
            Self::Io(_) => Category::Environment,
            Self::Serialize(_) => Category::Internal,
        }
    }
}

impl Project {
    pub fn new() -> Self {
        Self {
            schema: SUPPORTED_SCHEMA as u32,
            components: Vec::new(),
            dependencies: BTreeMap::new(),
        }
    }

    pub fn record(&mut self, component: &str, dependencies: BTreeMap<String, String>) {
        if !self.components.iter().any(|name| name == component) {
            self.components.push(component.to_string());
        }
        self.dependencies.extend(dependencies);
    }
}

pub fn path(dir: &Path) -> PathBuf {
    dir.join(FILE_NAME)
}

pub fn parse(content: &str) -> Result<Project, ProjectError> {
    let table = content
        .parse::<toml::Table>()
        .map_err(ProjectError::Invalid)?;

    let schema = table
        .get("schema")
        .and_then(|value| value.as_integer())
        .ok_or(ProjectError::MissingSchema)?;

    if schema != SUPPORTED_SCHEMA {
        return Err(ProjectError::UnsupportedSchema(schema));
    }

    toml::from_str::<Project>(content).map_err(ProjectError::Invalid)
}

pub fn load(dir: &Path) -> Result<Project, ProjectError> {
    let content = match std::fs::read_to_string(path(dir)) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(ProjectError::NotAProject);
        }
        Err(error) => return Err(ProjectError::Io(error)),
    };

    parse(&content)
}

pub fn save(dir: &Path, project: &Project) -> Result<(), ProjectError> {
    let content = toml::to_string(project).map_err(ProjectError::Serialize)?;
    std::fs::write(path(dir), content).map_err(ProjectError::Io)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deps(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(name, version)| (name.to_string(), version.to_string()))
            .collect()
    }

    #[test]
    fn parses_a_valid_project() {
        let content = r#"
            schema = 1
            components = ["python"]

            [dependencies]
            python3 = "3.12.1"
            uv = "0.4.2"
        "#;

        let project = parse(content).unwrap();

        assert_eq!(project.components, vec!["python"]);
        assert_eq!(project.dependencies["uv"], "0.4.2");
    }

    #[test]
    fn rejects_an_unsupported_schema() {
        let content = "schema = 2\ncomponents = []\n\n[dependencies]\n";

        let result = parse(content);

        assert!(matches!(result, Err(ProjectError::UnsupportedSchema(2))));
    }

    #[test]
    fn rejects_a_file_without_schema() {
        let content = "components = []\n\n[dependencies]\n";

        let result = parse(content);

        assert!(matches!(result, Err(ProjectError::MissingSchema)));
    }

    #[test]
    fn record_adds_a_component_and_its_dependencies() {
        let mut project = Project::new();

        project.record("python", deps(&[("python3", "3.12.1"), ("uv", "0.4.2")]));

        assert_eq!(project.components, vec!["python"]);
        assert_eq!(project.dependencies.len(), 2);
    }

    #[test]
    fn record_does_not_duplicate_a_component() {
        let mut project = Project::new();

        project.record("python", deps(&[("python3", "3.12.1")]));
        project.record("python", deps(&[("python3", "3.13.0")]));

        assert_eq!(project.components, vec!["python"]);
        assert_eq!(project.dependencies["python3"], "3.13.0");
    }

    #[test]
    fn record_keeps_earlier_components() {
        let mut project = Project::new();

        project.record("python", deps(&[("python3", "3.12.1")]));
        project.record("docker", deps(&[("docker", "27.3.1")]));

        assert_eq!(project.components, vec!["python", "docker"]);
        assert_eq!(project.dependencies.len(), 2);
    }

    #[test]
    fn saves_and_loads_a_round_trip() {
        let dir = std::env::temp_dir().join(format!("dev-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let mut project = Project::new();
        project.record("python", deps(&[("python3", "3.12.1")]));
        save(&dir, &project).unwrap();

        let loaded = load(&dir).unwrap();

        assert_eq!(loaded, project);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn load_without_the_file_is_not_a_project() {
        let dir = std::env::temp_dir().join("dev-test-definitely-not-a-project");

        let result = load(&dir);

        assert!(matches!(result, Err(ProjectError::NotAProject)));
    }
}
