use std::sync::Arc;

use glam::{Quat, Vec3};
use gobs::{
    game::{
        app::{Application, RenderError, Run},
        input::{Input, Key},
    },
    gobs_core::{
        entity::uniform::{UniformData, UniformLayout, UniformProp, UniformPropData},
        Transform,
    },
    render::context::Context,
    scene::{
        graph::scenegraph::{Node, NodeValue},
        import::gltf,
        scene::Scene,
    },
    vulkan::{
        command::{CommandBuffer, CommandPool},
        descriptor::{
            DescriptorSet, DescriptorSetLayout, DescriptorSetPool, DescriptorStage, DescriptorType,
        },
        image::{ColorSpace, Image, ImageExtent2D, ImageFormat, ImageLayout, ImageUsage},
        pipeline::{Pipeline, PipelineLayout, Shader, ShaderType},
        swapchain::{PresentationMode, SwapChain},
        sync::Semaphore,
    },
};
use uuid::Uuid;

const SHADER_DIR: &str = "examples/shaders";
const ASSET_DIR: &str = "examples/assets";

struct FrameData {
    pub command_buffer: CommandBuffer,
    pub swapchain_semaphore: Semaphore,
    pub render_semaphore: Semaphore,
}

impl FrameData {
    pub fn new(ctx: &Context) -> Self {
        let command_pool = CommandPool::new(ctx.device.clone(), &ctx.queue.family);
        let command_buffer =
            CommandBuffer::new(ctx.device.clone(), ctx.queue.clone(), command_pool, "Frame");

        let swapchain_semaphore = Semaphore::new(ctx.device.clone(), "Swapchain");
        let render_semaphore = Semaphore::new(ctx.device.clone(), "Render");

        FrameData {
            command_buffer,
            swapchain_semaphore,
            render_semaphore,
        }
    }

    pub fn reset(&mut self) {
        self.command_buffer.reset();
    }
}

#[allow(unused)]
struct App {
    frame_number: usize,
    frames: Vec<FrameData>,
    swapchain: SwapChain,
    swapchain_images: Vec<Image>,
    draw_image: Image,
    depth_image: Image,
    render_scaling: f32,
    draw_ds_pool: DescriptorSetPool,
    draw_ds_layout: Arc<DescriptorSetLayout>,
    draw_ds: DescriptorSet,
    bg_pipeline: Pipeline,
    bg_pipeline_layout: Arc<PipelineLayout>,
    scene: Scene,
}

impl Run for App {
    async fn create(ctx: &Context) -> Self {
        log::info!("Create");

        let swapchain = Self::create_swapchain(ctx);
        let swapchain_images = swapchain.create_images();

        let extent = ctx.surface.get_extent(ctx.device.clone());
        let draw_image = Image::new(
            "color",
            ctx.device.clone(),
            ctx.color_format,
            ImageUsage::Color,
            extent,
            ctx.allocator.clone(),
        );

        let depth_image = Image::new(
            "depth",
            ctx.device.clone(),
            ctx.depth_format,
            ImageUsage::Depth,
            extent,
            ctx.allocator.clone(),
        );

        let draw_ds_layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::StorageImage, DescriptorStage::Compute)
            .build(ctx.device.clone());
        let mut draw_ds_pool =
            DescriptorSetPool::new(ctx.device.clone(), draw_ds_layout.clone(), 10);
        let draw_ds = draw_ds_pool.allocate();

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
            PipelineLayout::new(ctx.device.clone(), &[draw_ds_layout.clone()], 0);
        let bg_pipeline = Pipeline::compute_builder(ctx.device.clone())
            .layout(bg_pipeline_layout.clone())
            .compute_shader("main", compute_shader)
            .build();

        let scene = Scene::new(ctx, draw_image.extent);

        let frames = (0..ctx.frames_in_flight)
            .map(|_| FrameData::new(ctx))
            .collect();

