use std::boxed::Box;
use std::sync::Arc;

use cgmath::Matrix4;

use vulkano::buffer::{BufferUsage, CpuBufferPool};
use vulkano::descriptor::descriptor_set::{FixedSizeDescriptorSetBuilder,
    FixedSizeDescriptorSetsPool};
use vulkano::descriptor::descriptor_set::DescriptorSet;
use vulkano::framebuffer::{Subpass, RenderPassAbstract};
use vulkano::pipeline::vertex::OneVertexOneInstanceDefinition;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};

use RenderInstance;
use RenderVertex;
use cache::TextureCacheEntry;
use context::Context;
use pipeline;
use pipeline::Pipeline;
use scene::Light;

pub struct LinePipeline {
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    descriptor_sets_pool: FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Send + Sync>>,
    matrix_buffers: CpuBufferPool<pipeline::vs::ty::MatrixData>,
    light_buffers: CpuBufferPool<pipeline::vs::ty::LightData>
}

impl Pipeline for LinePipeline {
    fn get_pipeline(&self) -> Arc<GraphicsPipelineAbstract + Send + Sync> {
        self.pipeline.clone()
    }

    fn get_descriptor_set(&mut self,
            projection: Matrix4<f32>, light: &Light, texture: &TextureCacheEntry)
            -> Arc<DescriptorSet + Send + Sync> {
        let matrix_data = pipeline::vs::ty::MatrixData {
            projection: projection.into(),
        };

        let light_dir = *light.direction();
        let light_color = *light.color();
        let light_ambient = *light.ambient();

        let light_data = pipeline::vs::ty::LightData {
            light_dir: light_dir.into(),
            light_color: light_color.into(),
            light_ambient: light_ambient.into(),
        };

        let matrix_buffer = self.matrix_buffers.next(matrix_data).unwrap();
        let light_buffer = self.light_buffers.next(light_data).unwrap();

        let set = self.descriptor_sets_pool.next()
            .add_buffer(matrix_buffer).unwrap()
            .add_buffer(light_buffer).unwrap()
            .add_sampled_image(texture.image(), texture.sampler()).unwrap()
            .build().unwrap();

        Arc::new(set)
    }
}

impl LinePipeline {
    pub fn new<R>(context: Arc<Context>,
            subpass: Subpass<R>) -> Box<Pipeline>
            where R: RenderPassAbstract + Send + Sync + 'static {
        let vshader = pipeline::vs::Shader::load(context.device()).expect("error");
        let fshader = pipeline::fs::Shader::load(context.device()).expect("error");

        let pipeline = Arc::new(GraphicsPipeline::start()
            .vertex_input(OneVertexOneInstanceDefinition::<RenderVertex, RenderInstance>::new())
            .vertex_shader(vshader.main_entry_point(), ())
            .line_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fshader.main_entry_point(), ())
            .blend_alpha_blending()
            .depth_stencil_simple_depth()
            .cull_mode_back()
            .render_pass(subpass)
            .build(context.device()).unwrap())
            as Arc<GraphicsPipelineAbstract + Send + Sync>;

        let descriptor_sets_pool = FixedSizeDescriptorSetsPool::new(pipeline.clone(), 0);

        let matrix_buffers = CpuBufferPool::<pipeline::vs::ty::MatrixData>::new(
            context.device(), BufferUsage::uniform_buffer());
        let light_buffers = CpuBufferPool::<pipeline::vs::ty::LightData>::new(
            context.device(), BufferUsage::uniform_buffer());

        Box::new(LinePipeline {
            pipeline: Arc::new(pipeline),
            descriptor_sets_pool: descriptor_sets_pool,
            matrix_buffers: matrix_buffers,
            light_buffers: light_buffers
        })
    }

    pub fn pipeline(&self) -> Arc<GraphicsPipelineAbstract + Send + Sync> {
        self.pipeline.clone()
    }

    pub fn descriptor_builder(&mut self)
    -> FixedSizeDescriptorSetBuilder<Arc<GraphicsPipelineAbstract + Send + Sync>, ()> {
        self.descriptor_sets_pool.next()
    }
}
