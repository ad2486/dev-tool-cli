use crate::config::{self, ConfigTable};
use crate::core::Registry;
use crate::errors::AppError;
use crate::os::{CommandRunner, OsAdapter};
use crate::project::{self, Project, ProjectError};
use crate::providers::{InstallContext, Provider};
use std::rc::Rc;

pub fn run(
    registry: &Registry,
    config: &ConfigTable,
    os_adapter: Rc<dyn OsAdapter>,
    command_runner: Rc<dyn CommandRunner>,
) -> Result<(), AppError> {
    let project = project::load(&std::env::current_dir().map_err(ProjectError::Io)?)?;

    for provider in declared_providers(registry, &project)? {
        let defaults = provider.manifest().tools.clone();
        let resolved = config::resolve(config, provider.name(), &defaults)?;

        let ctx = InstallContext {
            config: resolved,
            os_adapter: os_adapter.clone(),
            command_runner: command_runner.clone(),
        };

        let report = provider.doctor(&ctx)?;
        print!("{report}");
    }

    Ok(())
}

fn declared_providers<'a>(
    registry: &'a Registry,
    project: &Project,
) -> Result<Vec<&'a dyn Provider>, ProjectError> {
    project
        .components
        .iter()
        .map(|component| {
            registry
                .get(component)
                .ok_or_else(|| ProjectError::UnknownComponent(component.clone()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::python::PythonProvider;

    fn registry_with_python() -> Registry {
        let mut registry = Registry::new();
        registry.register(Box::new(PythonProvider::new().unwrap()));
        registry
    }

    fn project_with(components: &[&str]) -> Project {
        let mut project = Project::new();
        project.components = components.iter().map(|name| name.to_string()).collect();
        project
    }

    #[test]
    fn resolves_only_the_declared_components() {
        let registry = registry_with_python();
        let project = project_with(&["python"]);

        let providers = declared_providers(&registry, &project).unwrap();

        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].name(), "python");
    }

    #[test]
    fn a_project_declaring_nothing_checks_nothing() {
        let registry = registry_with_python();
        let project = project_with(&[]);

        let providers = declared_providers(&registry, &project).unwrap();

        assert!(providers.is_empty());
    }

    #[test]
    fn an_unknown_component_is_an_error() {
        let registry = registry_with_python();
        let project = project_with(&["python", "cobol"]);

        let result = declared_providers(&registry, &project);

        assert!(matches!(
            result,
            Err(ProjectError::UnknownComponent(name)) if name == "cobol"
        ));
    }
}
