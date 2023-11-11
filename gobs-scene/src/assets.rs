use std::io::{BufReader, Cursor};
use std::sync::Arc;

use anyhow::Result;
use log::*;

use gobs_render as render;
use gobs_utils as utils;

use render::model::{Material, Mesh, MeshBuilder, Model, ModelBuilder, Texture};
use render::shader::Shader;
use utils::load::{self, AssetType};

use crate::Gfx;

pub async fn load_model(file_name: &str, gfx: &Gfx, shader: Arc<Shader>) -> Result<Arc<Model>> {
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

    let materials = load_material(gfx, file_name, obj_materials?).await?;

    let meshes = load_mesh(models, &materials).await;

    info!(
        "{}: {} meshes / {} materials loaded",
        file_name,
        meshes.len(),
        materials.len()
    );

    let model = ModelBuilder::new().meshes(meshes).build(gfx, shader);

    Ok(model)
}

async fn load_mesh(
    models: Vec<tobj::Model>,
    materials: &Vec<Arc<Material>>,
) -> Vec<(Arc<Mesh>, Option<Arc<Material>>)> {
    models
        .into_iter()
        .map(|m| {
            let mut mesh = MeshBuilder::new(&m.name);

            for i in 0..m.mesh.positions.len() / 3 {
                let position = [
                    m.mesh.positions[i * 3],
                    m.mesh.positions[i * 3 + 1],
                    m.mesh.positions[i * 3 + 2],
                ];
                let color = [
                    *(m.mesh.vertex_color.get(i * 3).unwrap_or(&1.)),
                    *(m.mesh.vertex_color.get(i * 3 + 1).unwrap_or(&1.)),
                    *(m.mesh.vertex_color.get(i * 3 + 2).unwrap_or(&1.)),
                    1.,
                ];
                let texture = [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]];
                let normal = [
                    m.mesh.normals[i * 3],
                    m.mesh.normals[i * 3 + 1],
                    m.mesh.normals[i * 3 + 2],
                ];
                mesh = mesh.add_vertex(
                    position.into(),
                    color.into(),
                    texture.into(),
                    normal.into(),
                    texture.into(),
                )
            }

            let material_id = m.mesh.material_id.unwrap_or(0);
            let material = if materials.len() > material_id {
                Some(materials[material_id].clone())
            } else {
                None
            };

            (mesh.add_indices(&m.mesh.indices).build(), material)
        })
        .collect::<Vec<_>>()
}

async fn load_material(
    gfx: &Gfx,
    name: &str,
    obj_materials: Vec<tobj::Material>,
) -> Result<Vec<Arc<Material>>> {
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

        materials.push(Material::new(m.name, diffuse_texture, normal_texture));
    }

    Ok(materials)
}
