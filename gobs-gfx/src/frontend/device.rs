use std::sync::Arc;

use anyhow::Result;

use crate::backend::{GfxCommand, GfxDisplay, GfxInstance};

pub trait Device {
    fn new(instance: Arc<GfxInstance>, display: Arc<GfxDisplay>) -> Result<Arc<Self>>;
    fn run_immediate<F>(&self, callback: F)
    where
        F: Fn(&GfxCommand);
    fn run_immediate_mut<F>(&self, callback: F)
    where
        F: FnMut(&GfxCommand);
}
