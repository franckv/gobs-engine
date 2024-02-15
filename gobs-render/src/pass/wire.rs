use std::sync::Arc;

use glam::Mat3;
use uuid::Uuid;

use gobs_core::entity::uniform::{UniformData, UniformLayout, UniformProp, UniformPropData};
use gobs_utils::load;
use gobs_vulkan::{
    descriptor::{DescriptorSetLayout, DescriptorStage, DescriptorType},
    image::{Image, ImageExtent2D, ImageLayout},
    pipeline::{
        CullMode, DynamicStateElem, FrontFace, Pipeline, PipelineLayout, PolygonMode, Rect2D,
        Shader, ShaderType, Viewport,
    },
};

use crate::{
    context::Context,
    geometry::VertexFlag,
    graph::RenderError,
    pass::{PassId, PassType, RenderPass},
    renderable::{RenderObject, RenderStats},
    CommandBuffer,
};

pub struct WirePass {
    id: PassId,
    name: String,
    ty: PassType,
    pipeline: Arc<Pipeline>,
    vertex_flags: VertexFlag,
    push_layout: Arc<UniformLayout>,
}

impl WirePass {
    pub fn new(ctx: &Context, name: &str) -> Arc<dyn RenderPass> {
        let vertex_flags = VertexFlag::POSITION | VertexFlag::COLOR;

        let push_layout = UniformLayout::builder()
            .prop("world_matrix", UniformProp::Mat4F)
            .prop("normal_matrix", UniformProp::Mat3F)
            .prop("vertex_buffer_address", UniformProp::U64)
            .build();

        let scene_descriptor_layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::Uniform, DescriptorStage::All)
            .build(ctx.device.clone());

        let ds_layouts = vec![scene_descriptor_layout.clone()];

        let pipeline_layout =
            PipelineLayout::new(ctx.device.clone(), &ds_layouts, push_layout.size());

        let vertex_file = load::get_asset_dir("wire.vert.spv", load::AssetType::SHADER).unwrap();
        let vertex_shader = Shader::from_file(vertex_file, ctx.device.clone(), ShaderType::Vertex);

        let fragment_file = load::get_asset_dir("wire.frag.spv", load::AssetType::SHADER).unwrap();
        let fragment_shader =
            Shader::from_file(fragment_file, ctx.device.clone(), ShaderType::Fragment);

        let pipeline = Pipeline::graphics_builder(ctx.device.clone())
            .layout(pipeline_layout.clone())
            .polygon_mode(PolygonMode::Line)
            .vertex_shader("main", vertex_shader)
            .fragment_shader("main", fragment_shader)
            .viewports(vec![Viewport::new(0., 0., 0., 0.)])
            .scissors(vec![Rect2D::new(0, 0, 0, 0)])
            .dynamic_states(&vec![DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .attachments(ctx.color_format, Some(ctx.depth_format))
            .depth_test_disable()
            .cull_mode(CullMode::Back)
            .front_face(FrontFace::CCW)
            .build();

        Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Wire,
            pipeline,
            vertex_flags,
            push_layout,
        })
    }
}

impl RenderPass for WirePass {
    fn id(&self) -> PassId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn ty(&self) -> super::PassType {
        self.ty
    }

    fn pipeline(&self) -> Option<Arc<Pipeline>> {
        Some(self.pipeline.clone())
    }

    fn vertex_flags(&self) -> Option<VertexFlag> {
        Some(self.vertex_flags)
    }

    fn push_layout(&self) -> Option<Arc<UniformLayout>> {
        Some(self.push_layout.clone())
    }

    fn uniform_data_layout(&self) -> Option<Arc<UniformLayout>> {
        None
    }

    fn render(
        self: Arc<Self>,
        _ctx: &Context,
        cmd: &CommandBuffer,
        render_targets: &mut [&mut Image],
        draw_extent: ImageExtent2D,
        draw_cmd: &mut dyn FnMut(Arc<dyn RenderPass>, &CommandBuffer),
    ) -> Result<(), RenderError> {
        log::debug!("Draw wire");

        cmd.begin_label("Draw wire");

        cmd.transition_image_layout(&mut render_targets[0], ImageLayout::Color);

        cmd.begin_rendering(&render_targets[0], draw_extent, None, false, [0.; 4], 1.);

        cmd.set_viewport(draw_extent.width, draw_extent.height);

        draw_cmd(self, cmd);

        cmd.end_rendering();

        cmd.end_label();

        Ok(())
    }

    fn draw(
        &self,
        ctx: &Context,
        cmd: &CommandBuffer,
        render_list: &[RenderObject],
        _scene_data: Option<UniformData>,
        render_stats: &mut RenderStats,
    ) {
        let mut last_model = Uuid::nil();

        cmd.bind_pipeline(&self.pipeline);
        render_stats.binds += 1;

        for render_object in render_list {
            if render_object.pass.id() != self.id {
                continue;
            }
            let world_matrix = render_object.transform.matrix;
            let normal_matrix = Mat3::from_quat(render_object.transform.rotation);

            if let Some(push_layout) = render_object.pass.push_layout() {
                // TODO: hardcoded
                let model_data = UniformData::new(
                    &push_layout,
                    &[
                        UniformPropData::Mat4F(world_matrix.to_cols_array_2d()),
                        UniformPropData::Mat3F(normal_matrix.to_cols_array_2d()),
                        UniformPropData::U64(
                            render_object
                                .model
                                .vertex_buffer
                                .address(ctx.device.clone()),
                        ),
                    ],
                );
                cmd.push_constants(self.pipeline.layout.clone(), &model_data.raw());
            }

            if last_model != render_object.model.model.id {
                cmd.bind_index_buffer::<u32>(
                    &render_object.model.index_buffer,
                    render_object.indices_offset,
                );
                render_stats.binds += 1;
                last_model = render_object.model.model.id;
            }
            cmd.draw_indexed(render_object.indices_len, 1);
            render_stats.draws += 1;
        }
    }
}
