mod ui;

use thiserror::Error;

use gobs_render_graph::RenderError;

pub use ui::UIRenderer;

#[derive(Debug, Error)]
pub enum UIError {
    #[error("render error")]
    RenderError(#[from] RenderError),
}
