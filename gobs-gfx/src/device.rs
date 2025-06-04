use std::sync::Arc;

use crate::{GfxError, Renderer};

pub trait Device<R: Renderer> {
    fn new(instance: Arc<R::Instance>, display: &R::Display) -> Result<Arc<Self>, GfxError>;
    fn wait(&self);
}
