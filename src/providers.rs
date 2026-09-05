mod manifest;
use anyhow::Result;
use std::{collections::HashMap, rc::Rc};
use crate::os::{OsAdapter, CommandRunner};
pub use manifest::Manifest;
pub struct InstallContext {
    pub config: HashMap<String, String>,
    pub os_adapter: Rc<dyn OsAdapter>,
    pub command_runner: Rc<dyn CommandRunner>,
}
pub struct Report;
pub struct InitOpts;
pub trait Provider {
    fn name(&self) -> &str;
    fn manifest(&self) -> &Manifest;
    fn install(&self, ctx: &InstallContext) -> Result<()>;
    fn doctor(&self, ctx: &InstallContext) -> Result<Report>;
    fn init(&self, ctx: &InstallContext, opts: InitOpts) -> Result<()>;
}
