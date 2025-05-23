pub mod app;
pub mod context;

use thiserror::Error;

use gobs_egui::UIError;
use gobs_render_graph::RenderError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("render error")]
    RenderError(#[from] RenderError),
    #[error("ui error")]
    UIError(#[from] UIError),
}
