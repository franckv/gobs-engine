use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

use log::debug;
use uuid::Uuid;

use gobs_scene as scene;
use gobs_utils as utils;
use gobs_vulkan as backend;

use scene::model::Transform;
use scene::model::Vertex;

use super::context::Context;
use super::display::Display;
use super::frame::Frame;
use super::model::ModelCache;

use backend::descriptor::{DescriptorSetLayout, DescriptorSetPool, 
    DescriptorSetLayoutBuilder, DescriptorType, DescriptorStage};
use backend::image::Sampler;
use backend::pipeline::{Pipeline, Shader, ShaderType,
    VertexAttributeFormat, VertexLayoutBindingType, 
                        VertexLayoutBuilder, Viewport, Rect2D, DynamicStateElem};

use utils::timer::Timer;

macro_rules! offset_of {
    ($base: path, $field: ident) => {
        #[allow(unused_unsafe)]
        unsafe{
            let b: $base = mem::zeroed();
            (&b.$field as *const _ as usize) - (&b as *const _ as usize)
        }
    }
}

pub struct Renderer {
    pub context: Arc<Context>,
    pub display: Display,
    pub descriptor_layout: Arc<DescriptorSetLayout>,
    pub descriptor_pool: DescriptorSetPool,
    pub pipeline: Pipeline,
    pub sampler: Sampler,
    pub frames: Vec<Frame>,
    pub current_frame: usize,
    pub current_texture: Uuid,
    pub max_instances: usize,
    pub current_image: usize
}

impl Renderer {
    pub fn new(context: Arc<Context>,
               display: Display,
               max_instances: usize, max_draws: usize) -> Self {
        let vshader = Shader::from_file("examples/assets/shaders/vert.spv",
                                        context.device(), ShaderType::Vertex);
        let fshader = Shader::from_file("examples/assets/shaders/frag.spv",
                                        context.device(), ShaderType::Fragment);

        let vertex_layout = VertexLayoutBuilder::new()
            .binding::<Vertex>(VertexLayoutBindingType::Vertex)
            .attribute(VertexAttributeFormat::Vec3,
                       offset_of!(Vertex, position))
            .attribute(VertexAttributeFormat::Vec3,
                       offset_of!(Vertex, normal))
            .attribute(VertexAttributeFormat::Vec2,
                       offset_of!(Vertex, tex_uv))
            .binding::<Transform>(VertexLayoutBindingType::Instance)
            .attribute(VertexAttributeFormat::Mat4,
                       offset_of!(Transform, matrix))
            .build();

        let descriptor_layout = DescriptorSetLayoutBuilder::new()
            .binding(DescriptorType::Uniform,
                DescriptorStage::Vertex)
            .binding(DescriptorType::ImageSampler,
                DescriptorStage::Fragment)
            .build(context.device());

        let frame_count = display.image_count;

        let descriptor_pool =
            DescriptorSetPool::new(context.device(),
                                   descriptor_layout.clone(),
                                   frame_count * max_draws);

        let (width, height) = display.dimensions();

        let pipeline = Pipeline::builder(context.device())
                    .renderpass(display.renderpass().clone())
                    .vertex_shader("main", vshader)
                    .fragment_shader("main", fshader)
                    .vertex_layout(&vertex_layout)
                    .viewports(vec![Viewport::new(0., 0., width as f32, height as f32)])
                    .scissors(vec![Rect2D::new(0, 0, width, height)])
                    .dynamic_states(&vec![DynamicStateElem::Viewport, DynamicStateElem::Scissor])
                    .descriptor_layout(descriptor_layout.clone())
                    .build();

        let sampler = Sampler::new(context.device());

        let frames =
            Frame::new(&context, frame_count, max_instances);

        Renderer {
            context,
            display,
            descriptor_layout,
            descriptor_pool,
            pipeline,
            sampler,
            frames,
            current_frame: 0,
            current_texture: Uuid::nil(),
            max_instances,
            current_image: 0
        }
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
                Ok(index) => {
                    self.current_image = index;
                    false
                }
            }
        };

        if error {
            for frame in &mut self.frames {
                frame.dirty = true;
            }
            Err(())
        } else {
            Ok(())
        }
    }

    pub fn submit_frame(&mut self) {
        let error = {
            let frame = &mut self.frames[self.current_frame];

            frame.submit_fence.reset();

            frame.command_buffer.submit(self.context.queue(),
                                        Some(&frame.wait_image),
                                        &frame.wait_command, &frame.submit_fence);

            match self.display.present(self.current_image, &frame.wait_command) {
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

    pub fn draw_frame(&mut self, instances: Vec<(Arc<ModelCache<Vertex>>, Transform)>) {
        let mut timer = Timer::new();
        debug_assert!(instances.len() <= self.max_instances);

        if self.frames[self.current_frame].dirty {
            self.begin_frame();
        }

        let instances =
            Self::sort_instances(instances);

        for id in instances.keys() {
            let instances = instances.get(&id).unwrap();

            let transforms: Vec<Transform> = {
                instances.iter().map(|instance| {
                    instance.1.into()
                }).collect()
            };

            let instance_count = transforms.len();

            self.update_instances(*id, transforms);

            if self.frames[self.current_frame].dirty {
                self.draw_instances(&instances[0].0, instance_count);
            }

            debug!("Draw instances {}: {}", id, timer.delta() / 1_000_000);
        }

        if self.frames[self.current_frame].dirty {
            self.end_frame();
        }

        debug!("Draw frame: {}", timer.delta() / 1_000_000);
    }

    /// group instances by texture
    fn sort_instances(mut instances: Vec<(Arc<ModelCache<Vertex>>, Transform)>)
                      -> HashMap<Uuid, Vec<(Arc<ModelCache<Vertex>>, Transform)>> {
        let mut map = HashMap::new();

        for instance in instances.drain(..) {
            let id = instance.0.texture_id;
            if !map.contains_key(&id) {
                map.insert(id, Vec::new());
            }
            map.get_mut(&id).unwrap().push(instance);
        }

        map
    }

    fn update_instances(&mut self, id: Uuid, transform: Vec<Transform>) {
        let frame = &mut self.frames[self.current_frame];

        frame.instance_buffer_mut(id).copy(&transform);
    }

    fn begin_frame(&mut self) {
        let frame = &mut self.frames[self.current_frame];
        let command_buffer = &mut frame.command_buffer;

        command_buffer.begin();
        command_buffer.start_render_pass(self.display.framebuffer(self.current_image));
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

    fn draw_instances(&mut self, model: &Arc<ModelCache<Vertex>>, instance_count: usize) {
        let id = model.texture_id;
        let frame = &mut self.frames[self.current_frame];
        {
            let instance_buffer = &frame.instance_buffer(&id);
            &mut frame.command_buffer.bind_vertex_buffer(1, instance_buffer);
        }

        let command_buffer = &mut frame.command_buffer;

        command_buffer.bind_vertex_buffer(0, &model.vertex_buffer);
        command_buffer.bind_index_buffer(&model.index_buffer);

        let descriptor_set = self.descriptor_pool.next();
        descriptor_set.start_update()
            .bind_buffer(&frame.view_proj_buffer, 0, 1)
            .bind_image(&model.texture, &self.sampler)
            .end();

        command_buffer.bind_descriptor_set(descriptor_set, &self.pipeline,
                                           vec![]);
        command_buffer.draw_indexed(model.index_buffer.count(),
                                    instance_count);
    }
}
