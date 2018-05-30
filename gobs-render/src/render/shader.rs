use std::boxed::Box;
use std::sync::Arc;

use cgmath::Matrix4;

use vulkano::buffer::{BufferUsage, CpuBufferPool};
use vulkano::descriptor::descriptor_set::{FixedSizeDescriptorSetBuilder,
    FixedSizeDescriptorSetsPool};
use vulkano::descriptor::descriptor_set::DescriptorSet;
use vulkano::framebuffer::{Subpass, RenderPassAbstract};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::pipeline::vertex::OneVertexOneInstanceDefinition;

use RenderInstance;
use RenderVertex;
use cache::TextureCacheEntry;
use context::Context;
use scene::Light;
use scene::model::PrimitiveType;

mod vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[path = "src/render/shader/vertex.glsl"]
    struct _Dummy;

    #[cfg(debug_assertions)]
    fn _reload() {
        include_bytes!("shader/vertex.glsl");
    }
}

mod fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[path = "src/render/shader/fragment.glsl"]
    struct _Dummy;

    #[cfg(debug_assertions)]
    fn _reload() {
        include_bytes!("shader/fragment.glsl");
    }
}

pub trait Shader {
    fn get_pipeline(&mut self, render_pass: Arc<RenderPassAbstract + Send + Sync>,
        primitive: PrimitiveType) -> Arc<GraphicsPipelineAbstract + Send + Sync>;

    fn get_descriptor_set(&mut self, render_pass: Arc<RenderPassAbstract + Send + Sync>,
        projection: Matrix4<f32>, light: &Light, texture: &TextureCacheEntry,
        primitive: PrimitiveType) -> Arc<DescriptorSet + Send + Sync>;
}

pub struct Pipeline {
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    descriptor_sets_pool: FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Send + Sync>>
}

impl Pipeline {
    pub fn new(pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>) -> Self {
        let descriptor_sets_pool = FixedSizeDescriptorSetsPool::new(pipeline.clone(), 0);

        Pipeline {
            pipeline: pipeline,
            descriptor_sets_pool: descriptor_sets_pool
        }
    }

    pub fn pipeline(&self) -> Arc<GraphicsPipelineAbstract + Send + Sync> {
        self.pipeline.clone()
    }

    pub fn descriptor_builder(&mut self)
    -> FixedSizeDescriptorSetBuilder<Arc<GraphicsPipelineAbstract + Send + Sync>, ()> {
        self.descriptor_sets_pool.next()
    }
}

pub struct DefaultShader {
    context: Arc<Context>,
    pipeline: Option<Pipeline>,
    pipeline_line: Option<Pipeline>,
    matrix_buffers: CpuBufferPool<vs::ty::MatrixData>,
    light_buffers: CpuBufferPool<vs::ty::LightData>
}

impl DefaultShader {
    pub fn new(context: Arc<Context>) -> Box<Shader> {
        let matrix_buffers = CpuBufferPool::<vs::ty::MatrixData>::new(
            context.device(), BufferUsage::uniform_buffer());
        let light_buffers = CpuBufferPool::<vs::ty::LightData>::new(
            context.device(), BufferUsage::uniform_buffer());

        Box::new(DefaultShader {
            context: context,
            pipeline: None,
            pipeline_line: None,
            matrix_buffers: matrix_buffers,
            light_buffers: light_buffers
        })
    }

    fn create_pipeline(context: Arc<Context>,
        render_pass: Arc<RenderPassAbstract + Send + Sync>, primitive: PrimitiveType)
        -> Pipeline {
        let vshader = vs::Shader::load(context.device()).expect("error");
        let fshader = fs::Shader::load(context.device()).expect("error");

        let mut builder = GraphicsPipeline::start()
            .vertex_input(OneVertexOneInstanceDefinition::<RenderVertex, RenderInstance>::new())
            .vertex_shader(vshader.main_entry_point(), ());

        builder = match primitive {
            PrimitiveType::Triangle => builder,
            PrimitiveType::Line => builder.line_list(),
        };

        Pipeline::new(Arc::new(builder
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fshader.main_entry_point(), ())
            .blend_alpha_blending()
            .depth_stencil_simple_depth()
            .cull_mode_back()
            .render_pass(Subpass::from(render_pass, 0).unwrap())
            .build(context.device()).unwrap()))
    }

    fn init_pipelines(&mut self, render_pass: Arc<RenderPassAbstract + Send + Sync>) {
        if self.pipeline.is_none() {
            self.pipeline = Some(Self::create_pipeline(self.context.clone(),
                render_pass.clone(), PrimitiveType::Triangle));
        }

        if self.pipeline_line.is_none() {
            self.pipeline_line = Some(Self::create_pipeline(self.context.clone(),
                render_pass, PrimitiveType::Line));
        }
    }

    fn pipeline(&mut self, render_pass: Arc<RenderPassAbstract + Send + Sync>,
        primitive: PrimitiveType) -> &Pipeline {
        self.init_pipelines(render_pass);

        match primitive {
            PrimitiveType::Triangle => self.pipeline.as_ref().unwrap(),
            PrimitiveType::Line => self.pipeline_line.as_ref().unwrap(),
        }
    }

    fn pipeline_mut(&mut self, render_pass: Arc<RenderPassAbstract + Send + Sync>,
        primitive: PrimitiveType) -> &mut Pipeline {
        self.init_pipelines(render_pass);

        match primitive {
            PrimitiveType::Triangle => self.pipeline.as_mut().unwrap(),
            PrimitiveType::Line => self.pipeline_line.as_mut().unwrap(),
        }
    }
}

impl Shader for DefaultShader {
    fn get_pipeline(&mut self, render_pass: Arc<RenderPassAbstract + Send + Sync>,
        primitive: PrimitiveType) -> Arc<GraphicsPipelineAbstract + Send + Sync> {
        self.pipeline(render_pass, primitive).pipeline()
    }

    fn get_descriptor_set(&mut self, render_pass: Arc<RenderPassAbstract + Send + Sync>,
        projection: Matrix4<f32>, light: &Light, texture: &TextureCacheEntry,
        primitive: PrimitiveType) -> Arc<DescriptorSet + Send + Sync> {
        let matrix_data = vs::ty::MatrixData {
            projection: projection.into(),
        };

        let light_dir = *light.direction();
        let light_color = *light.color();
        let light_ambient = *light.ambient();

        let light_data = vs::ty::LightData {
            light_dir: light_dir.into(),
            light_color: light_color.into(),
            light_ambient: light_ambient.into(),
        };

        let matrix_buffer = self.matrix_buffers.next(matrix_data).unwrap();
        let light_buffer = self.light_buffers.next(light_data).unwrap();

        let set = self.pipeline_mut(render_pass, primitive).descriptor_builder()
            .add_buffer(matrix_buffer).unwrap()
            .add_buffer(light_buffer).unwrap()
            .add_sampled_image(texture.image(), texture.sampler()).unwrap()
            .build().unwrap();

        Arc::new(set)
    }
}
