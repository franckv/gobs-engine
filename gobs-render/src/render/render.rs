use std::mem;
use std::sync::Arc;
use std::slice::Iter;

use vulkano::buffer::{BufferUsage, ImmutableBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DrawIndirectCommand, DynamicState};
use vulkano::device::{Device, Queue};
use vulkano::format::Format;
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract};
use vulkano::image::AttachmentImage;
use vulkano::image::swapchain::SwapchainImage;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain;
use vulkano::swapchain::{AcquireError, Swapchain};
use vulkano::sync::GpuFuture;

use winit::Window;

use scene::camera::Camera;
use scene::light::Light;
use model::{Instance, MeshInstance};
use render::shader::Shader;

pub struct Renderer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain<Window>>,
    images: Vec<Arc<SwapchainImage<Window>>>,
    dimensions: [u32; 2],
    framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    shader: Shader
}

impl Renderer {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>,
               swapchain: Arc<Swapchain<Window>>, images: Vec<Arc<SwapchainImage<Window>>>,
               dimensions: [u32; 2]) -> Renderer {
        let render_pass = Self::create_render_pass(device.clone(), swapchain.format());

        let shader = Shader::new(render_pass.clone(), device.clone());

        let framebuffers = Self::create_framebuffer(render_pass.clone(), device.clone(), dimensions, &images);

        Renderer {
            device: device,
            queue: queue,
            swapchain: swapchain,
            images: images,
            dimensions: dimensions,
            framebuffers: framebuffers,
            render_pass: render_pass,
            shader: shader
        }
    }

    pub fn framebuffer(&self, id: usize) -> Arc<FramebufferAbstract + Send + Sync> {
        self.framebuffers[id].clone()
    }

    pub fn swapchain(&self) -> Arc<Swapchain<Window>> {
        self.swapchain.clone()
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

    pub fn new_frame(&mut self) -> Result<(usize, Box<GpuFuture>), AcquireError> {
        let (idx, acquire_future) = match swapchain::acquire_next_image(self.swapchain.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                println!("OutOfDate");
                return Err(AcquireError::OutOfDate)
            },
            Err(err) => panic!("{:?}", err)
        };

        Ok((idx, Box::new(acquire_future)))
    }

    pub fn draw_list(&mut self, builder: AutoCommandBufferBuilder,
        camera: &Camera, light: &Light, instances: Iter<Arc<MeshInstance>>)
        -> AutoCommandBufferBuilder {
        let instance_buffer = self.create_instance_buffer(instances.clone());
        let indirect_buffer = self.create_indirect_buffer(instances.clone());

        // TODO: change this
        let first = instances.as_slice().get(0).unwrap();
        let mesh = first.mesh();
        let texture = first.texture().unwrap();

        let set = self.shader.bind()
            .matrix(camera.combined())
            .light(light)
            .texture(texture)
            .get();

        let dynamic_state = DynamicState {
            viewports: Some(vec![Viewport {
                origin: [0., 0.],
                dimensions: [self.dimensions[0] as f32, self.dimensions[1] as f32],
                depth_range: 0.0 .. 1.0,
            }]),
            .. DynamicState::none()
        };

        builder.draw_indirect(
            self.shader.pipeline(), dynamic_state,
            vec![mesh.buffer(), instance_buffer],
            indirect_buffer, set.clone(), ()).unwrap()
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }

    fn create_framebuffer(render_pass: Arc<RenderPassAbstract + Send + Sync>,
        device: Arc<Device>, dimensions: [u32; 2], images: &Vec<Arc<SwapchainImage<Window>>>)
        -> Vec<Arc<FramebufferAbstract + Send + Sync>> {

        let depth_buffer = AttachmentImage::transient(device.clone(),
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
        BufferUsage::vertex_buffer(), self.queue.clone()).unwrap();

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
        BufferUsage::indirect_buffer(), self.queue.clone()).unwrap();

        indirect_buffer
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        println!("Resize");
        self.dimensions[0] = width;
        self.dimensions[1] = height;

        let (swapchain, images) = self.swapchain.recreate_with_dimension(self.dimensions).unwrap();

        mem::replace(&mut self.swapchain, swapchain);
        mem::replace(&mut self.images, images);

        self.framebuffers = Self::create_framebuffer(self.render_pass.clone(), self.device.clone(),
            self.dimensions, &self.images)
    }
}
