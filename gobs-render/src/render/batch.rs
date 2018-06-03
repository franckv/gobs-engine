use std::boxed::Box;
use std::collections::HashMap;
use std::sync::Arc;
use std::slice::Iter;

use cgmath::Matrix4;

use vulkano::buffer::{BufferUsage, ImmutableBuffer};
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DrawIndirectCommand, DynamicState};
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::viewport::Viewport;

use RenderInstance;
use cache::{MeshCache, TextureCache};
use context::Context;
use display::Display;
use pipeline::{Pipeline, LinePipeline, TrianglePipeline};
use scene::{Camera, Light, SceneGraph, SceneData};
use scene::model::{PrimitiveType, RenderObject};

pub struct Command {
    command: AutoCommandBuffer
}

impl Command {
    fn new(command: AutoCommandBuffer) -> Self {
        Command {
            command: command
        }
    }

    pub fn command(self) -> AutoCommandBuffer {
        self.command
    }
}

pub struct Batch {
    context: Arc<Context>,
    display: Arc<Display>,
    line_pipeline: Box<Pipeline>,
    triangle_pipeline: Box<Pipeline>,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    texture_cache: TextureCache,
    mesh_cache: MeshCache
}

impl Batch {
    pub fn new(display: Arc<Display>, context: Arc<Context>,
        render_pass: Arc<RenderPassAbstract + Send + Sync>) -> Self {

        let line_pipeline = LinePipeline::new(context.clone(),
            Subpass::from(render_pass.clone(), 0).unwrap());

        let triangle_pipeline = TrianglePipeline::new(context.clone(),
            Subpass::from(render_pass.clone(), 0).unwrap());

        Batch {
            context: context.clone(),
            display: display.clone(),
            line_pipeline: line_pipeline,
            triangle_pipeline: triangle_pipeline,
            render_pass: render_pass,
            texture_cache: TextureCache::new(context.clone()),
            mesh_cache: MeshCache::new(context)
        }
    }

    pub fn draw_graph(&mut self, graph: &mut SceneGraph) -> Command {
        let mut builder = AutoCommandBufferBuilder::secondary_graphics_one_time_submit(
            self.context.device(), self.context.queue().family(),
            Subpass::from(self.render_pass.clone(), 0).unwrap()).unwrap();

        let map = {
            let mut map = HashMap::new();

            graph.foreach(|data, transform| {
                match data {
                    SceneData::Object(o) => {
                        let mesh = o.mesh();
                        let id = mesh.id();

                        if !map.contains_key(&id) {
                            map.insert(id, Vec::new());
                        }
                        map.get_mut(&id).unwrap().push((o.clone(), transform));
                    },
                    _ => () // TODO
                }
            });

            map
        };

        for (_id, list) in map {
            let camera = graph.camera();
            let light = graph.light();

            builder = self.draw_list(
                builder, camera, light, list.iter())
        }

        Command::new(builder.build().unwrap())
    }

    fn draw_list(&mut self, builder: AutoCommandBufferBuilder,
        camera: &Camera, light: &Light,
        instances: Iter<(Arc<RenderObject>, Matrix4<f32>)>)
        -> AutoCommandBufferBuilder {
        let instance_buffer = self.create_instance_buffer(instances.clone());
        let indirect_buffer = self.create_indirect_buffer(instances.clone());

        // TODO: change this
        let first = instances.as_slice().get(0).unwrap();
        let mesh = first.0.mesh();
        let texture = first.0.texture().unwrap();

        let ref mut pipeline = match mesh.primitive_type() {
            PrimitiveType::Triangle => &mut self.triangle_pipeline,
            PrimitiveType::Line => &mut self.line_pipeline,
        };

        let mesh = self.mesh_cache.get(mesh);
        let texture = self.texture_cache.get(texture);

        let set = pipeline.get_descriptor_set(camera.combined(),
            light, texture);

        /* TODO: should be the size of the swapchain.
        However, if display has been resized, swapchain will be recreated
        on batch submission anyway
        */
        let dim = self.display.dimensions();

        let dynamic_state = DynamicState {
            viewports: Some(vec![Viewport {
                origin: [0., 0.],
                dimensions: [dim[0] as f32, dim[1] as f32],
                depth_range: 0.0 .. 1.0,
            }]),
            .. DynamicState::none()
        };

        let pipeline = pipeline.get_pipeline();

        builder.draw_indirect(
            pipeline, dynamic_state,
            vec![mesh.buffer(), instance_buffer],
            indirect_buffer, set.clone(), ()).unwrap()
    }

    fn create_instance_buffer(&mut self, instances: Iter<(Arc<RenderObject>,
        Matrix4<f32>)>) -> Arc<ImmutableBuffer<[RenderInstance]>> {
        let mut instances_data: Vec<RenderInstance> = Vec::new();

        for instance in instances {
            let instance_data: RenderInstance =
                instance.0.get_instance_data(instance.1).into();
            instances_data.push(instance_data);
        }

        let (instance_buffer, _future) =
            ImmutableBuffer::from_iter(instances_data.into_iter(),
        BufferUsage::vertex_buffer(), self.context.queue()).unwrap();

        instance_buffer
    }

    fn create_indirect_buffer(&mut self, instances: Iter<(Arc<RenderObject>,
        Matrix4<f32>)>) -> Arc<ImmutableBuffer<[DrawIndirectCommand]>> {
        let mesh = instances.as_slice().get(0).unwrap().0.mesh();

        let indirect_data = vec![DrawIndirectCommand {
            vertex_count: mesh.vlist().len() as u32,
            instance_count: instances.len() as u32,
            first_vertex: 0,
            first_instance: 0
        }];

        let (indirect_buffer, _future) = ImmutableBuffer::from_iter(indirect_data.into_iter(),
        BufferUsage::indirect_buffer(), self.context.queue()).unwrap();

        indirect_buffer
    }
}
