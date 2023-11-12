use std::sync::Arc;

use crate::{
    context::Gfx,
    graph::batch::Batch,
    model::VertexFlag,
    resources::ResourceManager,
    shader::{Shader, ShaderBindGroup},
};

pub struct RenderPass {
    pub name: String,
    shader: Option<Arc<Shader>>,
    clear: bool,
    enabled: bool,
}

impl RenderPass {
    pub fn new(name: &str, clear: bool) -> Self {
        RenderPass {
            name: name.to_string(),
            shader: None,
            clear,
            enabled: true,
        }
    }

    pub fn with_shader(name: &str, shader: Arc<Shader>, clear: bool) -> Self {
        RenderPass {
            name: name.to_string(),
            shader: Some(shader),
            clear,
            enabled: true,
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn prepare(&self, gfx: &Gfx, resource_manager: &mut ResourceManager, batch: &Batch<'_>) {
        for item in &batch.items {
            let shader = match &self.shader {
                Some(shader) => shader,
                None => &item.model.shader,
            };

            if let Some(instance_data) = item.instances {
                resource_manager.update_instance_data(gfx, item.model.clone(), instance_data);
            }

            for (mesh, material) in &item.model.meshes {
                resource_manager.update_mesh_buffer(gfx, mesh, shader);
                if let Some(material) = material {
                    if shader.vertex_flags.contains(VertexFlag::TEXTURE) {
                        resource_manager.update_material_bind_group(
                            gfx,
                            &material,
                            shader.layout(ShaderBindGroup::Material),
                        );
                    }
                }
            }
        }
    }

    pub fn render(
        &self,
        gfx: &Gfx,
        encoder: &mut wgpu::CommandEncoder,
        resource_manager: &mut ResourceManager,
        surface: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        batch: &Batch<'_>,
    ) {
        if !self.enabled {
            return;
        }

        self.prepare(gfx, resource_manager, batch);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&self.name),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: if self.clear {
                        wgpu::LoadOp::Clear(wgpu::Color::BLACK)
                    } else {
                        wgpu::LoadOp::Load
                    },
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
            let shader = match &self.shader {
                Some(shader) => shader,
                None => &item.model.shader,
            };

            render_pass.insert_debug_marker(&format!(
                "Using shader: {}, pipeline: {}",
                &self.name, &shader.pipeline.name
            ));
            render_pass.set_pipeline(&shader.pipeline.pipeline);
            render_pass.set_bind_group(0, &batch.camera_resource.bind_group, &[]);
            render_pass.set_bind_group(1, &batch.light_resource.bind_group, &[]);

            let model_instance = resource_manager.instance_data(item.model.clone());

            render_pass.set_vertex_buffer(1, model_instance.instance_buffer.slice(..));
            for (mesh, material) in &item.model.meshes {
                let mesh_data = resource_manager.mesh_buffer(mesh, shader);
                render_pass.set_vertex_buffer(0, mesh_data.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh_data.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                if let Some(material) = material {
                    if shader.vertex_flags.contains(VertexFlag::TEXTURE) {
                        let bind_group = resource_manager.material_bind_group(&material);
                        render_pass.insert_debug_marker("Draw with texture");
                        render_pass.set_bind_group(2, &bind_group, &[]);
                    }
                }
                render_pass.draw_indexed(
                    0..mesh.indices.len() as _,
                    0,
                    0..model_instance.instance_count as _,
                );
            }
            render_pass.pop_debug_group();
        }
    }
}
