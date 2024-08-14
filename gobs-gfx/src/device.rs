use std::sync::Arc;

use anyhow::Result;

pub trait Device {
    type GfxDisplay;
    type GfxInstance;
    type GfxCommand;

    fn new(instance: Arc<Self::GfxInstance>, display: &Self::GfxDisplay) -> Result<Arc<Self>>;
    fn run_immediate<F>(&self, callback: F)
    where
        F: Fn(&Self::GfxCommand);
    fn run_immediate_mut<F>(&self, callback: F)
    where
        F: FnMut(&Self::GfxCommand);
    fn wait(&self);
}
