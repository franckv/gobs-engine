use std::collections::HashMap;

use crate::{
    FrameData, GraphConfig, PassMetaData, RenderError, RenderPassType,
    graph::{
        Barrier, BarrierAccess, BarrierStage,
        barrier::{SyncScope, SyncStatus},
        resource::GraphResourceManager,
    },
    pass::{Attachment, AttachmentAccess},
};
use gobs_core::{ImageExtent2D, logger};
use gobs_render_hal::{CommandBuffer, GfxContext, ImageLayout, RenderHAL};

pub struct FrameGraphPass {
    pub pass: PassMetaData,
    pub enabled: bool,
}

pub struct FrameGraph {
    pub render_scaling: f32,
    pub passes: Vec<FrameGraphPass>,
    pub attachments: HashMap<String, Attachment>,
    pub resource_manager: GraphResourceManager,
}

impl FrameGraph {
    pub fn new() -> Self {
        Self {
            render_scaling: 1.,
            passes: Vec::new(),
            attachments: HashMap::new(),
            resource_manager: GraphResourceManager::new(),
        }
    }

    pub fn load<F>(
        graph_filename: &str,
        graph_name: &str,
        default_extent: ImageExtent2D,
        pass_config: F,
    ) -> Result<Self, RenderError>
    where
        F: FnMut(&PassMetaData, RenderPassType),
    {
        tracing::debug!(target: logger::INIT, "Load graph: {}", graph_name);

        GraphConfig::load_graph(graph_filename, graph_name, default_extent, pass_config)
            .map_err(|_| RenderError::InvalidData)
    }

    pub fn register_pass(&mut self, pass: PassMetaData, enabled: bool) {
        let pass = FrameGraphPass { pass, enabled };

        self.passes.push(pass);
    }

    pub fn register_attachment(&mut self, label: &str, attachment: Attachment) {
        self.attachments.insert(label.to_string(), attachment);
    }

    pub fn allocate_attachments(&mut self, ctx: &mut GfxContext) {
        // TODO: image creation should be deferred to the renderer
        for (label, attachment) in &self.attachments {
            self.resource_manager.register_image(
                ctx,
                label,
                attachment.format,
                attachment.usage,
                attachment.extent,
            );
        }
    }

    fn transition_attachments(
        hal: &mut dyn RenderHAL,
        cmd: &mut dyn CommandBuffer,
        resource_manager: &GraphResourceManager,
        pass: &PassMetaData,
    ) {
        tracing::debug!(target: logger::SYNC, "Transition attachment for pass {}", &pass.name);
        for (name, attachment) in &pass.attachments {
            cmd.transition_image_layout(hal, resource_manager.image(name), attachment.layout);
        }
    }

    pub fn get_pass<F>(&self, cmp: F) -> Result<&PassMetaData, RenderError>
    where
        F: Fn(&PassMetaData) -> bool,
    {
        for pass in &self.passes {
            if cmp(&pass.pass) {
                return Ok(&pass.pass);
            }
        }

        Err(RenderError::PassNotFound)
    }

    pub fn pass_by_name(&self, pass_name: &str) -> Result<&PassMetaData, RenderError> {
        self.get_pass(|pass| pass.name == pass_name)
    }

