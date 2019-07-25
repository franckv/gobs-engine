pub mod line;
pub mod triangle;

use std::sync::Arc;

use cgmath::Matrix4;

use vulkano::descriptor::descriptor_set::DescriptorSet;
use vulkano::pipeline::GraphicsPipelineAbstract;

use cache::TextureCacheEntry;
use scene::Light;

pub use self::line::LinePipeline;
pub use self::triangle::TrianglePipeline;

pub trait Pipeline: Send {
    fn get_pipeline(&self) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync>;

    fn get_descriptor_set(&mut self,
        projection: Matrix4<f32>, light: &Light, texture: &TextureCacheEntry)
        -> Arc<dyn DescriptorSet + Send + Sync>;
}

pub mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "src/pipeline/shader/vertex.glsl"
    }

    #[cfg(debug_assertions)]
    fn _reload() {
        include_bytes!("shader/vertex.glsl");
    }
}

pub mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "src/pipeline/shader/fragment.glsl"
    }

    #[cfg(debug_assertions)]
    fn _reload() {
        include_bytes!("shader/fragment.glsl");
    }
}
