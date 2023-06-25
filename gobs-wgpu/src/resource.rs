use std::env;
use std::io::{BufReader, Cursor};

use anyhow::Result;
use log::*;

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
                tex_coords: [
                    m.mesh.texcoords[i * 2], 
                    m.mesh.texcoords[i * 2 + 1]
                ],
                normal: [
                    m.mesh.normals[i * 3],
                    m.mesh.normals[i * 3 + 1],
                    m.mesh.normals[i * 3 + 2]
                ],
                tangent: [0.0; 3],
                bitangent: [0.0; 3],
            }).collect::<Vec<_>>();

            info!("{}: Load mesh {} ({} vertices / {} indices)", file_name, m.name, vertices.len(), m.mesh.indices.len());

            Mesh::new(device, file_name, &mut vertices, &m.mesh.indices, m.mesh.material_id.unwrap_or(0), true)
        }).collect::<Vec<_>>();

        info!("{}: {} meshes / {} materials loaded", file_name, meshes.len(), materials.len());

        Ok(Model { meshes, materials })
    }