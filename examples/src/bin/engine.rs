use std::sync::{Arc, Mutex};

use glam::{Quat, Vec3};
use gobs::{
    game::{
        app::{Application, RenderError, Run},
        input::{Input, Key},
    },
    gobs_core::{
        entity::uniform::{UniformData, UniformPropData},
        geometry::{
            mesh::Mesh,
            primitive::Primitive,
            vertex::{VertexData, VertexFlag},
            Transform,
        },
    },
    render::{context::Context, mesh::MeshBuffer, model::Model},
    scene::{
        graph::scenegraph::{Node, NodeValue},
        scene::Scene,
    },
    vulkan::{
        command::{CommandBuffer, CommandPool},
        descriptor::{
            DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutBuilder, DescriptorSetPool,
            DescriptorStage, DescriptorType,
        },
        device::Device,
        image::{ColorSpace, Image, ImageExtent2D, ImageFormat, ImageLayout, ImageUsage},
        pipeline::{Pipeline, PipelineLayout, Shader, ShaderType},
        queue::Queue,
        swapchain::{PresentationMode, SwapChain},
        sync::Semaphore,
        Wrap,
    },
};
use gpu_allocator::{vulkan::Allocator, vulkan::AllocatorCreateDesc, AllocatorDebugSettings};

const FRAMES_IN_FLIGHT: usize = 2;
const SHADER_DIR: &str = "examples/shaders";
const ASSET_DIR: &str = "examples/assets";

struct FrameData {
    pub command_buffer: CommandBuffer,
    pub swapchain_semaphore: Semaphore,
    pub render_semaphore: Semaphore,
}

impl FrameData {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        let command_pool = CommandPool::new(device.clone(), &queue.family);
        let command_buffer = CommandBuffer::new(device.clone(), queue.clone(), command_pool);

        let swapchain_semaphore = Semaphore::new(device.clone());
        let render_semaphore = Semaphore::new(device.clone());

        FrameData {
            command_buffer,
            swapchain_semaphore,
            render_semaphore,
        }
    }
}

#[allow(unused)]
struct App {
    frame_number: usize,
    frames: [FrameData; FRAMES_IN_FLIGHT],
    swapchain: SwapChain,
    swapchain_images: Vec<Image>,
    draw_image: Image,
    depth_image: Image,
    render_scaling: f32,
    ds_pool: DescriptorSetPool,
    draw_ds_layout: Arc<DescriptorSetLayout>,
    draw_ds: DescriptorSet,
    bg_pipeline: Pipeline,
    bg_pipeline_layout: Arc<PipelineLayout>,
    scene: Scene,
    allocator: Arc<Mutex<Allocator>>,
}

impl Run for App {
    async fn create(ctx: &Context) -> Self {
        log::info!("Create");

        let frames = [
            FrameData::new(ctx.device.clone(), ctx.queue.clone()),
            FrameData::new(ctx.device.clone(), ctx.queue.clone()),
        ];

        let swapchain = Self::create_swapchain(ctx);
        let swapchain_images = swapchain.create_images();

        let allocator = Arc::new(Mutex::new(
            Allocator::new(&AllocatorCreateDesc {
                instance: ctx.instance.cloned(),
                device: ctx.device.cloned(),
                physical_device: ctx.device.p_device.raw(),
                debug_settings: AllocatorDebugSettings {
                    log_memory_information: true,
                    log_leaks_on_shutdown: true,
                    store_stack_traces: false,
                    log_allocations: true,
                    log_frees: true,
                    log_stack_traces: false,
                },
                buffer_device_address: true,
                allocation_sizes: Default::default(),
            })
            .unwrap(),
        ));

        let extent = ctx.surface.get_extent(ctx.device.clone());
        let draw_image = Image::new(
            "color",
            ctx.device.clone(),
            ImageFormat::R16g16b16a16Sfloat,
            ImageUsage::Color,
            extent,
            allocator.clone(),
        );

        let depth_image = Image::new(
            "depth",
            ctx.device.clone(),
            ImageFormat::D32Sfloat,
            ImageUsage::Depth,
            extent,
            allocator.clone(),
        );

        let draw_ds_layout = DescriptorSetLayoutBuilder::new()
            .binding(DescriptorType::StorageImage, DescriptorStage::Compute)
            .build(ctx.device.clone());
        let ds_pool = DescriptorSetPool::new(ctx.device.clone(), draw_ds_layout.clone(), 10);
        let draw_ds = ds_pool.allocate(draw_ds_layout.clone());

        draw_ds
            .update()
            .bind_image(&draw_image, ImageLayout::General)
            .end();

        let compute_shader = Shader::from_file(
            &format!("{}/sky.comp.spv", SHADER_DIR),
            ctx.device.clone(),
            ShaderType::Compute,
        );

        let bg_pipeline_layout =
            PipelineLayout::new(ctx.device.clone(), Some(draw_ds_layout.clone()));
        let bg_pipeline = Pipeline::compute_builder(ctx.device.clone())
            .layout(bg_pipeline_layout.clone())
            .compute_shader("main", compute_shader)
            .build();

        let scene = Scene::new(
            ctx,
            draw_image.extent,
            draw_image.format,
            Some(depth_image.format),
        );

        App {
            frame_number: 0,
            frames,
            swapchain,
            swapchain_images,
            draw_image,
            depth_image,
            render_scaling: 1.,
            ds_pool,
            draw_ds_layout,
            draw_ds,
            bg_pipeline,
            bg_pipeline_layout,
            scene,
            allocator,
        }
    }

