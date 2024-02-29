use std::{path::Path, sync::Arc};

use glam::Quat;
use gltf::{
    buffer, image,
    material::AlphaMode,
    mesh::util::{ReadColors, ReadIndices},
    Document,
};

use gobs_core::{Color, Transform};
use gobs_render::{
    context::Context,
    geometry::{Mesh, Model, VertexData, VertexFlag},
    material::{Material, MaterialInstance, MaterialProperty, Texture, TextureType},
    pass::RenderPass,
    ImageExtent2D, ImageFormat, SamplerFilter,
};
use gobs_scene::graph::scenegraph::{NodeValue, SceneGraph};

pub fn load_gltf<P>(ctx: &Context, file: P, pass: Arc<dyn RenderPass>) -> Vec<Arc<Model>>
where
    P: AsRef<Path>,
{
    let (doc, buffers, images) = gltf::import(file).unwrap();

    let textures = load_textures(ctx, &doc, &images);
    let default_texture = textures.last().unwrap().clone();

    let materials = load_materials(ctx, pass.clone(), &doc, &textures, default_texture);
    let default_material_instance = materials.last().unwrap().clone();

    load_models(&doc, &buffers, default_material_instance, &materials)
}

pub fn load_gltf_scene<P>(ctx: &Context, file: P, pass: Arc<dyn RenderPass>) -> SceneGraph
where
    P: AsRef<Path>,
{
    let (doc, buffers, images) = gltf::import(file).unwrap();

    let textures = load_textures(ctx, &doc, &images);
    let default_texture = textures.last().unwrap().clone();

    let materials = load_materials(ctx, pass.clone(), &doc, &textures, default_texture);
    let default_material_instance = materials.last().unwrap().clone();

    let models = load_models(&doc, &buffers, default_material_instance, &materials);

    load_scene(&doc, &models)
}

fn load_scene(doc: &Document, models: &[Arc<Model>]) -> SceneGraph {
    let mut graph = SceneGraph::new();

    let mut nodes = vec![];

    for node in doc.nodes() {
        let (translation, rotation, scale) = node.transform().decomposed();
        let transform =
            Transform::new(translation.into(), Quat::from_array(rotation), scale.into());

        let node = match node.mesh() {
            Some(mesh) => NodeValue::Model(models[mesh.index()].clone()),
            None => NodeValue::None,
        };

        nodes.push(Some(graph.add(node, transform)));
    }

    for node in doc.nodes() {
        if let Some(parent_key) = nodes[node.index()] {
            for child in node.children() {
                let child = nodes[child.index()].take();
                if let Some(child_key) = child {
                    graph.add_child(parent_key, child_key);
                }
            }
        }
    }

    if let Some(scene) = doc.default_scene() {
        for node in scene.nodes() {
            if let Some(node_key) = nodes[node.index()] {
                graph.add_child(graph.root, node_key);
            }
        }
    }

    graph
}

