mod phong;
mod solid;

pub use phong::{DrawPhong, PhongShader};
pub use solid::{DrawSolid, SolidShader};

pub enum ShaderType {
    Phong,
    Solid,
}
