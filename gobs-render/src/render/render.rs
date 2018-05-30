use std::mem;
use std::sync::Arc;
use std::slice::Iter;

use vulkano::buffer::{BufferUsage, ImmutableBuffer};
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DrawIndirectCommand, DynamicState};
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract};
use vulkano::image::AttachmentImage;
use vulkano::image::swapchain::SwapchainImage;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain;
use vulkano::swapchain::{AcquireError, PresentMode, SurfaceTransform, Swapchain};
use vulkano::sync::{GpuFuture, now, FlushError};

use winit::Window;

use context::Context;
use display::Display;
use model::{Instance, MeshInstance};
use render::shader::Shader;
use scene::camera::Camera;
use scene::light::Light;

pub struct Renderer {
    context: Arc<Context>,
    display: Arc<Display>,
    swapchain: Arc<Swapchain<Window>>,
    framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
}

impl Renderer {
    pub fn new(context: Arc<Context>, display: Arc<Display>) -> Renderer {
        let (swapchain, images) = {
            let caps = display.surface().capabilities(
                context.device().physical_device()).expect("error");
            let alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps.supported_formats[0].0;

            Swapchain::new(context.device(), display.surface(), caps.min_image_count,
                format, display.dimensions(), 1, caps.supported_usage_flags, &context.queue(),
                SurfaceTransform::Identity, alpha, PresentMode::Fifo, true, None).expect("error")
        };

        let render_pass = Self::create_render_pass(context.device(), swapchain.format());

        let framebuffers = Self::create_framebuffer(render_pass.clone(), context.clone(),
            swapchain.dimensions(), images);

        Renderer {
            context: context,
            display: display,
            swapchain: swapchain,
            framebuffers: framebuffers,
            render_pass: render_pass,
        }
    }

    pub fn framebuffer(&self, id: usize) -> Arc<FramebufferAbstract + Send + Sync> {
        self.framebuffers[id].clone()
    }

    pub fn new_frame(&mut self) -> Result<(usize, Box<GpuFuture + Send + Sync>), AcquireError> {
        let (idx, acquire_future) = match swapchain::acquire_next_image(
            self.swapchain.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                println!("OutOfDate");
                return Err(AcquireError::OutOfDate)
            },
            Err(err) => panic!("{:?}", err)
        };

        Ok((idx, Box::new(acquire_future)))
    }

    pub fn new_builder(&self, swapchain_idx: usize) -> AutoCommandBufferBuilder {
        AutoCommandBufferBuilder::primary_one_time_submit(self.context.device(),
            self.context.queue().family()).unwrap()
        .begin_render_pass(self.framebuffer(swapchain_idx),
            false, vec![[0., 0., 1., 1.].into(), 1f32.into()]).unwrap()
    }

    pub fn get_command(&self, builder: AutoCommandBufferBuilder) -> AutoCommandBuffer {
        builder.end_render_pass().unwrap().build().unwrap()
    }

    pub fn submit<'a, T: GpuFuture + Send + Sync + 'a>(&self, command: AutoCommandBuffer,
        id: usize, last_frame: T) ->  Box<GpuFuture + Send + Sync + 'a> {
        let future = last_frame
            .then_execute(self.context.queue(), command).unwrap()
            .then_swapchain_present(self.context.queue(), self.swapchain.clone(), id)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                Box::new(future) as Box<_>
            },
            Err(FlushError::OutOfDate) => {
                Box::new(now(self.context.device())) as Box<_>
            },
            Err(e) => {
                println!("{:?}", e);
                Box::new(now(self.context.device())) as Box<_>
            }
        }
    }

    fn create_render_pass(device: Arc<Device>, format: Format) -> Arc<RenderPassAbstract + Send + Sync> {
        Arc::new(single_pass_renderpass!(device,
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: format,
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: Store,
                    format: Format::D16Unorm,
                    samples: 1,
                }
            }, pass: {
                color: [color],
                depth_stencil: {depth}
                resolve: [],
            }
        ).unwrap())
    }

    pub fn draw_list(&mut self, builder: AutoCommandBufferBuilder,
        camera: &Camera, light: &Light, shader: &mut Box<Shader>,
        instances: Iter<Arc<MeshInstance>>) -> AutoCommandBufferBuilder {
        let instance_buffer = self.create_instance_buffer(instances.clone());
        let indirect_buffer = self.create_indirect_buffer(instances.clone());

        // TODO: change this
        let first = instances.as_slice().get(0).unwrap();
        let mesh = first.mesh();
        let texture = first.texture().unwrap();
        let primitive = mesh.primitive_type();

        let set = shader.get_descriptor_set(self.render_pass.clone(),
            camera.combined(), light, texture, primitive);

        let dim = self.swapchain.dimensions();

        let dynamic_state = DynamicState {
            viewports: Some(vec![Viewport {
                origin: [0., 0.],
                dimensions: [dim[0] as f32, dim[1] as f32],
                depth_range: 0.0 .. 1.0,
            }]),
            .. DynamicState::none()
        };

        let pipeline = shader.get_pipeline(self.render_pass.clone(), primitive);

        builder.draw_indirect(
            pipeline, dynamic_state,
            vec![mesh.buffer(), instance_buffer],
            indirect_buffer, set.clone(), ()).unwrap()
    }

    fn create_framebuffer(render_pass: Arc<RenderPassAbstract + Send + Sync>,
        context: Arc<Context>, dimensions: [u32; 2],
        images: Vec<Arc<SwapchainImage<Window>>>)
        -> Vec<Arc<FramebufferAbstract + Send + Sync>> {

        let depth_buffer = AttachmentImage::transient(context.device(),
            dimensions.clone(), Format::D16Unorm).unwrap();

        images.iter().map(|image| {
            Arc::new(Framebuffer::start(render_pass.clone())
                     .add(image.clone()).unwrap()
                     .add(depth_buffer.clone()).unwrap()
                     .build().unwrap()) as Arc<FramebufferAbstract + Send + Sync>
        }).collect::<Vec<_>>()
    }

    fn create_instance_buffer(&mut self, instances: Iter<Arc<MeshInstance>>)
    -> Arc<ImmutableBuffer<[Instance]>> {
        let mut instances_data: Vec<Instance> = Vec::new();

        for instance in instances {
            instances_data.push(instance.get_instance_data());
        }

        let (instance_buffer, _future) = ImmutableBuffer::from_iter(instances_data.into_iter(),
        BufferUsage::vertex_buffer(), self.context.queue()).unwrap();

        instance_buffer
    }

    fn create_indirect_buffer(&mut self, instances: Iter<Arc<MeshInstance>>)
    -> Arc<ImmutableBuffer<[DrawIndirectCommand]>> {
        let mesh = instances.as_slice().get(0).unwrap().mesh();

        let indirect_data = vec![DrawIndirectCommand {
            vertex_count: mesh.size() as u32,
            instance_count: instances.len() as u32,
            first_vertex: 0,
            first_instance: 0
        }];

        let (indirect_buffer, _future) = ImmutableBuffer::from_iter(indirect_data.into_iter(),
        BufferUsage::indirect_buffer(), self.context.queue()).unwrap();

        indirect_buffer
    }

    pub fn resize(&mut self) {
        let (swapchain, images) = self.swapchain.recreate_with_dimension(
            self.display.dimensions()).unwrap();

        mem::replace(&mut self.swapchain, swapchain);

        self.framebuffers = Self::create_framebuffer(self.render_pass.clone(),
            self.context.clone(), self.swapchain.dimensions(), images)
    }
}
