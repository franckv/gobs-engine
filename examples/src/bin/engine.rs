use std::sync::Arc;

use glam::Mat4;
use gobs::{
    game::{
        app::{Application, RenderError, Run},
        context::Context,
        input::{Input, Key},
    },
    gobs_core::{
        entity::uniform::{UniformData, UniformProp},
        geometry::{
            mesh::Mesh,
            vertex::{VertexData, VertexFlag},
        },
    },
    vulkan::{
        buffer::{Buffer, BufferAddress, BufferUsage},
        command::{CommandBuffer, CommandPool},
        descriptor::{
            DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutBuilder, DescriptorSetPool,
            DescriptorStage, DescriptorType,
        },
        device::Device,
        image::{ColorSpace, Image, ImageExtent2D, ImageFormat, ImageLayout, ImageUsage},
        pipeline::{
            DynamicStateElem, FrontFace, Pipeline, PipelineLayout, Rect2D, Shader, ShaderType,
            Viewport,
        },
        queue::Queue,
        swapchain::{PresentationMode, SwapChain},
        sync::{Fence, Semaphore},
    },
};

const FRAMES_IN_FLIGHT: usize = 2;

struct FrameData {
    pub command_buffer: CommandBuffer,
    pub swapchain_semaphore: Semaphore,
    pub render_semaphore: Semaphore,
    pub render_fence: Fence,
}

impl FrameData {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        let command_pool = CommandPool::new(device.clone(), &queue.family);
        let command_buffer = CommandBuffer::new(device.clone(), queue.clone(), command_pool);

        let swapchain_semaphore = Semaphore::new(device.clone());
        let render_semaphore = Semaphore::new(device.clone());
        let render_fence = Fence::new(device, true);

        FrameData {
            command_buffer,
            swapchain_semaphore,
            render_semaphore,
            render_fence,
        }
    }
}

struct MeshResource {
    staging: Buffer,
    index_buffer: Buffer,
    vertex_buffer: Buffer,
    vertex_address: BufferAddress,
}

