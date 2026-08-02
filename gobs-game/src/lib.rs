mod app;
mod context;

use thiserror::Error;

use gobs_egui::UIError;
use gobs_render::RenderError;

pub use app::{Application, GobsGame};
pub use context::{AppInfo, GameContext, GobsContext};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("render error")]
    RenderError(#[from] RenderError),
    #[error("ui error")]
    UIError(#[from] UIError),
}
