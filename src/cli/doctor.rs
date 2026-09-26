use crate::config::{self, ConfigTable};
use crate::core::Registry;
use crate::errors::AppError;
use crate::os::{CommandRunner, OsAdapter};
use crate::project::{self, Project, ProjectError};
use crate::providers::{InstallContext, Provider, Report, Status, same_minor};
use std::collections::BTreeMap;
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

        let mut report = provider.doctor(&ctx)?;
        if !project.dependencies.is_empty() {
            let current = provider.versions(&ctx)?;
            compare_versions(&mut report, &project.dependencies, &current);
        }
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

fn compare_versions(
    report: &mut Report,
    recorded: &BTreeMap<String, String>,
    current: &BTreeMap<String, String>,
) {
    for check in report.checks.iter_mut() {
        if check.status != Status::Ok {
            continue;
        }
        let (Some(expected), Some(found)) = (recorded.get(&check.name), current.get(&check.name))
        else {
            continue;
        };
        if !same_minor(expected, found) {
            check.status = Status::Mismatch {
                expected: expected.clone(),
                found: found.clone(),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::Check;
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

    fn versions(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(name, version)| (name.to_string(), version.to_string()))
            .collect()
    }

    fn report(checks: &[(&str, Status)]) -> Report {
        Report::new(
            "python",
            checks
                .iter()
                .map(|(name, status)| Check::new(*name, status.clone()))
                .collect(),
        )
    }

    #[test]
    fn a_different_minor_version_is_a_mismatch() {
        let mut report = report(&[("python", Status::Ok)]);

        compare_versions(
            &mut report,
            &versions(&[("python", "3.12.1")]),
            &versions(&[("python", "3.13.0")]),
        );

        assert_eq!(
            report.checks[0].status,
            Status::Mismatch {
                expected: "3.12.1".to_string(),
                found: "3.13.0".to_string(),
            }
        );
    }

    #[test]
    fn a_different_patch_version_is_still_ok() {
        let mut report = report(&[("python", Status::Ok)]);

        compare_versions(
            &mut report,
            &versions(&[("python", "3.12.1")]),
            &versions(&[("python", "3.12.9")]),
        );

        assert_eq!(report.checks[0].status, Status::Ok);
    }

    #[test]
    fn a_missing_dependency_stays_missing() {
        let mut report = report(&[("python", Status::Missing)]);

        compare_versions(
            &mut report,
            &versions(&[("python", "3.12.1")]),
            &versions(&[]),
        );

        assert_eq!(report.checks[0].status, Status::Missing);
    }

    #[test]
    fn an_unrecorded_dependency_is_not_compared() {
        let mut report = report(&[("uv", Status::Ok)]);

        compare_versions(&mut report, &versions(&[]), &versions(&[("uv", "0.12.17")]));

        assert_eq!(report.checks[0].status, Status::Ok);
    }
}
