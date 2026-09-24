use crate::config::{self, ConfigTable};
use crate::core::Registry;
use crate::os::{CommandRunner, OsAdapter};
use crate::providers::InstallContext;
use anyhow::Result;
use std::rc::Rc;

pub fn run(
    registry: &Registry,
    config: &ConfigTable,
    os_adapter: Rc<dyn OsAdapter>,
    command_runner: Rc<dyn CommandRunner>,
) -> Result<()> {
    for provider in registry.all() {
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
