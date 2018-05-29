use std::boxed::Box;
use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

use vulkano::command_buffer::{AutoCommandBufferBuilder};
use vulkano::sync::{GpuFuture, now, FlushError};

use context::Context;
use display::Display;
use render::Renderer;
use render::shader::{DefaultShader, Shader};
use scene::SceneGraph;

pub struct Batch {
    renderer: Renderer,
    shader: Box<Shader>,
    context: Arc<Context>,
    builder: Option<AutoCommandBufferBuilder>,
    last_frame: Box<GpuFuture>,
    next_frame: Box<GpuFuture>,
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
            last_frame: Box::new(now(device.clone())) as Box<GpuFuture>,
            next_frame: Box::new(now(device)) as Box<GpuFuture>,
            swapchain_idx: 0
        }
    }

    pub fn begin(&mut self) {
        self.last_frame.cleanup_finished();

        if let Ok((id, future)) = self.renderer.new_frame() {
            self.next_frame = future;
            self.swapchain_idx = id;

            let builder = self.renderer.new_builder(id);

            self.builder = Some(builder);
        } else {
            self.builder = None;
        }
    }

    pub fn end(&mut self) {
        if self.builder.is_none() {
            return;
        };

        let builder = self.builder.take();

        let command_buffer = builder.unwrap().end_render_pass().unwrap().build().unwrap();

        let device = self.context.device();
        let queue = self.context.queue();
        let swapchain = self.renderer.swapchain();

        let last_frame = mem::replace(&mut self.last_frame, Box::new(now(device.clone())) as Box<GpuFuture>);
        let next_frame = mem::replace(&mut self.next_frame, Box::new(now(device.clone())) as Box<GpuFuture>);

        let future = last_frame.join(next_frame)
            .then_execute(queue.clone(), command_buffer).unwrap()
            .then_swapchain_present(queue, swapchain, self.swapchain_idx)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.last_frame = Box::new(future) as Box<_>;
            },
            Err(FlushError::OutOfDate) => {
                self.last_frame = Box::new(now(device.clone())) as Box<_>;
            },
            Err(e) => {
                println!("{:?}", e);
                self.last_frame = Box::new(now(device.clone())) as Box<_>;
            }
        }
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
            let builder = self.builder.take();
            let camera = graph.camera();
            let light = graph.light();

            let new_builder = self.renderer.draw_list(
                builder.unwrap(), camera, light, &mut self.shader, list.iter());

            self.builder = Some(new_builder);
        }
    }

    pub fn resize(&mut self) {
        self.renderer.resize();
    }
}
