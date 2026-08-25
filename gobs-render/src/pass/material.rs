use gobs_core::logger;
use gobs_render_graph::{
    FrameData, GraphResourceManager, PassMetaData, SceneData, SceneDataLayout, SceneDataProp,
};
use gobs_render_hal::{
    AttributeData, BindingGroupLayout, BindingGroupType, CommandBuffer, Handle, RenderHAL,
    UniformBuffer, UniformData as _,
};
use gobs_vulkan::{DescriptorStage, DescriptorType};

#[cfg(debug_assertions)]
use crate::render_object::RenderObject;
use crate::{GfxContext, RenderError, RenderFlags, job::RenderJob};

pub struct MaterialPassData {
    pub(crate) pipeline: Option<Handle>,
    pub(crate) render_flags: RenderFlags,
    pub(crate) uniform_buffer: Vec<UniformBuffer>,
    scene_layout: SceneDataLayout,
}

impl MaterialPassData {
    pub fn new(
        ctx: &mut GfxContext,
        pass_metadata: &PassMetaData,
        pipeline: Option<Handle>,
        render_flags: RenderFlags,
        scene_layout: SceneDataLayout,
        frames_in_flight: usize,
    ) -> Self {
        let label = format!("Scene data {}", pass_metadata.name());

        let uniform_buffer = (0..frames_in_flight)
            .map(|_| {
                let uniform_bindgroup = BindingGroupLayout::new(BindingGroupType::SceneData)
                    .add_binding(DescriptorType::Uniform, DescriptorStage::All, 1);

                UniformBuffer::new(
                    &label,
                    ctx.hal_mut(),
                    uniform_bindgroup,
                    scene_layout.uniform_layout(),
                )
            })
            .collect();

        Self {
            pipeline,
            render_flags,
            uniform_buffer,
            scene_layout,
        }
    }

    pub fn update_uniform(&self, ctx: &mut GfxContext, frame_id: usize, uniform_data: &[u8]) {
        self.uniform_buffer[frame_id].update(ctx.hal_mut(), uniform_data);
    }
}

pub struct MaterialPass;

impl MaterialPass {
    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn begin_pass(
        hal: &dyn RenderHAL,
        cmd: &mut dyn CommandBuffer,
        pass_metadata: &PassMetaData,
        resource_manager: &GraphResourceManager,
    ) {
        tracing::debug!(target: logger::RENDER, "Begin material pass {}", pass_metadata.name);

        cmd.begin_label(&format!("Draw {}", pass_metadata.name));

        let (color_img, color_clear, color_extent) = match pass_metadata.color_attachments.first() {
            Some(color) => {
                let color_attach = &pass_metadata.attachments[color];
                (
                    Some(resource_manager.image(color)),
                    color_attach.clear,
                    Some(color_attach.scaled_extent()),
                )
            }
            None => (None, false, None),
        };

        let (depth_img, depth_clear, depth_extent) = match pass_metadata.depth_attachments.first() {
            Some(depth) => {
                let depth_attach = &pass_metadata.attachments[depth];
                (
                    Some(resource_manager.image(depth)),
                    depth_attach.clear,
                    Some(depth_attach.scaled_extent()),
                )
            }
            None => (None, false, None),
        };

        let extent = color_extent.unwrap_or_else(|| depth_extent.unwrap());

        cmd.begin_rendering(
            hal,
            color_img,
            extent,
            depth_img,
            color_clear,
            depth_clear,
            [0.; 4],
            1.,
        );

        cmd.set_viewport(extent.width, extent.height);
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn end_pass(cmd: &mut dyn CommandBuffer) {
        cmd.end_rendering();
        cmd.end_label();
    }

    /*
    #[cfg(debug_assertions)]
    fn validate_scene_layout(
        render_job: &RenderJob,
        scene_layout: &SceneDataLayout,
        render_list: &[RenderObject],
    ) {
        for obj in render_list {
            if render_job.should_render(obj)
                && let Some(material_scene_layout) = &obj.material.scene_layout
            {
                assert_eq!(
                    scene_layout,
                    material_scene_layout,
                    "Validate pass scene layout = obj scene layout for pass {}",
                    render_job.pass_name()
                );
            }
        }
    }
    */
}

impl MaterialPass {
    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn render(
        ctx: &mut GfxContext,
        pass_data: &mut MaterialPassData,
        pass_metadata: &PassMetaData,
        frame: &mut FrameData,
        resource_manager: &GraphResourceManager,
        render_list: &[RenderObject],
        scene_data: &SceneData,
    ) -> Result<(), RenderError> {
        tracing::debug!(target: logger::RENDER, "Draw {}", pass_metadata.name());

        Self::begin_pass(
            ctx.hal(),
            frame.command.as_mut(),
            pass_metadata,
            resource_manager,
        );

        tracing::debug!(target: logger::RENDER, "Upload scene data");
        let mut scene_data_bytes = Vec::new();

        tracing::debug!(target: logger::RENDER, "Scene data layout: {:?}", pass_data.scene_layout.uniform_layout());

        // #[cfg(debug_assertions)]
        // if !render_job.has_pipeline() {
        //     Self::validate_scene_layout(render_job, &self.scene_layout, render_list);
        // };

        pass_data
            .scene_layout
            .copy_data(&mut scene_data_bytes, |prop| match prop {
                SceneDataProp::CameraPosition => {
                    AttributeData::Vec3F(scene_data.camera_transform.translation().into())
                }
                SceneDataProp::CameraViewProj => {
                    let uniform_data = scene_data
                        .camera
                        .view_proj(scene_data.camera_transform.translation())
                        .to_cols_array_2d();

                    AttributeData::Mat4F(uniform_data)
                }
                SceneDataProp::CameraViewPort => AttributeData::Vec2F(scene_data.extent.into()),
                SceneDataProp::LightDirection => AttributeData::Vec3F(
                    scene_data
                        .light_transform
                        .expect("No lights in scene")
                        .translation()
                        .normalize()
                        .into(),
                ),
                SceneDataProp::LightColor => AttributeData::Vec4F(
                    scene_data.light.expect("No lights in scene").colour.into(),
                ),
                SceneDataProp::LightAmbientColor => AttributeData::Vec4F([0.1, 0.1, 0.1, 1.]),
            });

        tracing::debug!(target: logger::RENDER, "Update uniform (scene data, push)");
        pass_data.update_uniform(ctx, frame.id, &scene_data_bytes);

        tracing::debug!(target: logger::RENDER, "Start render job");
        let mut render_job = RenderJob::new()
            .with_pipeline(pass_data.pipeline)
            .with_scene_buffer(&pass_data.uniform_buffer[frame.id]);

        tracing::debug!(target: logger::RENDER, "Draw render object list");
        render_job.draw_list(
            ctx,
            frame,
            pass_metadata.name(),
            render_list,
            pass_data.render_flags,
        )?;

        tracing::debug!(target: logger::RENDER, "Stop render job");

        Self::end_pass(frame.command.as_mut());

        Ok(())
    }
}