impl MeshResource {
    pub fn new(device: Arc<Device>, mesh: Arc<Mesh>) -> Self {
        let flags =
            VertexFlag::POSITION | VertexFlag::COLOR | VertexFlag::TEXTURE | VertexFlag::NORMAL;

        let vertices_data = mesh.vertices_data(flags);
        let vertices_size = vertices_data.len();

        let indices_size = mesh.indices.len() * std::mem::size_of::<u32>();

        let mut staging = Buffer::new(
            indices_size + vertices_size,
            BufferUsage::Staging,
            device.clone(),
        );

        let index_buffer = Buffer::new(indices_size, BufferUsage::Index, device.clone());
        let vertex_buffer = Buffer::new(vertices_size, BufferUsage::Vertex, device.clone());
        let vertex_address = vertex_buffer.address(device.clone());

        staging.copy(&vertices_data, 0);
        staging.copy(&mesh.indices, vertices_size);

        MeshResource {
            staging,
            index_buffer,
            vertex_buffer,
            vertex_address,
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
    render_scaling: f32,
    ds_pool: DescriptorSetPool,
    draw_ds_layout: Arc<DescriptorSetLayout>,
    draw_ds: DescriptorSet,
    bg_pipeline: Pipeline,
    bg_pipeline_layout: Arc<PipelineLayout>,
    scene_pipeline: Pipeline,
    scene_pipeline_layout: Arc<PipelineLayout>,
    scene_data: UniformData,
    mesh_resource: MeshResource,
    immediate_cmd: CommandBuffer,
    immediate_fence: Fence,
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

        let extent = ctx.surface.get_extent(ctx.device.clone());
        let draw_image = Image::new(
            ctx.device.clone(),
            ImageFormat::R16g16b16a16Sfloat,
            ImageUsage::Color,
            extent,
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
            "examples/shaders/sky.comp.spv",
            ctx.device.clone(),
            ShaderType::Compute,
        );

        let bg_pipeline_layout =
            PipelineLayout::new(ctx.device.clone(), Some(draw_ds_layout.clone()));
        let bg_pipeline = Pipeline::compute_builder(ctx.device.clone())
            .layout(bg_pipeline_layout.clone())
            .compute_shader("main", compute_shader)
            .build();

        let vertex_shader = Shader::from_file(
            "examples/shaders/mesh.vert.spv",
            ctx.device.clone(),
            ShaderType::Vertex,
        );

        let fragment_shader = Shader::from_file(
            "examples/shaders/triangle.frag.spv",
            ctx.device.clone(),
            ShaderType::Fragment,
        );

        let mesh = App::get_mesh();

        let mesh_resource = MeshResource::new(ctx.device.clone(), mesh);

        let scene_data = UniformData::builder("scene data")
            .prop(
                "world_matrix",
                UniformProp::Mat4F(Mat4::from_scale([0.5, 0.5, 1.].into()).to_cols_array_2d()),
            )
            .prop(
                "vertex_buffer",
                UniformProp::U64(mesh_resource.vertex_address),
            )
            .build();

        let scene_pipeline_layout =
            PipelineLayout::with_constants(ctx.device.clone(), None, scene_data.raw().len());
        let scene_pipeline = Pipeline::graphics_builder(ctx.device.clone())
            .layout(scene_pipeline_layout.clone())
            .vertex_shader("main", vertex_shader)
            .fragment_shader("main", fragment_shader)
            .viewports(vec![Viewport::new(
                0.,
                0.,
                draw_image.extent.width as f32,
                draw_image.extent.height as f32,
            )])
            .scissors(vec![Rect2D::new(
                0,
                0,
                draw_image.extent.width,
                draw_image.extent.height,
            )])
            .dynamic_states(&vec![DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .attachments(draw_image.format, None)
            .depth_test_disable()
            .front_face(FrontFace::CW)
            .build();

        let immediate_cmd_pool = CommandPool::new(ctx.device.clone(), &ctx.queue.family);
        let immediate_cmd =
            CommandBuffer::new(ctx.device.clone(), ctx.queue.clone(), immediate_cmd_pool);
        let immediate_fence = Fence::new(ctx.device.clone(), true);

        App {
            frame_number: 0,
            frames,
            swapchain,
            swapchain_images,
            draw_image,
            render_scaling: 1.,
            ds_pool,
            draw_ds_layout,
            draw_ds,
            bg_pipeline,
            bg_pipeline_layout,
            scene_pipeline,
            scene_pipeline_layout,
            scene_data,
            mesh_resource,
            immediate_cmd,
            immediate_fence,
        }
    }

    fn start(&mut self, _ctx: &Context) {
        self.immediate_submit(|cmd| {
            cmd.copy_buffer(
                &self.mesh_resource.staging,
                &self.mesh_resource.vertex_buffer,
                self.mesh_resource.vertex_buffer.size,
                0,
            );
            cmd.copy_buffer(
                &self.mesh_resource.staging,
                &self.mesh_resource.index_buffer,
                self.mesh_resource.index_buffer.size,
                self.mesh_resource.vertex_buffer.size,
            );
        });
    }

    fn update(&mut self, _ctx: &Context, _delta: f32) {
        log::debug!("Update");
    }

    fn render(&mut self, ctx: &Context) -> Result<(), RenderError> {
        log::debug!("Render frame {}", self.frame_number);

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

        frame.render_fence.wait_and_reset();
        assert!(!frame.render_fence.signaled());

        let Ok(image_index) = self.swapchain.acquire_image(&frame.swapchain_semaphore) else {
            return Err(RenderError::Outdated);
        };

        self.draw_image.invalidate();
        self.swapchain_images[image_index as usize].invalidate();

        frame.command_buffer.reset();

        frame.command_buffer.begin();

        frame
            .command_buffer
            .begin_label(&format!("Frame {}", self.frame_number));

        frame
            .command_buffer
            .transition_image_layout(&mut self.draw_image, ImageLayout::General);

        self.draw_background(&frame.command_buffer, draw_extent);

        frame
            .command_buffer
            .transition_image_layout(&mut self.draw_image, ImageLayout::Color);

        self.draw_scene(&frame.command_buffer, draw_extent);

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
            &frame.render_fence,
        );

        let Ok(_) = self
            .swapchain
            .present(image_index, &ctx.queue, &frame.render_semaphore)
        else {
            return Err(RenderError::Outdated);
        };

        self.frame_number += 1;

        log::debug!("End render");

        Ok(())
    }

    fn input(&mut self, _ctx: &Context, input: Input) {
        log::debug!("Input");

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
        log::info!("Resize");
        self.resize_swapchain(ctx);
    }

    fn close(&mut self, ctx: &Context) {
        log::info!("Closing");

        ctx.device.wait();

        log::info!("Closed");
    }
}

impl App {
    fn immediate_submit<F>(&self, callback: F)
    where
        F: Fn(&CommandBuffer),
    {
        log::info!("Submit immediate command");
        self.immediate_fence.reset();
        assert!(!self.immediate_fence.signaled());

        self.immediate_cmd.reset();

        self.immediate_cmd.begin();

        callback(&self.immediate_cmd);

        self.immediate_cmd.end();

        self.immediate_cmd
            .submit2(None, None, &self.immediate_fence);

        self.immediate_fence.wait();
        log::info!("Immediate command done");
    }

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

    fn draw_scene(&self, cmd: &CommandBuffer, draw_extent: ImageExtent2D) {
        cmd.begin_rendering(&self.draw_image, draw_extent, None, false, [1.; 4]);
        cmd.bind_pipeline(&self.scene_pipeline);
        cmd.push_constants(self.scene_pipeline_layout.clone(), &self.scene_data.raw());
        cmd.set_viewport(draw_extent.width, draw_extent.height);
        cmd.bind_index_buffer::<u32>(&self.mesh_resource.index_buffer);
        cmd.draw_indexed(6, 1);
        cmd.end_rendering();
    }

    fn get_mesh() -> Arc<Mesh> {
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

        Mesh::builder("quad")
            .add_indices(&indices)
            .add_vertices(&vertices)
            .build()
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