    fn build_barriers(&self) -> Vec<Barrier> {
        let mut barriers = Vec::new();
        let mut attachments_status: HashMap<String, SyncStatus> = HashMap::new();

        for pass in &self.passes {
            if !pass.enabled {
                continue;
            }

            tracing::info!(target: logger::SYNC, "Generate barriers for pass {} [{}]", pass.pass.name(), pass.pass.id);

            for (attachment_name, attachment) in &pass.pass.attachments {
                let pass_id = pass.pass.id;
                let ty = attachment.ty;
                let access = attachment.access;
                let layout = attachment.layout;
                let scope = Barrier::barrier_scope(ty, access);

                if let Some(status) = attachments_status.get_mut(attachment_name) {
                    if status.last_layout() != layout {
                        // image layout transition barrier
                        let barrier = Barrier::new(attachment_name, pass_id)
                            .layouts(status.last_layout(), layout)
                            .memory(status.last_write(), scope);

                        barriers.push(barrier);

                        status.update(scope, layout);
                        status.clear_invalidates();
                        if access == AttachmentAccess::Read {
                            status.invalidate(scope);
                        }
                    } else {
                        match access {
                            AttachmentAccess::Read => {
                                if !status.is_flushed() {
                                    // RAW -> flush + invalidate
                                    let barrier = Barrier::new(attachment_name, pass_id)
                                        .layouts(status.last_layout(), layout)
                                        .memory(status.last_write(), scope);

                                    barriers.push(barrier);
                                    status.invalidate(scope);
                                } else if !status.is_invalidated(scope) {
                                    // RAW, already flushed ->  invalidate
                                    let barrier = Barrier::new(attachment_name, pass_id)
                                        .layouts(status.last_layout(), layout)
                                        .invalidation(status.last_write(), scope);

                                    barriers.push(barrier);
                                    status.invalidate(scope);
                                } else {
                                    // RAR, invalidated -> no barrier
                                }
                            }
                            AttachmentAccess::Write => {
                                if !status.is_flushed() {
                                    // WAW -> flush
                                    let barrier = Barrier::new(attachment_name, pass_id)
                                        .layouts(status.last_layout(), layout)
                                        .flush(status.last_write(), scope);

                                    barriers.push(barrier);
                                } else {
                                    // WAR -> execution barrier only
                                    let barrier = Barrier::new(attachment_name, pass_id)
                                        .layouts(status.last_layout(), layout)
                                        .execution(status.last_write(), scope);

                                    barriers.push(barrier);
                                }

                                status.update(scope, layout);
                                status.clear_invalidates();
                            }
                            AttachmentAccess::ReadWrite => {
                                if !status.is_flushed() {
                                    // WAW -> flush + invalidate
                                    let barrier = Barrier::new(attachment_name, pass_id)
                                        .layouts(status.last_layout(), layout)
                                        .memory(status.last_write(), scope);

                                    barriers.push(barrier);
                                } else if !status.is_invalidated(scope) {
                                    // WAR -> invalidate
                                    let barrier = Barrier::new(attachment_name, pass_id)
                                        .layouts(status.last_layout(), layout)
                                        .invalidation(status.last_write(), scope);

                                    barriers.push(barrier);
                                } else {
                                    // WAR -> flush
                                    let barrier = Barrier::new(attachment_name, pass_id)
                                        .layouts(status.last_layout(), layout)
                                        .execution(status.last_write(), scope);

                                    barriers.push(barrier);
                                }

                                status.update(scope, layout);
                                status.clear_invalidates();
                            }
                        }
                    }
                } else {
                    // first resource usage: do a transition from UNDEFINED
                    let barrier = Barrier::new(attachment_name, pass_id)
                        .layouts(ImageLayout::Undefined, layout)
                        .memory(
                            SyncScope {
                                stage: BarrierStage::TopOfPipe,
                                access: BarrierAccess::empty(),
                            },
                            scope,
                        );

                    attachments_status
                        .insert(attachment_name.to_string(), SyncStatus::new(scope, layout));

                    barriers.push(barrier);
                }
            }
        }

        barriers
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn begin(&mut self, ctx: &mut GfxContext) -> Result<(), RenderError> {
        // FIXME: use attachments from graph
        let draw_image_extent = ctx.get_image_extent(self.resource_manager.image("draw"));
        if self.resource_manager.resources.contains_key("depth") {
            debug_assert_eq!(
                draw_image_extent,
                ctx.get_image_extent(self.resource_manager.image("depth"))
            );
        }

        self.resource_manager.invalidate(ctx);

        Ok(())
    }

    fn run_pass<F>(
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        pass: &mut FrameGraphPass,
        resource_manager: &GraphResourceManager,
        mut run_pass_cb: F,
    ) -> Result<(), RenderError>
    where
        F: FnMut(
            &mut GfxContext,
            &mut FrameData,
            &GraphResourceManager,
            &PassMetaData,
        ) -> Result<(), RenderError>,
    {
        if !pass.enabled {
            tracing::debug!(target: logger::RENDER,
                "Skip pass: {}", &pass.pass.name);
            return Ok(());
        }

        let pass = &pass.pass;

        Self::transition_attachments(ctx, frame.command.as_mut(), resource_manager, pass);

        tracing::debug!(target: logger::SYNC, "Begin render pass {}", &pass.name);

        let span =
            tracing::span!(target: logger::PROFILE, tracing::Level::TRACE, "Pass", "{}", &pass.name)
            .entered();

        tracing::debug!(target: logger::RENDER, ">>> Begin rendering pass {}", &pass.name);

        run_pass_cb(ctx, frame, resource_manager, pass)?;

        tracing::debug!(target: logger::RENDER, "<<< End rendering pass {}", &pass.name);
        span.exit();

        tracing::debug!(target: logger::SYNC, "End render pass {}", &pass.name);

        Ok(())
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn end(&mut self, ctx: &mut GfxContext, frame: &mut FrameData) -> Result<(), RenderError> {
        let cmd = &mut frame.command;

        if let Some(render_target) = ctx.get_render_target() {
            cmd.transition_image_layout(ctx, render_target, ImageLayout::Present);
        } else {
            tracing::debug!(target: logger::RENDER, "No render target to present");
        }

        Ok(())
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn run<F>(
        &mut self,
        ctx: &mut GfxContext,
        frame: &mut FrameData,
        mut run_pass_cb: F,
    ) -> Result<(), RenderError>
    where
        F: FnMut(
            &mut GfxContext,
            &mut FrameData,
            &GraphResourceManager,
            &PassMetaData,
        ) -> Result<(), RenderError>,
    {
        self.begin(ctx)?;

        for pass in &mut self.passes {
            Self::run_pass(ctx, frame, pass, &self.resource_manager, &mut run_pass_cb)?;
        }

        self.end(ctx, frame)?;

        Ok(())
    }

    pub fn enable_pass(&mut self, name: &str, enabled: bool) {
        for pass in &mut self.passes {
            if pass.pass.name == name {
                pass.enabled = enabled;
            }
        }
    }
}

impl Default for FrameGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use gobs_render_hal::ImageLayout;
    use tracing::Level;
    use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

    use gobs_core::{ImageExtent2D, logger};

    use crate::{
        GraphConfig,
        graph::{BarrierAccess, BarrierStage, barrier::SyncScope},
    };

    const GRAPH: &str = r#"
        GraphConfig(
            graphes: {
                "graph_compute_raw": ["compute_writer1", "compute_reader1"],
                "graph_compute_raww": ["compute_writer1", "compute_writer2", "compute_reader12"],
                "graph_compute_war": ["compute_reader1", "compute_writer1"],
                "graph_compute_color": ["compute_writer1", "color_writer1"],
                "graph_compute_color_blend": ["compute_writer1", "color_blend_writer1"],
            },
            passes: {
                "compute_writer1": (ty: Compute, config: "test_c", attachments: { "draw": StorageImage(access: Write) }),
                "compute_writer2": (ty: Compute, config: "test_c", attachments: { "draw2": StorageImage(access: Write) }),
                "compute_reader1": (ty: Compute, config: "test_c", attachments: { "draw": StorageImage(access: Read) }),
                "compute_reader12": (ty: Compute, config: "test_c", attachments: { "draw": StorageImage(access: Read), "draw2": StorageImage(access: Read) }),
                "color_writer1": (ty: Material, config: "test", attachments: { "draw": ColorAttachment(access: Write, clear: false) }),
                "color_blend_writer1": (ty: Material, config: "test", attachments: { "draw": ColorAttachment(access: ReadWrite, clear: false) }),
                "depth_writer1": (ty: Material, config: "test", attachments: { "depth": DepthAttachment(access: ReadWrite, clear: true) }),
            },
            attachments: {
                "draw": (usage: Color, format: R8g8b8a8Unorm),
                "draw2": (usage: Color, format: R8g8b8a8Unorm),
                "depth": (usage: Depth, format: D32Sfloat),
            },
        )
    "#;

    fn setup() {
        let sub = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .with_span_events(FmtSpan::CLOSE)
            .finish();
        tracing::subscriber::set_global_default(sub).unwrap_or_default();
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_barrier_compute_raw() {
        setup();

        let graph = GraphConfig::load_graph_with_data(
            GRAPH,
            "graph_compute_raw",
            ImageExtent2D::default(),
            |_, _| {},
        )
        .unwrap();

        let barriers = graph.build_barriers();

        assert_eq!(barriers.len(), 2);

        for barrier in &barriers {
            tracing::info!(target: logger::SYNC, "Generate barrier: {:#?}", barrier);
            assert_eq!(barrier.attachment, "draw");
        }

        assert_eq!(barriers[0].src_layout, ImageLayout::Undefined);
        assert_eq!(barriers[0].dst_layout, ImageLayout::General);
        assert_eq!(
            barriers[0].src_scope,
            SyncScope {
                stage: BarrierStage::TopOfPipe,
                access: BarrierAccess::empty()
            }
        );
        assert_eq!(
            barriers[0].dst_scope,
            SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite
            }
        );

        assert_eq!(barriers[1].src_layout, ImageLayout::General);
        assert_eq!(barriers[1].dst_layout, ImageLayout::General);
        assert_eq!(
            barriers[1].src_scope,
            SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite
            }
        );
        assert_eq!(
            barriers[1].dst_scope,
            SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageRead
            }
        );
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_barrier_compute_raww() {
        setup();

        let graph = GraphConfig::load_graph_with_data(
            GRAPH,
            "graph_compute_raww",
            ImageExtent2D::default(),
            |_, _| {},
        )
        .unwrap();

        let barriers = graph.build_barriers();

        assert_eq!(barriers.len(), 4);

        for barrier in &barriers {
            tracing::info!(target: logger::SYNC, "Generate barrier: {:#?}", barrier);
        }

        assert_eq!(barriers[0].attachment, "draw");
        assert_eq!(barriers[0].src_layout, ImageLayout::Undefined);
        assert_eq!(barriers[0].dst_layout, ImageLayout::General);
        assert_eq!(
            barriers[0].src_scope,
            SyncScope {
                stage: BarrierStage::TopOfPipe,
                access: BarrierAccess::empty()
            }
        );
        assert_eq!(
            barriers[0].dst_scope,
            SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite
            }
        );

        assert_eq!(barriers[1].attachment, "draw2");
        assert_eq!(barriers[1].src_layout, ImageLayout::Undefined);
        assert_eq!(barriers[1].dst_layout, ImageLayout::General);
        assert_eq!(
            barriers[1].src_scope,
            SyncScope {
                stage: BarrierStage::TopOfPipe,
                access: BarrierAccess::empty()
            }
        );
        assert_eq!(
            barriers[1].dst_scope,
            SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite
            }
        );

