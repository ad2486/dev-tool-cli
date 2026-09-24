use std::collections::BTreeMap;

use crate::providers::Provider;

pub struct Registry {
    providers: BTreeMap<String, Box<dyn Provider>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            providers: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn Provider>) {
        let name = provider.name().to_string();
        self.providers.insert(name, provider);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Provider> {
        self.providers.get(name).map(|provider| provider.as_ref())
    }

    pub fn all(&self) -> Vec<&dyn Provider> {
        self.providers
            .values()
            .map(|provider| provider.as_ref())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{InitOpts, InstallContext, Manifest, Report};

    struct FakeProvider {
        name: String,
        manifest: Manifest,
    }

    impl Provider for FakeProvider {
        fn name(&self) -> &str {
            &self.name
        }

        fn manifest(&self) -> &Manifest {
            &self.manifest
        }

        fn install(&self, _ctx: &InstallContext) -> anyhow::Result<()> {
            Ok(())
        }

        fn doctor(&self, _ctx: &InstallContext) -> anyhow::Result<Report> {
            Ok(Report::new(self.name.clone(), vec![]))
        }

        fn init(&self, _ctx: &InstallContext, _opts: InitOpts) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn register_and_get_by_name() {
        let mut registry = Registry::new();
        let provider = FakeProvider {
            name: "python".to_string(),
            manifest: Manifest::default(),
        };

        registry.register(Box::new(provider));

        let result = registry.get("python");

        assert!(result.is_some());
    }

    #[test]
    fn get_missing_provider_returns_none() {
        let registry = Registry::new();
        let result = registry.get("python");

        assert!(result.is_none());
    }

    #[test]
    fn all_returns_providers_ordered_by_name() {
        let mut registry = Registry::new();
        for name in ["rust", "docker", "python"] {
            registry.register(Box::new(FakeProvider {
                name: name.to_string(),
                manifest: Manifest::default(),
            }));
        }

        let names: Vec<&str> = registry.all().iter().map(|p| p.name()).collect();

        assert_eq!(names, vec!["docker", "python", "rust"]);
    }
}
