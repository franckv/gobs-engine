pub mod app;
pub mod context;
pub mod options;

use thiserror::Error;

use gobs_egui::UIError;
use gobs_render::RenderError;

pub use app::{Application, Run};
pub use context::{AppInfo, GameContext};
pub use options::GameOptions;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("render error")]
    RenderError(#[from] RenderError),
    #[error("ui error")]
    UIError(#[from] UIError),
}
