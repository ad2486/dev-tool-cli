use crate::config::{self, ConfigError, ConfigTable};
use crate::core::Registry;
use crate::errors::AppError;
use crate::providers::Provider;
use std::path::Path;

#[derive(Debug, PartialEq)]
struct Entry {
    key: String,
    value: String,
    is_default: bool,
}

impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} = {}", self.key, self.value)?;
        if self.is_default {
            write!(f, " (default)")?;
        }
        Ok(())
    }
}

pub fn get(registry: &Registry, config: &ConfigTable, key: Option<&str>) -> Result<(), AppError> {
    let entries = match key {
        Some(key) => {
            let (provider, _) = lookup(registry, key)?;
            entries(config, provider)?
                .into_iter()
                .filter(|entry| entry.key == key)
                .collect()
        }
        None => {
            let mut all = Vec::new();
            for provider in registry.all() {
                all.extend(entries(config, provider)?);
            }
            all
        }
    };

    for entry in entries {
        println!("{entry}");
    }
    Ok(())
}

pub fn set(
    registry: &Registry,
    path: &Path,
    key: &str,
    value: &str,
    dry_run: bool,
) -> Result<(), AppError> {
    let (provider, name) = lookup(registry, key)?;

    if dry_run {
        log::info!("[dry-run] would set {key} = {value} in {}", path.display());
        return Ok(());
    }

    config::set(path, provider.name(), name, value)?;
    println!("{key} = {value}");
    Ok(())
}

fn lookup<'a, 'k>(
    registry: &'a Registry,
    key: &'k str,
) -> Result<(&'a dyn Provider, &'k str), AppError> {
    let (language, name) = key
        .split_once('.')
        .ok_or_else(|| ConfigError::MalformedKey(key.to_string()))?;
    let provider = registry.find(language)?;
    if !provider.manifest().tools.contains_key(name) {
        return Err(ConfigError::UnknownKey(key.to_string()).into());
    }
    Ok((provider, name))
}

fn entries(config: &ConfigTable, provider: &dyn Provider) -> Result<Vec<Entry>, ConfigError> {
    let resolved = config::resolve(config, provider.name(), &provider.manifest().tools)?;
    let section = config
        .get(provider.name())
        .and_then(|value| value.as_table());

    let mut entries: Vec<Entry> = resolved
        .into_iter()
        .map(|(name, value)| Entry {
            is_default: !section.is_some_and(|section| section.contains_key(&name)),
            key: format!("{}.{name}", provider.name()),
            value,
        })
        .collect();
    entries.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::python::PythonProvider;

    fn registry() -> Registry {
        let mut registry = Registry::new();
        registry.register(Box::new(PythonProvider::new().unwrap()));
        registry
    }

    #[test]
    fn entries_mark_which_values_come_from_the_manifest() {
        let registry = registry();
        let config: ConfigTable = "[python]\nformatter = \"black\"\n".parse().unwrap();

        let entries = entries(&config, registry.find("python").unwrap()).unwrap();

        let lines: Vec<String> = entries.iter().map(|entry| entry.to_string()).collect();
        assert_eq!(
            lines,
            vec![
                "python.formatter = black",
                "python.linter = ruff (default)",
                "python.tester = pytest (default)",
            ]
        );
    }

    #[test]
    fn lookup_splits_a_known_key() {
        let registry = registry();

        let (provider, name) = lookup(&registry, "python.formatter").unwrap();

        assert_eq!(provider.name(), "python");
        assert_eq!(name, "formatter");
    }

    #[test]
    fn lookup_rejects_a_key_without_a_language() {
        let registry = registry();
        let result = lookup(&registry, "formatter");

        assert!(matches!(
            result,
            Err(AppError::Config(ConfigError::MalformedKey(_)))
        ));
    }

    #[test]
    fn lookup_rejects_an_unknown_language() {
        let registry = registry();
        let result = lookup(&registry, "cobol.formatter");

        assert!(matches!(result, Err(AppError::Cli(_))));
    }

    #[test]
    fn lookup_rejects_an_unknown_key() {
        let registry = registry();
        let result = lookup(&registry, "python.formattr");

        assert!(matches!(
            result,
            Err(AppError::Config(ConfigError::UnknownKey(key))) if key == "python.formattr"
        ));
    }
}