        App {
            frame_number: 0,
            frames,
            swapchain,
            swapchain_images,
            draw_image,
            depth_image,
            render_scaling: 1.,
            draw_ds_pool,
            draw_ds_layout,
            draw_ds,
            bg_pipeline,
            bg_pipeline_layout,
            scene,
        }
    }

    fn start(&mut self, ctx: &Context) {
        log::trace!("Start");

        self.load_scene(ctx);
    }

    fn update(&mut self, ctx: &Context, _delta: f32) {
        log::trace!("Update");

        self.scene.update(ctx, self.frame_number);
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

        self.new_frame(ctx);

        let frame = &self.frames[self.current_frame_id(ctx)];
        let cmd = &frame.command_buffer;

        let Ok(image_index) = self.swapchain.acquire_image(&frame.swapchain_semaphore) else {
            return Err(RenderError::Outdated);
        };

        self.draw_image.invalidate();
        self.depth_image.invalidate();
        self.swapchain_images[image_index as usize].invalidate();

        cmd.begin();

        cmd.begin_label(&format!("Frame {}", self.frame_number));

        cmd.transition_image_layout(&mut self.draw_image, ImageLayout::General);

        cmd.begin_label("Draw background");
        self.draw_background(cmd, draw_extent);
        cmd.end_label();

        cmd.transition_image_layout(&mut self.draw_image, ImageLayout::Color);
        cmd.transition_image_layout(&mut self.depth_image, ImageLayout::Depth);

        cmd.begin_label("Draw scene");
        self.draw_scene(ctx, cmd, draw_extent);
        cmd.end_label();

        cmd.transition_image_layout(&mut self.draw_image, ImageLayout::TransferSrc);

        let swapchain_image = &mut self.swapchain_images[image_index as usize];

        cmd.transition_image_layout(swapchain_image, ImageLayout::TransferDst);

        cmd.copy_image_to_image(
            &self.draw_image,
            draw_extent,
            swapchain_image,
            swapchain_image.extent,
        );

        cmd.transition_image_layout(swapchain_image, ImageLayout::Present);

        cmd.end_label();

        cmd.end();

        cmd.submit2(
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

    fn input(&mut self, ctx: &Context, input: Input) {
        log::trace!("Input");

        match input {
            Input::KeyPressed(key) => match key {
                Key::E => self.render_scaling = (self.render_scaling + 0.1).min(1.),
                Key::A => self.render_scaling = (self.render_scaling - 0.1).max(0.1),
                Key::D => log::info!("{:?}", ctx.allocator.allocator.lock().unwrap()),
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
    fn current_frame_id(&self, ctx: &Context) -> usize {
        self.frame_number % ctx.frames_in_flight
    }

    fn new_frame(&mut self, ctx: &Context) {
        let frame_id = self.frame_number % ctx.frames_in_flight;
        let frame = &mut self.frames[frame_id];
        let cmd = &frame.command_buffer;

        cmd.fence.wait_and_reset();
        debug_assert!(!cmd.fence.signaled());

        frame.reset();
    }

    #[allow(unused)]
    fn clear_background(&self, cmd: &CommandBuffer) {
        let flash = (self.frame_number as f32 / 120.).sin().abs();
        cmd.clear_color(&self.draw_image, [flash, 0., 0., 1.]);
    }

    fn draw_background(&self, cmd: &CommandBuffer, draw_extent: ImageExtent2D) {
        cmd.bind_pipeline(&self.bg_pipeline);
        cmd.bind_descriptor_set(&self.draw_ds, 0, &self.bg_pipeline);
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

        cmd.set_viewport(draw_extent.width, draw_extent.height);

        let mut last_material = Uuid::nil();
        self.scene
            .graph
            .visit(self.scene.graph.root, &mut |transform, model| {
                if let NodeValue::Model(model) = model {
                    let world_matrix = transform.matrix;

                    let model_data = UniformData::new(
                        &model.model_data_layout,
                        &[
                            UniformPropData::Mat4F(world_matrix.to_cols_array_2d()),
                            UniformPropData::U64(
                                model.mesh.vertex_buffer.address(ctx.device.clone()),
                            ),
                        ],
                    );

                    for primitive in &model.mesh.primitives {
                        let material = &model.materials[primitive.material];
                        let pipeline = &material.pipeline;

                        if last_material != material.id {
                            cmd.bind_pipeline(&material.pipeline);
                            cmd.bind_descriptor_set(
                                &self.scene.uniform_ds(ctx, self.frame_number),
                                0,
                                pipeline,
                            );
                            cmd.bind_descriptor_set(&material.material_ds, 1, pipeline);
                            last_material = material.id;
                        }

                        cmd.push_constants(material.pipeline.layout.clone(), &model_data.raw());
                        cmd.bind_index_buffer::<u32>(&model.mesh.index_buffer, primitive.offset);
                        cmd.draw_indexed(primitive.len, 1);
                    }
                }
            });

        cmd.end_rendering();
    }

    #[allow(unused)]
    fn load_scene(&mut self, ctx: &Context) {
        let models = gltf::load_gltf(ctx, &format!("{}/basicmesh.glb", ASSET_DIR));

        let i_max = 3;
        let j_max = 3;
        let x_range = (-5., 5.);
        let y_range = (-3., 3.);
        let scale = 0.7;

        let model = models[2].clone();

        for i in 0..=i_max {
            for j in 0..=j_max {
                let x = x_range.0 + (i as f32) * (x_range.1 - x_range.0) / (i_max as f32);
                let y = y_range.0 + (j as f32) * (y_range.1 - y_range.0) / (j_max as f32);
                let transform = Transform::new(
                    [x, y, -7.].into(),
                    Quat::IDENTITY,
                    Vec3::new(scale, -scale, scale),
                );
                let node = Node::new(NodeValue::Model(model.clone()), transform);
                self.scene.graph.insert(self.scene.graph.root, node);
            }
        }
    }

    #[allow(unused)]
    fn load_scene2(&mut self, ctx: &Context) {
        let models = gltf::load_gltf(ctx, &format!("{}/basicmesh.glb", ASSET_DIR));

        let scale = 1.;

        let model = models[2].clone();

        let model_data_layout = UniformLayout::builder()
            .prop("world_matrix", UniformProp::Mat4F)
            .prop("vertex_buffer_address", UniformProp::U64)
            .build();

        let transform = Transform::new(
            [0., 0., -5.].into(),
            Quat::IDENTITY,
            Vec3::new(scale, -scale, scale),
        );
        let node = Node::new(NodeValue::Model(model.clone()), transform);
        self.scene.graph.insert(self.scene.graph.root, node);
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
