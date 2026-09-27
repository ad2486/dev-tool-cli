use crate::config::{self, ConfigTable};
use crate::core::Registry;
use crate::errors::AppError;
use crate::os::{CommandRunner, OsAdapter};
use crate::providers::InstallContext;
use std::rc::Rc;

pub fn run(
    registry: &Registry,
    config: &ConfigTable,
    os_adapter: Rc<dyn OsAdapter>,
    command_runner: Rc<dyn CommandRunner>,
    language: &str,
) -> Result<(), AppError> {
    let provider = registry.find(language)?;

    let defaults = provider.manifest().tools.clone();
    let resolved = config::resolve(config, provider.name(), &defaults)?;

    let ctx = InstallContext {
        config: resolved,
        os_adapter,
        command_runner,
    };

    provider.install(&ctx)?;
    Ok(())
}
