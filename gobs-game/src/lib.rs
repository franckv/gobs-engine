pub mod app;

use gobs_render::RenderError;
use thiserror::Error;

use gobs_egui::UIError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("render error")]
    RenderError(#[from] RenderError),
    #[error("ui error")]
    UIError(#[from] UIError),
}
