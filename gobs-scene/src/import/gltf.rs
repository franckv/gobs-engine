use std::sync::Arc;

use gltf::mesh::util::{ReadColors, ReadIndices};

use gobs_core::entity::uniform::{UniformLayout, UniformProp};
use gobs_material::{vertex::VertexData, Material};
use gobs_render::context::Context;

use crate::{mesh::Mesh, model::Model};

pub fn load_gltf(ctx: &Context, file: &str) -> Vec<Arc<Model>> {
    let (doc, buffers, _) = gltf::import(file).unwrap();

    let mut models = Vec::new();

    let model_data_layout = UniformLayout::builder()
        .prop("world_matrix", UniformProp::Mat4F)
        .prop("vertex_buffer_address", UniformProp::U64)
        .build();

    let material = Material::new(ctx, model_data_layout.clone());

    for m in doc.meshes() {
        let name = m.name().unwrap_or_default();
        log::info!(
            "Mesh #{}: {:?}, {} primitives",
            m.index(),
            name,
            m.primitives().len()
        );

        let mut meshes = Vec::new();

        for p in m.primitives() {
            let mut mesh_data = Mesh::builder();

            let reader = p.reader(|buffer| Some(&buffers[buffer.index()]));

            if let Some(read_indices) = reader.read_indices() {
                match read_indices {
                    ReadIndices::U8(iter) => {
                        for idx in iter {
                            mesh_data = mesh_data.index(idx as u32);
                        }
                    }
                    ReadIndices::U16(iter) => {
                        for idx in iter {
                            mesh_data = mesh_data.index(idx as u32);
                        }
                    }
                    ReadIndices::U32(iter) => {
                        for idx in iter {
                            mesh_data = mesh_data.index(idx as u32);
                        }
                    }
                }
            }

            if let Some(iter) = reader.read_positions() {
                for pos in iter {
                    mesh_data = mesh_data.vertex(
                        VertexData::builder()
                            .position(pos.into())
                            .padding(true)
                            .build(),
                    );
                }
            }

            if let Some(iter) = reader.read_normals() {
                for (i, normal) in iter.enumerate() {
                    mesh_data.vertices[i].normal = normal.into();
                }
            }

            if let Some(read_tex_coords) = reader.read_tex_coords(0) {
                match read_tex_coords {
                    gltf::mesh::util::ReadTexCoords::U8(_) => todo!(),
                    gltf::mesh::util::ReadTexCoords::U16(_) => todo!(),
                    gltf::mesh::util::ReadTexCoords::F32(iter) => {
                        for (i, texture) in iter.enumerate() {
                            mesh_data.vertices[i].texture = texture.into();
                        }
                    }
                }
            }

            if let Some(read_colors) = reader.read_colors(0) {
                match read_colors {
                    ReadColors::RgbaU8(iter) => {
                        for (i, color) in iter.enumerate() {
                            mesh_data.vertices[i].color = [
                                color[0] as f32 / 255.,
                                color[1] as f32 / 255.,
                                color[2] as f32 / 255.,
                                color[3] as f32 / 255.,
                            ]
                            .into();
                        }
                    }
                    _ => todo!(),
                }
            } else {
                for i in 0..mesh_data.vertices.len() {
                    mesh_data.vertices[i].color = (mesh_data.vertices[i].normal, 1.).into();
                }
            }

            meshes.push(mesh_data.build());
        }

        let model = Model::new(
            ctx,
            name,
            &meshes,
            &[material.clone()],
            model_data_layout.clone(),
        );
        models.push(model);
    }

    models
}
