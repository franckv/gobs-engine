use std::sync::Arc;

use crate::{GfxError, Renderer};

pub trait Device<R: Renderer> {
    fn new(instance: Arc<R::Instance>, display: &R::Display) -> Result<Arc<Self>, GfxError>;
    fn run_transfer<F>(&self, callback: F)
    where
        F: Fn(&R::Command);
    fn run_transfer_mut<F>(&self, callback: F)
    where
        F: FnMut(&R::Command);

    fn run_immediate<F>(&self, callback: F)
    where
        F: Fn(&R::Command);
    fn run_immediate_mut<F>(&self, callback: F)
    where
        F: FnMut(&R::Command);
    fn wait(&self);
    fn wait_transfer(&self);
}
