use std::env;
use std::io::{BufReader, Cursor, Error, ErrorKind};
use std::path::PathBuf;

use anyhow::Result;
use log::*;
use uuid::Uuid;

use crate::model::{Material, Mesh, Model, Texture};
use crate::render::Gfx;
use crate::shader::ShaderType;
use crate::shader_data::{VertexData, VertexP, VertexPTN};

pub enum AssetType {
    SHADER,
    IMAGE,
    MODEL,
}

pub fn get_asset_dir(file_name: &str, ty: AssetType) -> Result<PathBuf> {
    let current_exe = env::current_exe()?;
    let current_dir = current_exe
        .parent()
        .ok_or(Error::from(ErrorKind::NotFound))?;
    let path = match ty {
        AssetType::SHADER => current_dir.join("shaders"),
        AssetType::MODEL => current_dir.join("assets"),
        AssetType::IMAGE => current_dir.join("assets"),
    };

    Ok(path.join(file_name))
}

pub async fn load_string(file_name: &str, ty: AssetType) -> Result<String> {
    let path = get_asset_dir(file_name, ty)?;

    info!("Loading string: {:?}", path);

    let txt = std::fs::read_to_string(path)?;

    Ok(txt)
}

pub async fn load_binary(file_name: &str, ty: AssetType) -> Result<Vec<u8>> {
    let path = get_asset_dir(file_name, ty)?;

    info!("Loading bin: {:?}", path);

    let data = std::fs::read(path)?;

    Ok(data)
}

pub async fn load_texture(file_name: &str, is_normal_map: bool, gfx: &Gfx) -> Result<Texture> {
    let data = load_binary(file_name, AssetType::IMAGE).await?;
    Texture::from_bytes(gfx, &data, file_name, is_normal_map)
}

pub async fn load_model(
    file_name: &str,
    gfx: &Gfx,
    shader_type: ShaderType,
    layout: &wgpu::BindGroupLayout,
    scale: f32,
) -> Result<Model> {
    let obj_text = load_string(file_name, AssetType::MODEL).await?;
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
            let mat_text = load_string(&p, AssetType::MODEL).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let materials = load_material(gfx, file_name, obj_materials?, layout).await?;

    let meshes = load_mesh(gfx, file_name, shader_type, models).await;

    info!(
        "{}: {} meshes / {} materials loaded",
        file_name,
        meshes.len(),
        materials.len()
    );

    Ok(Model {
        id: Uuid::new_v4(),
        scale,
        meshes,
        materials,
    })
}

async fn load_mesh(
    gfx: &Gfx,
    name: &str,
    shader_type: ShaderType,
    models: Vec<tobj::Model>,
) -> Vec<Mesh> {
    models
        .into_iter()
        .map(|m| {
            let mut vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| match shader_type {
                    ShaderType::Phong => VertexData::VertexPTN(VertexPTN {
                        position: [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2],
                        ],
                        tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                        normal: [
                            m.mesh.normals[i * 3],
                            m.mesh.normals[i * 3 + 1],
                            m.mesh.normals[i * 3 + 2],
                        ],
                        tangent: [0.0; 3],
                        bitangent: [0.0; 3],
                    }),
                    ShaderType::Solid => VertexData::VertexP(VertexP {
                        position: [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2],
                        ],
                    }),
                })
                .collect::<Vec<_>>();

            info!(
                "{}: Load mesh {} ({} vertices / {} indices)",
                name,
                m.name,
                vertices.len(),
                m.mesh.indices.len()
            );

            Mesh::new(
                gfx,
                name,
                &mut vertices,
                &m.mesh.indices,
                m.mesh.material_id.unwrap_or(0),
                true,
            )
        })
        .collect::<Vec<_>>()
}

async fn load_material(
    gfx: &Gfx,
    name: &str,
    obj_materials: Vec<tobj::Material>,
    layout: &wgpu::BindGroupLayout,
) -> Result<Vec<Material>> {
    let mut materials = Vec::new();

    for m in obj_materials {
        info!("{}: Load material {}", name, m.name);

        let diffuse_texture = {
            if let Some(texture_name) = &m.diffuse_texture {
                load_texture(texture_name, false, gfx).await?
            } else {
                load_texture("cube-diffuse.jpg", false, gfx).await?
            }
        };

        let normal_texture = {
            if let Some(texture_name) = &m.normal_texture {
                load_texture(texture_name, true, gfx).await?
            } else {
                load_texture("cube-normal.png", true, gfx).await?
            }
        };

        materials.push(Material::new(
            m.name,
            gfx,
            layout,
            diffuse_texture,
            normal_texture,
        ));
    }

    Ok(materials)
}
