use crate::graph::batch::Batch;

pub struct RenderPass {
    name: String,
}

impl RenderPass {
    pub fn new(name: &str) -> Self {
        RenderPass {
            name: name.to_string(),
        }
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        batch: &Batch<'_>,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&self.name),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        for item in &batch.items {
            render_pass.push_debug_group("Render batch item");
            item.model.shader.draw_instanced(
                &mut render_pass,
                item.model,
                batch.camera_resource,
                batch.light_resource,
                item.instances_buffer.unwrap(),
                item.instances_count as _,
            );
            render_pass.pop_debug_group();
        }
    }
}
