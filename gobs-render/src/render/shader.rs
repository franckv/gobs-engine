use std::sync::Arc;

use cgmath::Matrix4;

use vulkano::buffer::{BufferUsage, CpuBufferPool};
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::descriptor::descriptor_set::DescriptorSet;
use vulkano::device::Device;
use vulkano::framebuffer::{Subpass, RenderPassAbstract};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::pipeline::vertex::OneVertexOneInstanceDefinition;

use scene::light::Light;
use model::{Instance, PrimitiveType, Texture, Vertex};

mod vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[path = "src/render/shader/vertex.glsl"]
    struct _Dummy;
}

mod fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[path = "src/render/shader/fragment.glsl"]
    struct _Dummy;
}

// force recompilation if changed
fn _reload() {
    include_bytes!("shader/vertex.glsl");
    include_bytes!("shader/fragment.glsl");
}

pub struct DescriptorBuilder<'a> {
    shader: &'a mut Shader,
    matrix_data: Option<vs::ty::MatrixData>,
    light_data: Option<vs::ty::LightData>,
    texture: Option<Arc<Texture>>
}

impl<'a> DescriptorBuilder<'a> {
    fn new(shader: &'a mut Shader) -> Self {
        DescriptorBuilder {
            shader: shader,
            matrix_data: None,
            light_data: None,
            texture: None
        }
    }

    pub fn matrix(mut self, projection: Matrix4<f32>) -> Self {
        self.matrix_data = Some(vs::ty::MatrixData {
            projection: projection.into(),
        });

        self
    }

    pub fn light(mut self, light: &Light) -> Self {
        let light_dir = *light.direction();
        let light_color = *light.color();
        let light_ambient = *light.ambient();

        self.light_data = Some(vs::ty::LightData {
            light_dir: light_dir.into(),
            light_color: light_color.into(),
            light_ambient: light_ambient.into(),
        });

        self
    }

    pub fn texture(mut self, texture: Arc<Texture>) -> Self {
        self.texture = Some(texture);

        self
    }

    pub fn get(self) -> Arc<DescriptorSet + Send + Sync> {
        let matrix_data = self.matrix_data.unwrap();
        let light_data = self.light_data.unwrap();
        let texture = self.texture.unwrap();

        let matrix_buffer = self.shader.matrix_buffers.next(matrix_data).unwrap();
        let light_buffer = self.shader.light_buffers.next(light_data).unwrap();

        let set = self.shader.descriptor_sets_pool.next();

        let set = set.add_buffer(matrix_buffer).unwrap();
        let set = set.add_buffer(light_buffer).unwrap();
        let set = set.add_sampled_image(texture.image(), texture.sampler()).unwrap()
            .build().unwrap();

        Arc::new(set)
    }
}

pub struct Shader {
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    pipeline_line: Arc<GraphicsPipelineAbstract + Send + Sync>,
    descriptor_sets_pool: FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Send + Sync>>,
    matrix_buffers: CpuBufferPool<vs::ty::MatrixData>,
    light_buffers: CpuBufferPool<vs::ty::LightData>
}

impl Shader {
    pub fn new(render_pass: Arc<RenderPassAbstract + Send + Sync>, device: Arc<Device>)
    -> Shader {
        let pipeline = Self::create_pipeline(render_pass.clone(), device.clone(),
            PrimitiveType::Triangle);
        let pipeline_line = Self::create_pipeline(render_pass, device.clone(),
            PrimitiveType::Line);

        let descriptor_sets_pool = FixedSizeDescriptorSetsPool::new(pipeline.clone(), 0);

        let matrix_buffers = CpuBufferPool::<vs::ty::MatrixData>::new(device.clone(), BufferUsage::uniform_buffer());
        let light_buffers = CpuBufferPool::<vs::ty::LightData>::new(device.clone(), BufferUsage::uniform_buffer());

        Shader {
            pipeline: pipeline,
            pipeline_line: pipeline_line,
            descriptor_sets_pool: descriptor_sets_pool,
            matrix_buffers: matrix_buffers,
            light_buffers: light_buffers
        }
    }

    fn create_pipeline(render_pass: Arc<RenderPassAbstract + Send + Sync>,
        device: Arc<Device>, primitive: PrimitiveType)
        -> Arc<GraphicsPipelineAbstract + Send + Sync> {
        let vshader = vs::Shader::load(device.clone()).expect("error");
        let fshader = fs::Shader::load(device.clone()).expect("error");

        let mut builder = GraphicsPipeline::start()
            .vertex_input(OneVertexOneInstanceDefinition::<Vertex, Instance>::new())
            .vertex_shader(vshader.main_entry_point(), ());

        builder = match primitive {
            PrimitiveType::Triangle => builder,
            PrimitiveType::Line => builder.line_list(),
        };

        Arc::new(builder
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fshader.main_entry_point(), ())
            .blend_alpha_blending()
            .depth_stencil_simple_depth()
            .cull_mode_back()
            .render_pass(Subpass::from(render_pass, 0).unwrap())
            .build(device.clone()).unwrap())
    }

    pub fn pipeline(&self, primitive: PrimitiveType)
    -> Arc<GraphicsPipelineAbstract + Send + Sync> {
        match primitive {
            PrimitiveType::Triangle => self.pipeline.clone(),
            PrimitiveType::Line => self.pipeline_line.clone()
        }
    }

    pub fn bind(&mut self) -> DescriptorBuilder {
        DescriptorBuilder::new(self)
    }
}