    fn start(&mut self, ctx: &Context) {
        log::trace!("Start");

        let meshes = gobs::scene::import::gltf::load_gltf(&format!("{}/basicmesh.glb", ASSET_DIR));
        let mesh = meshes[2].clone();

        let vertex_flags =
            VertexFlag::POSITION | VertexFlag::COLOR | VertexFlag::TEXTURE | VertexFlag::NORMAL;

        let mesh_buffer = MeshBuffer::new(ctx, mesh.clone(), vertex_flags, self.allocator.clone());

        let mut model = Model::new(mesh_buffer);
        let mut start = 0;
        for p in &mesh.primitives {
            model.add_surface(start, p.indices.len());
            start += p.indices.len();
        }

        let view = Transform::new(
            [0., 0., -3.].into(),
            Quat::from_rotation_y((15. as f32).to_radians()),
            Vec3::new(1., -1., 1.),
        );

        let node = Node::new(NodeValue::Model(model), view);

        self.scene.graph.insert(self.scene.graph.root, node);
    }

    fn update(&mut self, _ctx: &Context, _delta: f32) {
        log::trace!("Update");
    }

    fn render(&mut self, ctx: &Context) -> Result<(), RenderError> {
        log::trace!("Render frame {}", self.frame_number);

        debug_assert_eq!(self.draw_image.extent, self.depth_image.extent);

        let draw_extent = ImageExtent2D::new(
            (self
                .draw_image
                .extent
                .width
                .min(ctx.surface.get_dimensions().width) as f32
                * self.render_scaling) as u32,
            (self
                .draw_image
                .extent
                .height
                .min(ctx.surface.get_dimensions().height) as f32
                * self.render_scaling) as u32,
        );

        let frame = &self.frames[self.frame_number % FRAMES_IN_FLIGHT];

        frame.command_buffer.fence.wait_and_reset();
        debug_assert!(!frame.command_buffer.fence.signaled());

        let Ok(image_index) = self.swapchain.acquire_image(&frame.swapchain_semaphore) else {
            return Err(RenderError::Outdated);
        };

        self.draw_image.invalidate();
        self.depth_image.invalidate();
        self.swapchain_images[image_index as usize].invalidate();

        frame.command_buffer.reset();

        frame.command_buffer.begin();

        frame
            .command_buffer
            .begin_label(&format!("Frame {}", self.frame_number));

        frame
            .command_buffer
            .transition_image_layout(&mut self.draw_image, ImageLayout::General);

        frame.command_buffer.begin_label("Draw background");
        self.draw_background(&frame.command_buffer, draw_extent);
        frame.command_buffer.end_label();

        frame
            .command_buffer
            .transition_image_layout(&mut self.draw_image, ImageLayout::Color);

        frame
            .command_buffer
            .transition_image_layout(&mut self.depth_image, ImageLayout::Depth);

        frame.command_buffer.begin_label("Draw scene");
        self.draw_scene(ctx, &frame.command_buffer, draw_extent);
        frame.command_buffer.end_label();

        frame
            .command_buffer
            .transition_image_layout(&mut self.draw_image, ImageLayout::TransferSrc);

        let swapchain_image = &mut self.swapchain_images[image_index as usize];

        frame
            .command_buffer
            .transition_image_layout(swapchain_image, ImageLayout::TransferDst);

        frame.command_buffer.copy_image_to_image(
            &self.draw_image,
            draw_extent,
            swapchain_image,
            swapchain_image.extent,
        );

        frame
            .command_buffer
            .transition_image_layout(swapchain_image, ImageLayout::Present);

        frame.command_buffer.end_label();

        frame.command_buffer.end();

        frame.command_buffer.submit2(
            Some(&frame.swapchain_semaphore),
            Some(&frame.render_semaphore),
        );

        let Ok(_) = self
            .swapchain
            .present(image_index, &ctx.queue, &frame.render_semaphore)
        else {
            return Err(RenderError::Outdated);
        };

        self.frame_number += 1;

        log::trace!("End render");

        Ok(())
    }

    fn input(&mut self, _ctx: &Context, input: Input) {
        log::trace!("Input");

        match input {
            Input::KeyPressed(key) => match key {
                Key::E => self.render_scaling = (self.render_scaling + 0.1).min(1.),
                Key::A => self.render_scaling = (self.render_scaling - 0.1).max(0.1),
                _ => (),
            },
            _ => (),
        }
    }

    fn resize(&mut self, ctx: &Context, _width: u32, _height: u32) {
        log::trace!("Resize");
        self.resize_swapchain(ctx);
    }

    fn close(&mut self, ctx: &Context) {
        log::info!("Closing");

        ctx.device.wait();

        log::info!("Closed");
    }
}

