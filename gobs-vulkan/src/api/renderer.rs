use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

use uuid::Uuid;
use winit::Window;

use scene::model::Transform;
use scene::model::Vertex;

use api::context::Context;
use api::display::Display;
use api::frame::Frame;
use api::instance::VertexInstance;
use api::model::Model;
use api::model_instance::ModelInstance;

use backend::descriptor::{DescriptorSetLayout,
                          DescriptorSetPool, DescriptorSetResources};
use backend::image::{ColorSpace, ImageFormat, Sampler};
use backend::instance::Instance;
use backend::physical::PhysicalDevice;
use backend::pipeline::{Pipeline, Shader, PipelineLayout,
                        PipelineLayoutBindingType, PipelineLayoutBindingStage,
                        PipelineLayoutBuilder, VertexAttributeFormat,
                        VertexLayoutBindingType, VertexLayoutBuilder};
use backend::renderpass::RenderPass;
use backend::surface::{Surface, SurfaceFormat};

use utils::timer::Timer;

macro_rules! offset_of {
    ($base: path, $field: ident) => {
        #[allow(unused_unsafe)]
        unsafe{
            let b: $base = mem::uninitialized();
            (&b.$field as *const _ as usize) - (&b as *const _ as usize)
        }
    }
}

pub struct Renderer {
    pub context: Arc<Context>,
    pub display: Display,
    pub renderpass: Arc<RenderPass>,
    pub descriptor_layout: Arc<DescriptorSetLayout>,
    pub descriptor_pool: DescriptorSetPool,
    pub pipeline_layout: Arc<PipelineLayout>,
    pub pipeline: Pipeline,
    pub sampler: Sampler,
    pub frames: Vec<Frame<Transform, VertexInstance>>,
    pub current_frame: usize,
    pub current_texture: Uuid,
    pub max_instances: usize
}

impl Renderer {
    pub fn new(title: &str, window: Window,
               max_instances: usize, max_draws: usize) -> Self {
        let instance = Instance::new(title, 0);

        let surface = Surface::new(instance.clone(), window);

        let context = Context::new(instance, &surface);

        let format = Self::get_surface_format(&surface,
                                              &context.device_ref().p_device);

        let renderpass = RenderPass::new(
            context.device(), format.format);

        let display = Display::new(context.clone(),
                                   surface.clone(),
                                   format,
                                   renderpass.clone());

        let vshader = Shader::from_file("examples/assets/shaders/vert.spv",
                                        context.device());
        let fshader = Shader::from_file("examples/assets/shaders/frag.spv",
                                        context.device());

        let vertex_layout = VertexLayoutBuilder::new()
            .binding::<Vertex>(VertexLayoutBindingType::Vertex)
            .attribute(VertexAttributeFormat::Vec3,
                       offset_of!(Vertex, position))
            .attribute(VertexAttributeFormat::Vec3,
                       offset_of!(Vertex, normal))
            .attribute(VertexAttributeFormat::Vec2,
                       offset_of!(Vertex, tex_uv))
            .binding::<VertexInstance>(VertexLayoutBindingType::Instance)
            .attribute(VertexAttributeFormat::Mat4,
                       offset_of!(VertexInstance, matrix))
            .build();

        let pipeline_layout = PipelineLayoutBuilder::new()
            .binding(PipelineLayoutBindingType::Uniform,
                     PipelineLayoutBindingStage::Vertex)
            .binding(PipelineLayoutBindingType::ImageSampler,
                     PipelineLayoutBindingStage::Fragment)
            .build();

        let frame_count = display.image_count;

        let descriptor_layout =
            DescriptorSetLayout::new(context.device(), &pipeline_layout);

        let descriptor_pool =
            DescriptorSetPool::new(context.device(),
                                   descriptor_layout.clone(),
                                   &pipeline_layout,
                                   frame_count * max_draws);

        let pipeline =
            Pipeline::new(context.device(), vshader, fshader,
                          vertex_layout,
                          descriptor_layout.clone(),
                          renderpass.clone(),
                          0);

        let sampler = Sampler::new(context.device());

        let frames =
            Frame::new(&context, frame_count, max_instances);

        Renderer {
            context,
            display,
            renderpass,
            descriptor_layout,
            descriptor_pool,
            pipeline_layout,
            pipeline,
            sampler,
            frames,
            current_frame: 0,
            current_texture: Uuid::nil(),
            max_instances
        }
    }

    fn get_surface_format(surface: &Arc<Surface>,
                          p_device: &PhysicalDevice) -> SurfaceFormat {
        let formats =
            surface.get_available_format(p_device);

        *formats.iter().find(|f| {
            f.format == ImageFormat::B8g8r8a8Unorm &&
                f.color_space == ColorSpace::SrgbNonlinear
        }).unwrap()
    }

