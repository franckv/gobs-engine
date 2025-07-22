mod ui;

use gobs_render::RenderError;
use thiserror::Error;

pub use ui::UIRenderer;

#[derive(Debug, Error)]
pub enum UIError {
    #[error("render error")]
    RenderError(#[from] RenderError),
}