fn load_models(
    doc: &Document,
    buffers: &[buffer::Data],
    default_material_instance: Arc<MaterialInstance>,
    materials: &[Arc<MaterialInstance>],
) -> Vec<Arc<Model>> {
    let mut models = Vec::new();

    for m in doc.meshes() {
        let name = m.name().unwrap_or_default();
        log::info!(
            "Mesh #{}: {}, primitives: {}",
            m.index(),
            name,
            m.primitives().len(),
        );

        let mut model = Model::builder(name);

        for p in m.primitives() {
            log::info!(
                "Primitive #{}, material {:?}",
                p.index(),
                p.material().index()
            );
            let material = match p.material().index() {
                Some(mat_idx) => materials[mat_idx].clone(),
                None => default_material_instance.clone(),
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
                    ReadColors::RgbaF32(iter) => {
                        for (i, color) in iter.enumerate() {
                            mesh_data.vertices[i].color = color.into();
                        }
                    }
                    ReadColors::RgbaU16(iter) => {
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

            model = model.mesh(mesh_data.build(), material);
        }

        models.push(model.build());
    }

    log::info!("{} models loaded", models.len());

    models
}

fn load_materials(
    ctx: &Context,
    pass: Arc<dyn RenderPass>,
    doc: &Document,
    textures: &[Texture],
    default_texture: Texture,
) -> Vec<Arc<MaterialInstance>> {
    let mut materials = vec![];

    let vertex_flags = VertexFlag::POSITION
        | VertexFlag::TEXTURE
        | VertexFlag::NORMAL
        | VertexFlag::TANGENT
        | VertexFlag::BITANGENT;

    let texture_material = Material::builder("gltf.texture.vert.spv", "gltf.texture.frag.spv")
        .vertex_flags(vertex_flags)
        .prop("diffuse", MaterialProperty::Texture)
        .build(ctx, pass.clone());

    let texture_material_transparent =
        Material::builder("gltf.texture.vert.spv", "gltf.texture.frag.spv")
            .vertex_flags(vertex_flags)
            .prop("diffuse", MaterialProperty::Texture)
            .blending_enabled()
            .build(ctx, pass.clone());

    let vertex_flags = VertexFlag::POSITION | VertexFlag::COLOR;

    let color_material = Material::builder("gltf.color.vert.spv", "gltf.color.frag.spv")
        .vertex_flags(vertex_flags)
        .build(ctx, pass.clone());

    let color_material_transparent =
        Material::builder("gltf.color.vert.spv", "gltf.color.frag.spv")
            .vertex_flags(vertex_flags)
            .blending_enabled()
            .build(ctx, pass.clone());

    let default_material_instance = texture_material
        .clone()
        .instantiate(vec![default_texture.clone()]);
    log::debug!("Default material id: {}", default_material_instance.id);

    let color_material_instance = color_material.instantiate(vec![]);
    log::debug!("Color material id: {}", color_material_instance.id);

    let color_material_transparent_instance = color_material_transparent.instantiate(vec![]);
    log::debug!(
        "Color material id: {}",
        color_material_transparent_instance.id
    );

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
                    let material_instance = match mat.alpha_mode() {
                        AlphaMode::Blend => texture_material_transparent.instantiate(vec![texture]),
                        _ => texture_material.instantiate(vec![texture]),
                    };
                    materials.push(material_instance.clone());
                }
                None => {
                    let color: Color = pbr.base_color_factor().into();
                    log::info!("Using color material: {:?}", color);
                    let material_instance = match mat.alpha_mode() {
                        AlphaMode::Blend => color_material_transparent_instance.clone(),
                        _ => color_material_instance.clone(),
                    };
                    materials.push(material_instance);
                }
            }
        } else {
            log::info!("Using default material");
        }
    }

    materials.push(default_material_instance);

    log::info!("{} materials loaded", materials.len());

    materials
}

fn load_textures(ctx: &Context, doc: &Document, images: &[image::Data]) -> Vec<Texture> {
    log::info!("Reading {} images", images.len());

    let mut textures = vec![];

    let default_texture = Texture::default(ctx);

    for t in doc.textures() {
        let name = t.name().unwrap_or_default();
        let image = t.source();
        let data = &images[image.index()];

        let name = format!("Texture #{}: {}", t.index(), name);

        log::info!(
            "{}, image #{}, format: {:?}",
            &name,
            image.index(),
            data.format
        );

        match data.format {
            image::Format::R8G8B8A8 => {
                let texture = Texture::new(
                    ctx,
                    &name,
                    &data.pixels,
                    ImageExtent2D::new(data.width, data.height),
                    TextureType::Diffuse,
                    ImageFormat::R8g8b8a8Srgb,
                    SamplerFilter::FilterLinear,
                );

                textures.push(texture);
            }
            image::Format::R8G8B8 => {
                let mut pixels = vec![];
                for (i, pixel) in data.pixels.iter().enumerate() {
                    pixels.push(*pixel);
                    if i % 3 == 2 {
                        pixels.push(127);
                    }
                }

                let texture = Texture::new(
                    ctx,
                    &name,
                    &pixels,
                    ImageExtent2D::new(data.width, data.height),
                    TextureType::Diffuse,
                    ImageFormat::R8g8b8a8Srgb,
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

    textures.push(default_texture);

    log::info!("{} textures loaded", textures.len());

    textures
}
