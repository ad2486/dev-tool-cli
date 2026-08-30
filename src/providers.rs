mod manifest;
use anyhow::Result;
pub use manifest::Manifest;
pub struct InstallContext;
pub struct Report;
pub struct InitOpts;
pub trait Provider {
    fn name(&self) -> &str;
    fn manifest(&self) -> &Manifest;
    fn install(&self, ctx: &InstallContext) -> Result<()>;
    fn doctor(&self, ctx: &InstallContext) -> Result<Report>;
    fn init(&self, ctx: &InstallContext, opts: InitOpts) -> Result<()>;
}
