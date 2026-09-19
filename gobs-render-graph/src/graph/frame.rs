use std::collections::{HashMap, hash_map::Entry};

use crate::{
    FrameData, GraphConfig, PassId, PassMetaData, RenderError, RenderPassType,
    graph::{resource::GraphResourceManager, sync::SyncStatus},
    pass::{Attachment, AttachmentAccess, AttachmentType},
};
use gobs_core::{ImageExtent2D, logger};
use gobs_render_hal::{
    Barrier, BarrierAccess, BarrierStage, BarrierSyncScope, BarrierType, GfxContext, ImageLayout,
};

#[derive(Clone)]
pub struct FrameGraphPass {
    pub pass: PassMetaData,
    pub enabled: bool,
}

pub struct FrameGraph {
    pub render_scaling: f32,
    pub passes: Vec<FrameGraphPass>,
    pub attachments: HashMap<String, Attachment>,
    pub resource_manager: GraphResourceManager,
    pub barriers: HashMap<PassId, Vec<Barrier>>,
}

impl FrameGraph {
    pub fn new() -> Self {
        Self {
            render_scaling: 1.,
            passes: Vec::new(),
            attachments: HashMap::new(),
            resource_manager: GraphResourceManager::new(),
            barriers: HashMap::new(),
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

    pub fn add_barrier(&mut self, pass_id: PassId, barrier: Barrier) {
        match self.barriers.entry(pass_id) {
            Entry::Occupied(mut e) => e.get_mut().push(barrier),
            Entry::Vacant(e) => {
                e.insert(vec![barrier]);
            }
        }
    }

    fn barrier_scope(ty: AttachmentType, access: AttachmentAccess) -> BarrierSyncScope {
        match (ty, access) {
            (AttachmentType::Color, AttachmentAccess::Read) => BarrierSyncScope {
                stage: BarrierStage::ColorAttachmentOutput,
                access: BarrierAccess::ColorAttachmentRead,
            },
            (AttachmentType::Color, AttachmentAccess::Write) => BarrierSyncScope {
                stage: BarrierStage::ColorAttachmentOutput,
                access: BarrierAccess::ColorAttachmentWrite,
            },
            (AttachmentType::Color, AttachmentAccess::ReadWrite) => BarrierSyncScope {
                stage: BarrierStage::ColorAttachmentOutput,
                access: BarrierAccess::ColorAttachmentRead | BarrierAccess::ColorAttachmentWrite,
            },
            (AttachmentType::Depth, AttachmentAccess::Read) => BarrierSyncScope {
                stage: BarrierStage::FragmentTests,
                access: BarrierAccess::DepthStencilAttachmentRead,
            },
            (AttachmentType::Depth, AttachmentAccess::Write) => BarrierSyncScope {
                stage: BarrierStage::FragmentTests,
                access: BarrierAccess::DepthStencilAttachmentWrite,
            },
            (AttachmentType::Depth, AttachmentAccess::ReadWrite) => BarrierSyncScope {
                stage: BarrierStage::FragmentTests,
                access: BarrierAccess::DepthStencilAttachmentRead
                    | BarrierAccess::DepthStencilAttachmentWrite,
            },
            (AttachmentType::ImageStorage, AttachmentAccess::Read) => BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageRead,
            },
            (AttachmentType::ImageStorage, AttachmentAccess::Write) => BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite,
            },
            (AttachmentType::ImageStorage, AttachmentAccess::ReadWrite) => BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageRead | BarrierAccess::ShaderStorageWrite,
            },
        }
    }

    pub fn build_barriers(&mut self) {
        let mut attachments_status: HashMap<String, SyncStatus> = HashMap::new();

        for pass in self.passes.clone() {
            if !pass.enabled {
                continue;
            }

            tracing::info!(target: logger::SYNC, "Generate barriers for pass {} [{}]", pass.pass.name(), pass.pass.id);

            for (attachment_name, attachment) in &pass.pass.attachments {
                let pass_id = pass.pass.id;
                let ty = attachment.ty;
                let access = attachment.access;
                let layout = attachment.layout;
                let scope = Self::barrier_scope(ty, access);

                if let Some(status) = attachments_status.get_mut(attachment_name) {
                    if status.last_layout() != layout {
                        // image layout transition barrier
                        let barrier = Barrier::new(attachment_name)
                            .image(attachment_name)
                            .layouts(status.last_layout(), layout)
                            .memory(status.last_write(), scope);

                        self.add_barrier(pass_id, barrier);

                        status.update(scope, layout);
                        status.clear_invalidates();
                        status.invalidate(scope);
                    } else {
                        match access {
                            AttachmentAccess::Read => {
                                if !status.is_flushed() {
                                    // RAW -> flush + invalidate
                                    let barrier = Barrier::new(attachment_name)
                                        .image(attachment_name)
                                        .layouts(status.last_layout(), layout)
                                        .memory(status.last_write(), scope);

                                    self.add_barrier(pass_id, barrier);
                                    status.invalidate(scope);
                                    status.flush();
                                } else if !status.is_invalidated(scope) {
                                    // RAW, already flushed ->  invalidate
                                    let barrier = Barrier::new(attachment_name)
                                        .image(attachment_name)
                                        .layouts(status.last_layout(), layout)
                                        .invalidation(status.last_write(), scope);

                                    self.add_barrier(pass_id, barrier);
                                    status.invalidate(scope);
                                } else {
                                    // RAR, invalidated -> no barrier
                                }
                            }
                            AttachmentAccess::Write => {
                                if !status.is_flushed() {
                                    // WAW -> flush
                                    let barrier = Barrier::new(attachment_name)
                                        .image(attachment_name)
                                        .layouts(status.last_layout(), layout)
                                        .flush(status.last_write(), scope);

                                    self.add_barrier(pass_id, barrier);
                                    status.flush();
                                } else {
                                    // WAR -> execution barrier only
                                    let barrier = Barrier::new(attachment_name)
                                        .image(attachment_name)
                                        .layouts(status.last_layout(), layout)
                                        .execution(status.last_write(), scope);

                                    self.add_barrier(pass_id, barrier);
                                }

                                status.update(scope, layout);
                                status.clear_invalidates();
                            }
                            AttachmentAccess::ReadWrite => {
                                if !status.is_flushed() {
                                    // WAW -> flush + invalidate
                                    let barrier = Barrier::new(attachment_name)
                                        .image(attachment_name)
                                        .layouts(status.last_layout(), layout)
                                        .memory(status.last_write(), scope);

                                    self.add_barrier(pass_id, barrier);
                                    status.flush();
                                } else if !status.is_invalidated(scope) {
                                    // WAR -> invalidate
                                    let barrier = Barrier::new(attachment_name)
                                        .image(attachment_name)
                                        .layouts(status.last_layout(), layout)
                                        .invalidation(status.last_write(), scope);

                                    self.add_barrier(pass_id, barrier);
                                } else {
                                    // WAR -> execution barrier only
                                    let barrier = Barrier::new(attachment_name)
                                        .image(attachment_name)
                                        .layouts(status.last_layout(), layout)
                                        .execution(status.last_write(), scope);

                                    self.add_barrier(pass_id, barrier);
                                }

                                status.update(scope, layout);
                                status.clear_invalidates();
                            }
                        }
                    }
                } else {
                    // first resource usage: do a transition from UNDEFINED
                    let barrier = Barrier::new(attachment_name)
                        .image(attachment_name)
                        .layouts(ImageLayout::Undefined, layout)
                        .memory(
                            BarrierSyncScope {
                                stage: BarrierStage::TopOfPipe,
                                access: BarrierAccess::empty(),
                            },
                            scope,
                        );

                    let status = SyncStatus::new(scope, layout);
                    attachments_status.insert(attachment_name.to_string(), status);

                    self.add_barrier(pass_id, barrier);
                }
            }
        }
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
        barriers: &HashMap<PassId, Vec<Barrier>>,
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

        if let Some(barriers) = barriers.get(&pass.id) {
            for barrier in barriers {
                if let BarrierType::Image(label) = &barrier.ty {
                    tracing::debug!(target: logger::SYNC, "Insert image barrier image={}, pass={}", label, &pass.name);
                    let handle = resource_manager.image(label);

                    frame
                        .command
                        .as_mut()
                        .set_image_barrier(ctx, barrier, handle);
                }
            }
        }

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
            Self::run_pass(
                ctx,
                frame,
                pass,
                &self.resource_manager,
                &self.barriers,
                &mut run_pass_cb,
            )?;
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
    use gobs_render_hal::{BarrierAccess, BarrierStage, BarrierSyncScope, ImageLayout};
    use tracing::Level;
    use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

    use gobs_core::{ImageExtent2D, logger};

    use crate::GraphConfig;

    const GRAPH: &str = r#"
        GraphConfig(
            graphes: {
                "graph_compute_rw": ["compute_reader_writer1"],
                "graph_compute_raw": ["compute_writer1", "compute_reader1"],
                "graph_compute_raww": ["compute_writer1", "compute_writer2", "compute_reader12"],
                "graph_compute_war": ["compute_reader1", "compute_writer1"],
                "graph_compute_waw": ["compute_writer1", "compute_writer1b"],
                "graph_compute_rar": ["compute_reader1", "compute_reader1b"],
                "graph_compute_color": ["compute_writer1", "color_writer1"],
                "graph_compute_color_read": ["compute_writer1", "color_reader1"],
                "graph_compute_color_blend": ["compute_writer1", "color_blend_writer1"],
            },
            passes: {
                "compute_writer1": (ty: Compute, config: "test_c", attachments: { "draw": StorageImage(access: Write) }),
                "compute_writer1b": (ty: Compute, config: "test_c", attachments: { "draw": StorageImage(access: Write) }),
                "compute_writer2": (ty: Compute, config: "test_c", attachments: { "draw2": StorageImage(access: Write) }),
                "compute_reader1": (ty: Compute, config: "test_c", attachments: { "draw": StorageImage(access: Read) }),
                "compute_reader1b": (ty: Compute, config: "test_c", attachments: { "draw": StorageImage(access: Read) }),
                "compute_reader12": (ty: Compute, config: "test_c", attachments: { "draw": StorageImage(access: Read), "draw2": StorageImage(access: Read) }),
                "compute_reader_writer1": (ty: Compute, config: "test_c", attachments: { "draw": StorageImage(access: ReadWrite) }),
                "color_writer1": (ty: Material, config: "test", attachments: { "draw": ColorAttachment(access: Write, clear: false) }),
                "color_reader1": (ty: Material, config: "test", attachments: { "draw": ColorAttachment(access: Read, clear: false) }),
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
    fn test_barrier_compute_rw() {
        setup();

        let mut graph = GraphConfig::load_graph_with_data(
            GRAPH,
            "graph_compute_rw",
            ImageExtent2D::default(),
            |_, _| {},
        )
        .unwrap();

        graph.build_barriers();

        assert_eq!(graph.barriers.len(), 1);

        for barrier in graph.barriers.values() {
            tracing::info!(target: logger::SYNC, "Generate barrier: {:#?}", barrier);
            assert_eq!(barrier.len(), 1);
            assert_eq!(barrier[0].label, "draw");
        }

        let barrier = &graph.barriers[&graph.passes[0].pass.id][0];
        assert_eq!(barrier.src_layout, ImageLayout::Undefined);
        assert_eq!(barrier.dst_layout, ImageLayout::General);
        assert_eq!(
            barrier.src_scope,
            BarrierSyncScope {
                stage: BarrierStage::TopOfPipe,
                access: BarrierAccess::empty()
            }
        );
        assert_eq!(
            barrier.dst_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageRead | BarrierAccess::ShaderStorageWrite
            }
        );
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_barrier_compute_raw() {
        setup();

        let mut graph = GraphConfig::load_graph_with_data(
            GRAPH,
            "graph_compute_raw",
            ImageExtent2D::default(),
            |_, _| {},
        )
        .unwrap();

        graph.build_barriers();

        assert_eq!(graph.barriers.len(), 2);

        for barrier in graph.barriers.values() {
            tracing::info!(target: logger::SYNC, "Generate barrier: {:#?}", barrier);
            assert_eq!(barrier.len(), 1);
            assert_eq!(barrier[0].label, "draw");
        }

        let barrier = &graph.barriers[&graph.passes[0].pass.id][0];
        assert_eq!(barrier.src_layout, ImageLayout::Undefined);
        assert_eq!(barrier.dst_layout, ImageLayout::General);
        assert_eq!(
            barrier.src_scope,
            BarrierSyncScope {
                stage: BarrierStage::TopOfPipe,
                access: BarrierAccess::empty()
            }
        );
        assert_eq!(
            barrier.dst_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite
            }
        );

        let barrier = &graph.barriers[&graph.passes[1].pass.id][0];
        assert_eq!(barrier.src_layout, ImageLayout::General);
        assert_eq!(barrier.dst_layout, ImageLayout::General);
        assert_eq!(
            barrier.src_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite
            }
        );
        assert_eq!(
            barrier.dst_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageRead
            }
        );
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_barrier_compute_raww() {
        setup();

        let mut graph = GraphConfig::load_graph_with_data(
            GRAPH,
            "graph_compute_raww",
            ImageExtent2D::default(),
            |_, _| {},
        )
        .unwrap();

        graph.build_barriers();

        for barrier in graph.barriers.values() {
            tracing::info!(target: logger::SYNC, "Generate barrier: {:#?}", barrier);
        }

        assert_eq!(graph.barriers.len(), 3);

        let barrier = &graph.barriers[&graph.passes[0].pass.id][0];
        assert_eq!(barrier.label, "draw");
        assert_eq!(barrier.src_layout, ImageLayout::Undefined);
        assert_eq!(barrier.dst_layout, ImageLayout::General);
        assert_eq!(
            barrier.src_scope,
            BarrierSyncScope {
                stage: BarrierStage::TopOfPipe,
                access: BarrierAccess::empty()
            }
        );
        assert_eq!(
            barrier.dst_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite
            }
        );

        let barrier = &graph.barriers[&graph.passes[1].pass.id][0];
        assert_eq!(barrier.label, "draw2");
        assert_eq!(barrier.src_layout, ImageLayout::Undefined);
        assert_eq!(barrier.dst_layout, ImageLayout::General);
        assert_eq!(
            barrier.src_scope,
            BarrierSyncScope {
                stage: BarrierStage::TopOfPipe,
                access: BarrierAccess::empty()
            }
        );
        assert_eq!(
            barrier.dst_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite
            }
        );

        assert_ne!(
            graph.barriers[&graph.passes[2].pass.id][0].label,
            graph.barriers[&graph.passes[2].pass.id][1].label
        );
        let barrier = &graph.barriers[&graph.passes[2].pass.id][0];
        assert_eq!(barrier.src_layout, ImageLayout::General);
        assert_eq!(barrier.dst_layout, ImageLayout::General);
        assert_eq!(
            barrier.src_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite
            }
        );
        assert_eq!(
            barrier.dst_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageRead
            }
        );
        let barrier = &graph.barriers[&graph.passes[2].pass.id][1];
        assert_eq!(barrier.src_layout, ImageLayout::General);
        assert_eq!(barrier.dst_layout, ImageLayout::General);
        assert_eq!(
            barrier.src_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite
            }
        );
        assert_eq!(
            barrier.dst_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageRead
            }
        );
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_barrier_compute_war() {
        setup();

        let mut graph = GraphConfig::load_graph_with_data(
            GRAPH,
            "graph_compute_war",
            ImageExtent2D::default(),
            |_, _| {},
        )
        .unwrap();

        graph.build_barriers();

        assert_eq!(graph.barriers.len(), 2);

        for barrier in graph.barriers.values() {
            tracing::info!(target: logger::SYNC, "Generate barrier: {:#?}", barrier);
            assert_eq!(barrier.len(), 1);
            assert_eq!(barrier[0].label, "draw");
        }

        let barrier = &graph.barriers[&graph.passes[0].pass.id][0];
        assert_eq!(barrier.src_layout, ImageLayout::Undefined);
        assert_eq!(barrier.dst_layout, ImageLayout::General);
        assert_eq!(
            barrier.src_scope,
            BarrierSyncScope {
                stage: BarrierStage::TopOfPipe,
                access: BarrierAccess::empty()
            }
        );
        assert_eq!(
            barrier.dst_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageRead
            }
        );

        let barrier = &graph.barriers[&graph.passes[1].pass.id][0];
        assert_eq!(barrier.src_layout, ImageLayout::General);
        assert_eq!(barrier.dst_layout, ImageLayout::General);
        assert_eq!(
            barrier.src_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::empty()
            }
        );
        assert_eq!(
            barrier.dst_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::empty()
            }
        );
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_barrier_compute_waw() {
        setup();

        let mut graph = GraphConfig::load_graph_with_data(
            GRAPH,
            "graph_compute_waw",
            ImageExtent2D::default(),
            |_, _| {},
        )
        .unwrap();

        graph.build_barriers();

        assert_eq!(graph.barriers.len(), 2);

        for barrier in graph.barriers.values() {
            tracing::info!(target: logger::SYNC, "Generate barrier: {:#?}", barrier);
            assert_eq!(barrier.len(), 1);
            assert_eq!(barrier[0].label, "draw");
        }

        let barrier = &graph.barriers[&graph.passes[0].pass.id][0];
        assert_eq!(barrier.src_layout, ImageLayout::Undefined);
        assert_eq!(barrier.dst_layout, ImageLayout::General);
        assert_eq!(
            barrier.src_scope,
            BarrierSyncScope {
                stage: BarrierStage::TopOfPipe,
                access: BarrierAccess::empty()
            }
        );
        assert_eq!(
            barrier.dst_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite
            }
        );

        let barrier = &graph.barriers[&graph.passes[1].pass.id][0];
        assert_eq!(barrier.src_layout, ImageLayout::General);
        assert_eq!(barrier.dst_layout, ImageLayout::General);
        assert_eq!(
            barrier.src_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite
            }
        );
        assert_eq!(
            barrier.dst_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::empty()
            }
        );
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_barrier_compute_rar() {
        setup();

        let mut graph = GraphConfig::load_graph_with_data(
            GRAPH,
            "graph_compute_rar",
            ImageExtent2D::default(),
            |_, _| {},
        )
        .unwrap();

        graph.build_barriers();

        assert_eq!(graph.barriers.len(), 1);

        for barrier in graph.barriers.values() {
            tracing::info!(target: logger::SYNC, "Generate barrier: {:#?}", barrier);
            assert_eq!(barrier.len(), 1);
            assert_eq!(barrier[0].label, "draw");
        }

        let barrier = &graph.barriers[&graph.passes[0].pass.id][0];
        assert_eq!(barrier.src_layout, ImageLayout::Undefined);
        assert_eq!(barrier.dst_layout, ImageLayout::General);
        assert_eq!(
            barrier.src_scope,
            BarrierSyncScope {
                stage: BarrierStage::TopOfPipe,
                access: BarrierAccess::empty()
            }
        );
        assert_eq!(
            barrier.dst_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageRead
            }
        );
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_barrier_compute_color_read() {
        setup();

        let mut graph = GraphConfig::load_graph_with_data(
            GRAPH,
            "graph_compute_color_read",
            ImageExtent2D::default(),
            |_, _| {},
        )
        .unwrap();

        graph.build_barriers();

        assert_eq!(graph.barriers.len(), 2);

        for barrier in graph.barriers.values() {
            tracing::info!(target: logger::SYNC, "Generate barrier: {:#?}", barrier);
            assert_eq!(barrier.len(), 1);
            assert_eq!(barrier[0].label, "draw");
        }

        let barrier = &graph.barriers[&graph.passes[0].pass.id][0];
        assert_eq!(barrier.src_layout, ImageLayout::Undefined);
        assert_eq!(barrier.dst_layout, ImageLayout::General);
        assert_eq!(
            barrier.src_scope,
            BarrierSyncScope {
                stage: BarrierStage::TopOfPipe,
                access: BarrierAccess::empty()
            }
        );
        assert_eq!(
            barrier.dst_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite,
            }
        );

        let barrier = &graph.barriers[&graph.passes[1].pass.id][0];
        assert_eq!(barrier.src_layout, ImageLayout::General);
        assert_eq!(barrier.dst_layout, ImageLayout::Color);
        assert_eq!(
            barrier.src_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite,
            }
        );
        assert_eq!(
            barrier.dst_scope,
            BarrierSyncScope {
                stage: BarrierStage::ColorAttachmentOutput,
                access: BarrierAccess::ColorAttachmentRead,
            }
        );
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_barrier_compute_color() {
        setup();

        let mut graph = GraphConfig::load_graph_with_data(
            GRAPH,
            "graph_compute_color",
            ImageExtent2D::default(),
            |_, _| {},
        )
        .unwrap();

        graph.build_barriers();

        assert_eq!(graph.barriers.len(), 2);

        for barrier in graph.barriers.values() {
            tracing::info!(target: logger::SYNC, "Generate barrier: {:#?}", barrier);
            assert_eq!(barrier.len(), 1);
            assert_eq!(barrier[0].label, "draw");
        }

        let barrier = &graph.barriers[&graph.passes[0].pass.id][0];
        assert_eq!(barrier.src_layout, ImageLayout::Undefined);
        assert_eq!(barrier.dst_layout, ImageLayout::General);
        assert_eq!(
            barrier.src_scope,
            BarrierSyncScope {
                stage: BarrierStage::TopOfPipe,
                access: BarrierAccess::empty()
            }
        );
        assert_eq!(
            barrier.dst_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite,
            }
        );

        let barrier = &graph.barriers[&graph.passes[1].pass.id][0];
        assert_eq!(barrier.src_layout, ImageLayout::General);
        assert_eq!(barrier.dst_layout, ImageLayout::Color);
        assert_eq!(
            barrier.src_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite,
            }
        );
        assert_eq!(
            barrier.dst_scope,
            BarrierSyncScope {
                stage: BarrierStage::ColorAttachmentOutput,
                access: BarrierAccess::ColorAttachmentWrite,
            }
        );
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_barrier_compute_color_blend() {
        setup();

        let mut graph = GraphConfig::load_graph_with_data(
            GRAPH,
            "graph_compute_color_blend",
            ImageExtent2D::default(),
            |_, _| {},
        )
        .unwrap();

        graph.build_barriers();

        assert_eq!(graph.barriers.len(), 2);

        for barrier in graph.barriers.values() {
            tracing::info!(target: logger::SYNC, "Generate barrier: {:#?}", barrier);
            assert_eq!(barrier.len(), 1);
            assert_eq!(barrier[0].label, "draw");
        }

        let barrier = &graph.barriers[&graph.passes[0].pass.id][0];
        assert_eq!(barrier.src_layout, ImageLayout::Undefined);
        assert_eq!(barrier.dst_layout, ImageLayout::General);
        assert_eq!(
            barrier.src_scope,
            BarrierSyncScope {
                stage: BarrierStage::TopOfPipe,
                access: BarrierAccess::empty()
            }
        );
        assert_eq!(
            barrier.dst_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite,
            }
        );

        let barrier = &graph.barriers[&graph.passes[1].pass.id][0];
        assert_eq!(barrier.src_layout, ImageLayout::General);
        assert_eq!(barrier.dst_layout, ImageLayout::Color);
        assert_eq!(
            barrier.src_scope,
            BarrierSyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite,
            }
        );
        assert_eq!(
            barrier.dst_scope,
            BarrierSyncScope {
                stage: BarrierStage::ColorAttachmentOutput,
                access: BarrierAccess::ColorAttachmentRead | BarrierAccess::ColorAttachmentWrite,
            }
        );
    }
}
