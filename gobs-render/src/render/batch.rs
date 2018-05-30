use std::boxed::Box;
use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

use vulkano::command_buffer::{AutoCommandBufferBuilder};
use vulkano::sync::{GpuFuture, now};

use context::Context;
use display::Display;
use render::Renderer;
use render::shader::{DefaultShader, Shader};
use scene::Camera;
use scene::Light;
use scene::SceneGraph;
use scene::model::MeshInstance;

pub struct Batch {
    renderer: Renderer,
    shader: Box<Shader>,
    context: Arc<Context>,
    builder: Option<AutoCommandBufferBuilder>,
    last_frame: Box<GpuFuture + Sync + Send>,
    swapchain_idx: usize
}

impl Batch {
    pub fn new(display: Arc<Display>, context: Arc<Context>) -> Self {
        let renderer = Renderer::new(context.clone(), display.clone());
        let shader = DefaultShader::new(context.clone());
        let device = context.device();

        Batch {
            renderer: renderer,
            shader: shader,
            context: context,
            builder: None,
            last_frame: Box::new(now(device.clone())) as Box<GpuFuture + Sync + Send>,
            swapchain_idx: 0
        }
    }

    pub fn begin(&mut self) {
        self.last_frame.cleanup_finished();

        if let Ok((id, future)) = self.renderer.new_frame() {

            let last_frame = mem::replace(&mut self.last_frame,
                Box::new(now(self.context.device())) as Box<GpuFuture + Sync + Send>);

            self.last_frame = Box::new(last_frame.join(future));

            self.swapchain_idx = id;

            let builder = self.renderer.new_builder(id);

            self.builder = Some(builder);
        } else {
            self.builder = None;
        }
    }

    pub fn end(&mut self) {
        self.builder.take().map(|builder| {
            let command_buffer = self.renderer.get_command(builder);

            let last_frame = mem::replace(&mut self.last_frame,
                Box::new(now(self.context.device())) as Box<GpuFuture + Sync + Send>);

            self.last_frame = self.renderer.submit(command_buffer,
                self.swapchain_idx, last_frame);
        });
    }

    pub fn draw_instances(&mut self, camera: &Camera, light: &Light,
        instances: Vec<Arc<MeshInstance>>) {

        self.builder = self.builder.take().and_then(|builder| {
            Some(self.renderer.draw_list(
                builder, camera, light, &mut self.shader, instances.iter()))
        });
    }

    pub fn draw_graph(&mut self, graph: &SceneGraph) {
        if self.builder.is_none() {
            return;
        };

        let instances = graph.instances();

        let map = {
            let mut map = HashMap::new();

            for instance in instances {
                let mesh = instance.mesh();
                let id = mesh.id();
                if !map.contains_key(&id) {
                    map.insert(id, Vec::new());
                }
                map.get_mut(&id).unwrap().push(instance.clone());
            }

            map
        };

        for (_id, list) in map {
            let camera = graph.camera();
            let light = graph.light();

            self.draw_instances(camera, light, list);
        }
    }

    pub fn resize(&mut self) {
        self.renderer.resize();
    }
}
