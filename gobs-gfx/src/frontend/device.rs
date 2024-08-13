use std::sync::Arc;

use anyhow::Result;

use crate::backend::{GfxCommand, GfxInstance};

use super::display::DisplayType;

pub trait Device {
    fn new(instance: Arc<GfxInstance>, display: &DisplayType) -> Result<Arc<Self>>;
    fn run_immediate<F>(&self, callback: F)
    where
        F: Fn(&GfxCommand);
    fn run_immediate_mut<F>(&self, callback: F)
    where
        F: FnMut(&GfxCommand);
    fn wait(&self);
}
