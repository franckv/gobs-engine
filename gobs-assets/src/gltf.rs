use std::{
    collections::{hash_map::Entry, HashMap},
    path::Path,
    sync::Arc,
};

use gltf::mesh::util::{ReadColors, ReadIndices};

use gobs_core::Color;
use gobs_render::{
    context::Context,
    geometry::{Mesh, Model, VertexData, VertexFlag},
    material::{Material, MaterialInstanceId, MaterialProperty, Texture, TextureType},
    ImageExtent2D, SamplerFilter,
};

pub fn load_gltf<P>(ctx: &Context, file: P) -> Vec<Arc<Model>>
where
    P: AsRef<Path>,
{
    let (doc, buffers, images) = gltf::import(file).unwrap();

    let mut models = Vec::new();

    let vertex_flags = VertexFlag::POSITION
        | VertexFlag::TEXTURE
        | VertexFlag::NORMAL
        | VertexFlag::TANGENT
        | VertexFlag::BITANGENT;

    let texture_material = Material::builder("gltf.texture.vert.spv", "gltf.texture.frag.spv")
        .vertex_flags(vertex_flags)
        .prop("diffuse", MaterialProperty::Texture)
        .build(ctx);

    let vertex_flags = VertexFlag::POSITION | VertexFlag::COLOR;

    let color_material = Material::builder("gltf.color.vert.spv", "gltf.color.frag.spv")
        .vertex_flags(vertex_flags)
        .build(ctx);

    let default_texture = Texture::default(ctx);
    let default_material_instance = texture_material
        .clone()
        .instantiate(vec![default_texture.clone()]);
    log::debug!("Default material id: {}", default_material_instance.id);

    let color_material_instance = color_material.instantiate(vec![]);
    log::debug!("Color material id: {}", color_material_instance.id);

    let mut materials = vec![];
    let mut textures = vec![];

    log::info!("Reading {} images", images.len());

    for t in doc.textures() {
        let name = t.name().unwrap_or_default();
        let image = t.source();
        let data = &images[image.index()];

        log::info!(
            "Texture #{}: {}, image #{}, format: {:?}",
            t.index(),
            name,
            image.index(),
            data.format
        );

        match data.format {
            gltf::image::Format::R8G8B8A8 => {
                let texture = Texture::new(
                    ctx,
                    name,
                    &data.pixels,
                    ImageExtent2D::new(data.width, data.height),
                    TextureType::Diffuse,
                    SamplerFilter::FilterLinear,
                );

                textures.push(texture);
            }
            _ => {
                textures.push(default_texture.clone());
                log::warn!("Unsupported image format: {:?}", data.format)
            }
        };
    }

    for mat in doc.materials() {
        let name = mat.name().unwrap_or_default();
        if let Some(idx) = mat.index() {
            log::info!("Material #{}: {}", idx, name);

            let pbr = mat.pbr_metallic_roughness();
            let diffuse = pbr.base_color_texture();

            match diffuse {
                Some(tex_info) => {
                    log::info!(
                        "Using texture #{}: {}",
                        tex_info.texture().index(),
                        tex_info.texture().name().unwrap_or_default()
                    );
                    let texture = textures[tex_info.texture().index()].clone();
                    let material_instance = texture_material.clone().instantiate(vec![texture]);
                    materials.push(material_instance.clone());
                }
                None => {
                    let color: Color = pbr.base_color_factor().into();
                    log::info!("Using color material: {:?}", color);
                    materials.push(color_material_instance.clone());
                }
            }
        } else {
            log::info!("Using default material");
        }
    }

    for m in doc.meshes() {
        let name = m.name().unwrap_or_default();
        log::info!(
            "Mesh #{}: {}, primitives: {}",
            m.index(),
            name,
            m.primitives().len(),
        );

        let mut model = Model::builder(name);
        let mut used_materials = Vec::new();
        let mut material_map: HashMap<MaterialInstanceId, usize> = HashMap::new();

        for p in m.primitives() {
            let material = match p.material().index() {
                Some(mat_idx) => materials[mat_idx].clone(),
                None => default_material_instance.clone(),
            };

            let material_idx = match material_map.entry(material.id) {
                Entry::Occupied(entry) => *entry.get(),
                Entry::Vacant(entry) => {
                    let idx = used_materials.len();
                    used_materials.push(material.clone());
                    *entry.insert(idx)
                }
            };

            let mut mesh_data = Mesh::builder(name);

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
                            mesh_data.vertices[i].color = color.into();
                        }
                    }
                    _ => todo!(),
                }
            } else {
                for i in 0..mesh_data.vertices.len() {
                    mesh_data.vertices[i].color = mesh_data.vertices[i].normal.into();
                }
            }

            model = model.mesh(mesh_data.build(), material_idx);
        }

        model = model.materials(&mut used_materials);

        models.push(model.build());
    }

    models
}
