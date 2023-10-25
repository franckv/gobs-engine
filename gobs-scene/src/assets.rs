use std::io::{BufReader, Cursor};

use anyhow::Result;
use log::*;
use uuid::Uuid;

use gobs_utils as utils;
use gobs_wgpu as render;

use render::model::{Material, Mesh, MeshBuilder, Model, Texture};
use render::shader::{Shader, ShaderBindGroup, ShaderType};
use render::shader_data::VertexFlag;
use utils::load::{self, AssetType};

use crate::Gfx;

pub async fn load_model(file_name: &str, gfx: &Gfx, shader: &Shader, scale: f32) -> Result<Model> {
    let obj_text = load::load_string(file_name, AssetType::MODEL).await?;
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
            let mat_text = load::load_string(&p, AssetType::MODEL).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let materials = match shader.ty() {
        ShaderType::Phong => load_material(gfx, file_name, obj_materials?, shader).await?,
        ShaderType::Solid => Vec::new(),
    };

    let meshes = load_mesh(gfx, shader.ty(), models).await;

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

async fn load_mesh(gfx: &Gfx, shader_type: ShaderType, models: Vec<tobj::Model>) -> Vec<Mesh> {
    models
        .into_iter()
        .map(|m| {
            let flags = match shader_type {
                ShaderType::Phong => {
                    VertexFlag::POSITION | VertexFlag::TEXTURE | VertexFlag::NORMAL
                }
                ShaderType::Solid => VertexFlag::POSITION,
            };

            let mut mesh = MeshBuilder::new(&m.name, flags);

            for i in 0..m.mesh.positions.len() / 3 {
                let position = [
                    m.mesh.positions[i * 3],
                    m.mesh.positions[i * 3 + 1],
                    m.mesh.positions[i * 3 + 2],
                ];
                let texture = [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]];
                let normal = [
                    m.mesh.normals[i * 3],
                    m.mesh.normals[i * 3 + 1],
                    m.mesh.normals[i * 3 + 2],
                ];
                match shader_type {
                    ShaderType::Phong => {
                        mesh = mesh.add_vertex_PTN(position.into(), texture.into(), normal.into())
                    }
                    ShaderType::Solid => mesh = mesh.add_vertex_P(position.into()),
                }
            }
            mesh.material(m.mesh.material_id.unwrap_or(0))
                .add_indices(&m.mesh.indices)
                .build(gfx)
        })
        .collect::<Vec<_>>()
}

async fn load_material(
    gfx: &Gfx,
    name: &str,
    obj_materials: Vec<tobj::Material>,
    shader: &Shader,
) -> Result<Vec<Material>> {
    let mut materials = Vec::new();

    for m in obj_materials {
        info!("{}: Load material {}", name, m.name);

        let diffuse_texture = {
            if let Some(texture_name) = &m.diffuse_texture {
                Texture::load_texture(gfx, texture_name, false).await?
            } else {
                Texture::load_texture(gfx, "cube-diffuse.jpg", false).await?
            }
        };

        let normal_texture = {
            if let Some(texture_name) = &m.normal_texture {
                Texture::load_texture(gfx, texture_name, true).await?
            } else {
                Texture::load_texture(gfx, "cube-normal.png", true).await?
            }
        };

        materials.push(Material::new(
            m.name,
            gfx,
            shader.layout(ShaderBindGroup::Material),
            diffuse_texture,
            normal_texture,
        ));
    }

    Ok(materials)
}
