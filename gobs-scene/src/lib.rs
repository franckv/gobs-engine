pub mod camera;
pub mod data;
pub mod model;
pub mod scene;

pub use model::{Color, Texture};
pub use scene::camera::Camera;
pub use scene::coord::SphericalCoord;
pub use scene::light::{Light, LightBuilder};
pub use scene::scenegraph::SceneGraph;