    pub fn update_view_proj(&mut self, view_proj: Transform) {
        let frame = &mut self.frames[self.current_frame];

        let v = vec![view_proj];
        frame.view_proj_buffer.copy(&v);
    }

    pub fn new_frame(&mut self) -> Result<(), ()> {
        let error = {
            let frame = &mut self.frames[self.current_frame];

            frame.submit_fence.wait();

            match self.display.next_image(&frame.wait_image) {
                Err(_) => true,
                _ => false
            }
        };

        if error {
            for frame in &mut self.frames {
                frame.dirty = true;
            }
        }

        match error {
            true => Err(()),
            false => Ok(())
        }
    }

    pub fn submit_frame(&mut self) {
        let error = {
            let frame = &mut self.frames[self.current_frame];

            frame.submit_fence.reset();

            frame.command_buffer.submit(self.context.queue(),
                                        Some(&frame.wait_image),
                                        &frame.wait_command, &frame.submit_fence);

            match self.display.present(&frame.wait_command) {
                Err(_) => true,
                _ => false
            }
        };

        if error {
            for frame in &mut self.frames {
                frame.dirty = true;
            }
        }

        self.current_frame = (self.current_frame + 1) % self.frames.len();
    }

    pub fn draw_frame(&mut self, instances: Vec<ModelInstance<Vertex, Transform>>) {
        let mut timer = Timer::new();
        debug_assert!(instances.len() <= self.max_instances);

        if self.frames[self.current_frame].dirty {
            self.begin_frame();
        }

        let instances =
            Self::sort_instances(instances);

        for id in instances.keys() {
            let instances = instances.get(&id).unwrap();

            let transforms: Vec<VertexInstance> = {
                instances.iter().map(|instance| {
                    VertexInstance {
                        matrix: instance.transform().into()
                    }
                }).collect()
            };

            let instance_count = transforms.len();

            self.update_instances(*id, transforms);

            if self.frames[self.current_frame].dirty {
                self.draw_instances(&instances[0].model(), instance_count);
            }

            debug!("Draw instances {}: {}", id, timer.delta() / 1_000_000);
        }

        if self.frames[self.current_frame].dirty {
            self.end_frame();
        }

        debug!("Draw frame: {}", timer.delta() / 1_000_000);
    }

    fn sort_instances(mut instances: Vec<ModelInstance<Vertex, Transform>>)

                      -> HashMap<Uuid, Vec<ModelInstance<Vertex, Transform>>> {
        let mut map = HashMap::new();

        for instance in instances.drain(..) {
            let id = instance.model().texture_id;
            if !map.contains_key(&id) {
                map.insert(id, Vec::new());
            }
            map.get_mut(&id).unwrap().push(instance);
        }

        map
    }

    fn update_instances(&mut self, id: Uuid, transform: Vec<VertexInstance>) {
        let frame = &mut self.frames[self.current_frame];

        frame.instance_buffer_mut(id).copy(&transform);
    }

    fn begin_frame(&mut self) {
        let frame = &mut self.frames[self.current_frame];
        let command_buffer = &mut frame.command_buffer;

        command_buffer.begin();
        command_buffer.start_render_pass(self.display.framebuffer());
        command_buffer.bind_pipeline(&self.pipeline);
        let extent = self.display.dimensions();
        command_buffer.set_viewport(extent.0, extent.1);
    }

    fn end_frame(&mut self) {
        let frame = &mut self.frames[self.current_frame];
        let command_buffer = &mut frame.command_buffer;

        command_buffer.end_render_pass();
        command_buffer.end();

        frame.dirty = false;
    }

    fn draw_instances(&mut self, model: &Arc<Model<Vertex>>, instance_count: usize) {
        let id = model.texture_id;
        let frame = &mut self.frames[self.current_frame];
        {
            let instance_buffer = &frame.instance_buffer(&id);
            &mut frame.command_buffer.bind_vertex_buffer(1, instance_buffer);
        }

        let command_buffer = &mut frame.command_buffer;

        command_buffer.bind_vertex_buffer(0, &model.vertex_buffer);
        command_buffer.bind_index_buffer(&model.index_buffer);

        let mut descriptor_set = self.descriptor_pool.next();
        DescriptorSetResources::new(&mut descriptor_set)
            .bind_buffer(&frame.view_proj_buffer, 0, 1)
            .bind_image(&model.texture, &self.sampler)
            .update();

        command_buffer.bind_descriptor_set(descriptor_set, &self.pipeline,
                                           vec![]);
        command_buffer.draw_indexed(model.index_buffer.count(),
                                    instance_count);
    }
}
