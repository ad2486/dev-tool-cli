use std::collections::HashMap;

use crate::providers::Provider;

pub struct Registry {
    providers: HashMap<String, Box<dyn Provider>>,
}

impl Registry {
    fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }
    fn register(&mut self, provider: Box<dyn Provider>) {
        let name = provider.name().to_string();
        self.providers.insert(name, provider);
    }
    fn get(&self, name: &str) -> Option<&dyn Provider> {
        self.providers.get(name).map(|provider| provider.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use std::result;

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

        fn doctor(&self, ctx: &InstallContext) -> anyhow::Result<Report> {
            Ok(Report)
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
}
