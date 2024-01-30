use std::sync::Arc;

use gltf::mesh::util::{ReadColors, ReadIndices};

use crate::geometry::{mesh::Mesh, vertex::VertexData};

pub fn load_gltf(file: &str) -> Vec<Arc<Mesh>> {
    let (doc, buffers, _) = gltf::import(file).unwrap();

    let mut meshes = Vec::new();

    for m in doc.meshes() {
        log::info!(
            "Mesh #{}: {:?}, {} primitives",
            m.index(),
            m.name().unwrap_or_default(),
            m.primitives().len()
        );

        let mut mesh = Mesh::builder(m.name().unwrap_or_default());

        let mut indices = Vec::new();
        let mut vertices = Vec::new();

        for p in m.primitives() {
            let start_idx = vertices.len() as u32;
            let offset = indices.len();

            let reader = p.reader(|buffer| Some(&buffers[buffer.index()]));

            if let Some(read_indices) = reader.read_indices() {
                match read_indices {
                    ReadIndices::U8(iter) => {
                        for idx in iter {
                            indices.push(start_idx + idx as u32);
                        }
                    }
                    ReadIndices::U16(iter) => {
                        for idx in iter {
                            indices.push(start_idx + idx as u32);
                        }
                    }
                    ReadIndices::U32(iter) => {
                        for idx in iter {
                            indices.push(start_idx + idx);
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

            mesh = mesh.add_primitive(offset, indices.len() - offset, 0);
        }

        meshes.push(mesh.indices(indices).vertices(vertices).build());
    }

    meshes
}
