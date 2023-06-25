use std::env;
use std::io::{BufReader, Cursor};

use anyhow::Result;
use glam::{Vec2, Vec3};
use log::*;
use wgpu::util::DeviceExt;

use crate::model::{ Material, Mesh, Model, ModelVertex, Texture };

pub async fn load_string(file_name: &str)  -> Result<String> {
    let current_dir = env::current_dir()?;
    let path = current_dir.join("assets").join(file_name);

    info!("Loading string: {:?}", path);

    let txt = std::fs::read_to_string(path)?;

    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> Result<Vec<u8>> {
    let current_dir = env::current_dir()?;
    let path = current_dir.join("assets").join(file_name);

    info!("Loading bin: {:?}", path);

    let data = std::fs::read(path)?;

    Ok(data)
}

pub async fn load_texture(
    file_name: &str,
    is_normal_map: bool,
    device: &wgpu::Device,
    queue: &wgpu::Queue) -> Result<Texture> {
        let data = load_binary(file_name).await?;
        Texture::from_bytes(device, queue, &data, file_name, is_normal_map)
    }

pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue) -> Result<Model> {
        let obj_text = load_string(file_name).await?;
        let obj_cursor = Cursor::new(obj_text);
        let mut obj_reader = BufReader::new(obj_cursor);

        let (models, obj_materials) = tobj::load_obj_buf_async(
            &mut obj_reader,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
            |p| async move {
                let mat_text = load_string(&p).await.unwrap();
                tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
            }
        ).await?;

        let mut materials = Vec::new();
        for m in obj_materials? {
            info!("{}: Load material {}", file_name, m.name);

            let diffuse_texture = load_texture(&m.diffuse_texture, false, device, queue).await?;
            let normal_texture = load_texture(&m.normal_texture, true, device, queue).await?;

            materials.push(Material::new(m.name, device, diffuse_texture, normal_texture));
        }

        let meshes = models.into_iter().map(|m| {
            let mut vertices = (0..m.mesh.positions.len() / 3).map(|i| ModelVertex {
                position: [
                    m.mesh.positions[i * 3],
                    m.mesh.positions[i * 3 + 1],
                    m.mesh.positions[i * 3 + 2]
                ],
                tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                normal: [
                    m.mesh.normals[i * 3],
                    m.mesh.normals[i * 3 + 1],
                    m.mesh.normals[i * 3 + 2]
                ],
                tangent: [0.0; 3],
                bitangent: [0.0; 3],
            }).collect::<Vec<_>>();

            let indices = &m.mesh.indices;
            let mut triangles_included = vec![0; vertices.len()];

            for c in indices.chunks(3) {
                let v0 = vertices[c[0] as usize];
                let v1 = vertices[c[1] as usize];
                let v2 = vertices[c[2] as usize];

                let pos0: Vec3 = v0.position.into();
                let pos1: Vec3 = v1.position.into();
                let pos2: Vec3 = v2.position.into();

                let uv0: Vec2 = v0.tex_coords.into();
                let uv1: Vec2 = v1.tex_coords.into();
                let uv2: Vec2 = v2.tex_coords.into();

                let delta_pos1 = pos1 - pos0;
                let delta_pos2 = pos2 - pos0;
                let delta_uv1 = uv1 - uv0;
                let delta_uv2 = uv2 - uv0;

                let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

                vertices[c[0] as usize].tangent = (tangent + Vec3::from(vertices[c[0] as usize].tangent)).into();
                vertices[c[1] as usize].tangent = (tangent + Vec3::from(vertices[c[1] as usize].tangent)).into();
                vertices[c[2] as usize].tangent = (tangent + Vec3::from(vertices[c[2] as usize].tangent)).into();
                vertices[c[0] as usize].bitangent = (bitangent + Vec3::from(vertices[c[0] as usize].bitangent)).into();
                vertices[c[1] as usize].bitangent = (bitangent + Vec3::from(vertices[c[1] as usize].bitangent)).into();
                vertices[c[2] as usize].bitangent = (bitangent + Vec3::from(vertices[c[2] as usize].bitangent)).into();

                triangles_included[c[0] as usize] += 1;
                triangles_included[c[1] as usize] += 1;
                triangles_included[c[2] as usize] += 1;
            }

            for (i, n) in triangles_included.into_iter().enumerate() {
                let denom = 1.0 / n as f32;
                let mut v = &mut vertices[i];
                v.tangent = (Vec3::from(v.tangent) * denom).into();
                v.bitangent = (Vec3::from(v.bitangent) * denom).into();
            }

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX
            });

            info!("{}: Load mesh {} ({} vertices / {} indices)", file_name, m.name, vertices.len(), m.mesh.indices.len());

            Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0)
            }
        }).collect::<Vec<_>>();

        info!("{}: {} meshes / {} materials loaded", file_name, meshes.len(), materials.len());

        Ok(Model { meshes, materials })
    }