impl App {
    #[allow(unused)]
    fn clear_background(&self, cmd: &CommandBuffer) {
        let flash = (self.frame_number as f32 / 120.).sin().abs();
        cmd.clear_color(&self.draw_image, [flash, 0., 0., 1.]);
    }

    fn draw_background(&self, cmd: &CommandBuffer, draw_extent: ImageExtent2D) {
        cmd.bind_pipeline(&self.bg_pipeline);
        cmd.bind_descriptor_set(&self.draw_ds, &self.bg_pipeline);
        cmd.dispatch(draw_extent.width / 16 + 1, draw_extent.height / 16 + 1, 1);
    }

    fn draw_scene(&self, ctx: &Context, cmd: &CommandBuffer, draw_extent: ImageExtent2D) {
        cmd.begin_rendering(
            &self.draw_image,
            draw_extent,
            Some(&self.depth_image),
            false,
            [0.; 4],
            1.,
        );
        cmd.bind_pipeline(&self.scene.pipeline);
        cmd.set_viewport(draw_extent.width, draw_extent.height);

        self.scene
            .graph
            .visit(self.scene.graph.root, &|transform, model| {
                if let NodeValue::Model(model) = model {
                    let world_matrix = self.scene.camera.view_proj() * transform.matrix;

                    let scene_data = UniformData::builder("scene data")
                        .prop(
                            "world_matrix",
                            UniformPropData::Mat4F(world_matrix.to_cols_array_2d()),
                        )
                        .prop(
                            "vertex_buffer",
                            UniformPropData::U64(
                                model.buffers.vertex_buffer.address(ctx.device.clone()),
                            ),
                        )
                        .build();
                    cmd.push_constants(self.scene.pipeline_layout.clone(), &scene_data.raw());

                    for surface in &model.surfaces {
                        cmd.bind_index_buffer::<u32>(&model.buffers.index_buffer, surface.offset);
                        cmd.draw_indexed(surface.len, 1);
                    }
                }
            });

        cmd.end_rendering();
    }

    fn _load_mesh() -> Arc<Mesh> {
        let v1 = VertexData::builder()
            .padding(true)
            .position([0.5, -0.5, 0.].into())
            .normal([0., 0., 1.].into())
            .texture([1., 0.].into())
            .color([1., 0., 0., 1.].into())
            .build();

        let v2 = VertexData::builder()
            .padding(true)
            .position([0.5, 0.5, 0.].into())
            .normal([0., 0., 1.].into())
            .texture([1., 1.].into())
            .color([0.5, 0.5, 0.5, 1.].into())
            .build();

        let v3 = VertexData::builder()
            .padding(true)
            .position([-0.5, -0.5, 0.].into())
            .normal([0., 0., 1.].into())
            .texture([0., 0.].into())
            .color([0., 0., 1., 1.].into())
            .build();

        let v4 = VertexData::builder()
            .padding(true)
            .position([-0.5, 0.5, 0.].into())
            .normal([0., 0., 1.].into())
            .texture([0., 1.].into())
            .color([0., 1., 0., 1.].into())
            .build();

        let vertices = vec![v1, v2, v3, v4];
        let indices = vec![0, 1, 2, 2, 1, 3];

        let vertex_flags =
            VertexFlag::POSITION | VertexFlag::COLOR | VertexFlag::TEXTURE | VertexFlag::NORMAL;

        log::info!("Vertex size: {}", VertexData::size(vertex_flags, true));

        let p = Primitive::builder()
            .add_indices(&indices)
            .add_vertices(&vertices)
            .build();

        Mesh::builder("quad").add_primitive(p).build()
    }

    fn create_swapchain(ctx: &Context) -> SwapChain {
        let device = ctx.device.clone();
        let surface = ctx.surface.clone();

        let presents = surface.get_available_presentation_modes(device.clone());

        let present = *presents
            .iter()
            .find(|&&p| p == PresentationMode::Fifo)
            .unwrap();

        let caps = surface.get_capabilities(device.clone());

        let mut image_count = caps.min_image_count + 1;
        if caps.max_image_count > 0 && image_count > caps.max_image_count {
            image_count = caps.max_image_count;
        }

        let formats = surface.get_available_format(&device.p_device);

        let format = *formats
            .iter()
            .find(|f| {
                f.format == ImageFormat::B8g8r8a8Unorm && f.color_space == ColorSpace::SrgbNonlinear
            })
            .unwrap();

        log::info!("Swapchain format: {:?}", format);

        SwapChain::new(device, surface, format, present, image_count, None)
    }

    fn resize_swapchain(&mut self, ctx: &Context) {
        ctx.device.wait();

        self.swapchain = SwapChain::new(
            ctx.device.clone(),
            ctx.surface.clone(),
            self.swapchain.format,
            self.swapchain.present,
            self.swapchain.image_count,
            Some(&self.swapchain),
        );
        self.swapchain_images = self.swapchain.create_images();
    }
}

fn main() {
    examples::init_logger();

    log::info!("Engine start");

    Application::new("examples", 1600, 900).run::<App>();
}
