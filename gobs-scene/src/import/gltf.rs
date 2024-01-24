use std::sync::Arc;

use gltf::mesh::util::{ReadColors, ReadIndices};
use gobs_core::geometry::{mesh::Mesh, primitive::Primitive, vertex::VertexData};

pub fn load_gltf(file: &str) -> Vec<Arc<Mesh>> {
    let (doc, buffers, _) = gltf::import(file).unwrap();

    let mut indices = Vec::new();
    let mut vertices = Vec::new();
    let mut meshes = Vec::new();

    for m in doc.meshes() {
        log::info!("Mesh #{}: {:?}", m.index(), m.name().unwrap_or_default());

        let mut mesh = Mesh::builder(m.name().unwrap_or_default());

        for p in m.primitives() {
            log::info!("Primitive: {}", p.index());

            indices.clear();
            vertices.clear();

            let reader = p.reader(|buffer| Some(&buffers[buffer.index()]));

            if let Some(read_indices) = reader.read_indices() {
                match read_indices {
                    ReadIndices::U8(iter) => {
                        for idx in iter {
                            indices.push(idx as u32);
                        }
                    }
                    ReadIndices::U16(iter) => {
                        for idx in iter {
                            indices.push(idx as u32);
                        }
                    }
                    ReadIndices::U32(iter) => {
                        for idx in iter {
                            indices.push(idx);
                        }
                    }
                }
            }

            if let Some(iter) = reader.read_positions() {
                for pos in iter {
                    vertices.push(
                        VertexData::builder()
                            .position(pos.into())
                            .padding(true)
                            .build(),
                    );
                }
            }

            if let Some(iter) = reader.read_normals() {
                for (i, normal) in iter.enumerate() {
                    vertices[i].normal = normal.into();
                }
            }

            if let Some(read_tex_coords) = reader.read_tex_coords(0) {
                match read_tex_coords {
                    gltf::mesh::util::ReadTexCoords::U8(_) => todo!(),
                    gltf::mesh::util::ReadTexCoords::U16(_) => todo!(),
                    gltf::mesh::util::ReadTexCoords::F32(iter) => {
                        for (i, texture) in iter.enumerate() {
                            vertices[i].texture = texture.into();
                        }
                    }
                }
            }

            if let Some(read_colors) = reader.read_colors(0) {
                match read_colors {
                    ReadColors::RgbaU8(iter) => {
                        for (i, color) in iter.enumerate() {
                            vertices[i].color = [
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
                for i in 0..vertices.len() {
                    vertices[i].color = (vertices[i].normal, 1.).into();
                }
            }

            mesh = mesh.add_primitive(
                Primitive::builder()
                    .add_indices(&indices)
                    .add_vertices(&vertices)
                    .build(),
            )
        }

        meshes.push(mesh.build());
    }

    meshes
}
