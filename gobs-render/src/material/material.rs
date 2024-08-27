use std::sync::Arc;

use gobs_gfx::{
    BindingGroupType, BlendMode, CompareOp, CullMode, DescriptorStage, DescriptorType,
    DynamicStateElem, FrontFace, GraphicsPipelineBuilder, Pipeline, Rect2D, Renderer, Viewport,
};
use uuid::Uuid;

use gobs_resource::{geometry::VertexFlag, material::Texture};

use crate::{context::Context, material::MaterialInstance, pass::RenderPass};

pub type MaterialId = Uuid;

pub struct Material<R: Renderer> {
    pub id: MaterialId,
    pub vertex_flags: VertexFlag,
    pub pipeline: Arc<R::Pipeline>,
    pub blending_enabled: bool,
}

impl<R: Renderer> Material<R> {
    pub fn builder(
        ctx: &Context<R>,
        vertex_shader: &str,
        fragment_shader: &str,
    ) -> MaterialBuilder<R> {
        MaterialBuilder::new(ctx, vertex_shader, fragment_shader)
    }

    pub fn instantiate(self: &Arc<Self>, textures: Vec<Arc<Texture>>) -> Arc<MaterialInstance<R>> {
        MaterialInstance::new(self.clone(), textures)
    }
}

pub enum MaterialProperty {
    Texture,
}

pub struct MaterialBuilder<R: Renderer> {
    vertex_flags: VertexFlag,
    blend_mode: BlendMode,
    pipeline_builder: R::GraphicsPipelineBuilder,
}

impl<R: Renderer> MaterialBuilder<R> {
    pub fn new(ctx: &Context<R>, vertex_shader: &str, fragment_shader: &str) -> Self {
        let pipeline_builder = R::Pipeline::graphics("material", &ctx.device)
            .vertex_shader(vertex_shader, "main")
            .fragment_shader(fragment_shader, "main")
            .pool_size(ctx.frames_in_flight + 1)
            .viewports(vec![Viewport::new(0., 0., 0., 0.)])
            .scissors(vec![Rect2D::new(0, 0, 0, 0)])
            .dynamic_states(&vec![DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .attachments(Some(ctx.color_format), Some(ctx.depth_format))
            .depth_test_enable(false, CompareOp::LessEqual)
            .front_face(FrontFace::CCW)
            .binding_group(BindingGroupType::SceneData)
            .binding(DescriptorType::Uniform, DescriptorStage::All);

        Self {
            vertex_flags: VertexFlag::empty(),
            blend_mode: BlendMode::None,
            pipeline_builder,
        }
    }

    pub fn vertex_flags(mut self, vertex_flags: VertexFlag) -> Self {
        self.vertex_flags = vertex_flags;

        self
    }

    pub fn no_culling(mut self) -> Self {
        self.pipeline_builder = self.pipeline_builder.cull_mode(CullMode::None);

        self
    }

    pub fn cull_mode(mut self, cull_mode: CullMode) -> Self {
        self.pipeline_builder = self.pipeline_builder.cull_mode(cull_mode);

        self
    }

    pub fn blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.pipeline_builder = self.pipeline_builder.blending_enabled(blend_mode);
        self.blend_mode = blend_mode;

        self
    }

    pub fn prop(mut self, _name: &str, prop: MaterialProperty) -> Self {
        if self.pipeline_builder.current_binding_group() != Some(BindingGroupType::MaterialData) {
            self.pipeline_builder = self
                .pipeline_builder
                .binding_group(BindingGroupType::MaterialData);
        }

        match prop {
            MaterialProperty::Texture => {
                self.pipeline_builder = self
                    .pipeline_builder
                    .binding(DescriptorType::SampledImage, DescriptorStage::Fragment)
                    .binding(DescriptorType::Sampler, DescriptorStage::Fragment);
            }
        }

        self
    }

    pub fn build(self, pass: Arc<dyn RenderPass<R>>) -> Arc<Material<R>> {
        let pipeline_builder = match pass.push_layout() {
            Some(push_layout) => self.pipeline_builder.push_constants(push_layout.size()),
            None => self.pipeline_builder,
        };

        let pipeline = pipeline_builder.build();

        Arc::new(Material {
            id: Uuid::new_v4(),
            vertex_flags: self.vertex_flags,
            pipeline,
            blending_enabled: self.blend_mode != BlendMode::None,
        })
    }
}