        assert_ne!(barriers[2].attachment, barriers[3].attachment);
        assert_eq!(barriers[2].src_layout, ImageLayout::General);
        assert_eq!(barriers[2].dst_layout, ImageLayout::General);
        assert_eq!(
            barriers[2].src_scope,
            SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite
            }
        );
        assert_eq!(
            barriers[2].dst_scope,
            SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageRead
            }
        );
        assert_eq!(barriers[3].src_layout, ImageLayout::General);
        assert_eq!(barriers[3].dst_layout, ImageLayout::General);
        assert_eq!(
            barriers[3].src_scope,
            SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite
            }
        );
        assert_eq!(
            barriers[3].dst_scope,
            SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageRead
            }
        );
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_barrier_compute_war() {
        setup();

        let graph = GraphConfig::load_graph_with_data(
            GRAPH,
            "graph_compute_war",
            ImageExtent2D::default(),
            |_, _| {},
        )
        .unwrap();

        let barriers = graph.build_barriers();

        assert_eq!(barriers.len(), 2);

        for barrier in &barriers {
            tracing::info!(target: logger::SYNC, "Generate barrier: {:#?}", barrier);
            assert_eq!(barrier.attachment, "draw");
        }

        assert_eq!(barriers[0].src_layout, ImageLayout::Undefined);
        assert_eq!(barriers[0].dst_layout, ImageLayout::General);
        assert_eq!(
            barriers[0].src_scope,
            SyncScope {
                stage: BarrierStage::TopOfPipe,
                access: BarrierAccess::empty()
            }
        );
        assert_eq!(
            barriers[0].dst_scope,
            SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageRead
            }
        );

        assert_eq!(barriers[1].src_layout, ImageLayout::General);
        assert_eq!(barriers[1].dst_layout, ImageLayout::General);
        assert_eq!(
            barriers[1].src_scope,
            SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::empty()
            }
        );
        assert_eq!(
            barriers[1].dst_scope,
            SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::empty()
            }
        );
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_barrier_compute_color() {
        setup();

        let graph = GraphConfig::load_graph_with_data(
            GRAPH,
            "graph_compute_color",
            ImageExtent2D::default(),
            |_, _| {},
        )
        .unwrap();

        let barriers = graph.build_barriers();

        assert_eq!(barriers.len(), 2);

        for barrier in &barriers {
            tracing::info!(target: logger::SYNC, "Generate barrier: {:#?}", barrier);
            assert_eq!(barrier.attachment, "draw");
        }

        assert_eq!(barriers[0].src_layout, ImageLayout::Undefined);
        assert_eq!(barriers[0].dst_layout, ImageLayout::General);
        assert_eq!(
            barriers[0].src_scope,
            SyncScope {
                stage: BarrierStage::TopOfPipe,
                access: BarrierAccess::empty()
            }
        );
        assert_eq!(
            barriers[0].dst_scope,
            SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite,
            }
        );

        assert_eq!(barriers[1].src_layout, ImageLayout::General);
        assert_eq!(barriers[1].dst_layout, ImageLayout::Color);
        assert_eq!(
            barriers[1].src_scope,
            SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite,
            }
        );
        assert_eq!(
            barriers[1].dst_scope,
            SyncScope {
                stage: BarrierStage::ColorAttachmentOutput,
                access: BarrierAccess::ColorAttachmentWrite,
            }
        );
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_barrier_compute_color_blend() {
        setup();

        let graph = GraphConfig::load_graph_with_data(
            GRAPH,
            "graph_compute_color_blend",
            ImageExtent2D::default(),
            |_, _| {},
        )
        .unwrap();

        let barriers = graph.build_barriers();

        assert_eq!(barriers.len(), 2);

        for barrier in &barriers {
            tracing::info!(target: logger::SYNC, "Generate barrier: {:#?}", barrier);
            assert_eq!(barrier.attachment, "draw");
        }

        assert_eq!(barriers[0].src_layout, ImageLayout::Undefined);
        assert_eq!(barriers[0].dst_layout, ImageLayout::General);
        assert_eq!(
            barriers[0].src_scope,
            SyncScope {
                stage: BarrierStage::TopOfPipe,
                access: BarrierAccess::empty()
            }
        );
        assert_eq!(
            barriers[0].dst_scope,
            SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite,
            }
        );

        assert_eq!(barriers[1].src_layout, ImageLayout::General);
        assert_eq!(barriers[1].dst_layout, ImageLayout::Color);
        assert_eq!(
            barriers[1].src_scope,
            SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite,
            }
        );
        assert_eq!(
            barriers[1].dst_scope,
            SyncScope {
                stage: BarrierStage::ColorAttachmentOutput,
                access: BarrierAccess::ColorAttachmentRead | BarrierAccess::ColorAttachmentWrite,
            }
        );
    }
}
