use std::{fmt::Debug, path::Path, sync::Arc};

use glam::Quat;
use gltf::{
    Document, buffer, image,
    material::AlphaMode,
    mesh::util::{ReadColors, ReadIndices},
};

use gobs_core::{Color, ImageExtent2D, SamplerFilter, Transform, logger};
use gobs_render::{BlendMode, GfxContext, Model, TextureProperties, TextureType};
use gobs_resource::{
    geometry::{MeshGeometry, VertexData},
    manager::ResourceManager,
    resource::ResourceLifetime,
};
use gobs_scene::{
    components::{NodeId, NodeValue},
    graph::scenegraph::SceneGraph,
};

use crate::{AssetError, manager::MaterialManager};

pub struct GLTFLoader {
    material_manager: MaterialManager,
    pub models: Vec<Arc<Model>>,
    pub scene: SceneGraph,
    pub vertex_padding: bool,
}

impl GLTFLoader {
    pub fn new(
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
    ) -> Result<Self, AssetError> {
        let material_manager = MaterialManager::new(ctx, resource_manager)?;

        Ok(Self {
            material_manager,
            models: vec![],
            scene: SceneGraph::new(),
            vertex_padding: ctx.vertex_padding,
        })
    }

    pub fn load<P>(
        &mut self,
        resource_manager: &mut ResourceManager,
        file: P,
    ) -> Result<(), AssetError>
    where
        P: AsRef<Path> + Debug,
    {
        let (doc, buffers, images) = gltf::import(&file)?;

        self.load_material(resource_manager, &doc, &images);

        self.load_models(resource_manager, &doc, &buffers);
        self.load_scene(&doc);

        Ok(())
    }

    fn load_scene(&mut self, doc: &Document) {
        tracing::info!(target: logger::RESOURCES, "{} scenes found, default={}", doc.scenes().len(), doc.as_json().scene.unwrap());
        for scene in doc.scenes() {
            for node in scene.nodes() {
                self.add_node(self.scene.root, &node);
            }
            tracing::info!(target: logger::RESOURCES, "{} scene nodes loaded", self.scene.len());
        }
    }

    fn add_node(&mut self, parent: NodeId, node: &gltf::Node) {
        let (translation, rotation, scale) = node.transform().decomposed();
        let transform =
            Transform::new(translation.into(), Quat::from_array(rotation), scale.into());

        let node_value = match node.mesh() {
            Some(mesh) => NodeValue::Model(self.models[mesh.index()].clone()),
            None => NodeValue::None,
        };

        let node_key = self.scene.insert(parent, node_value, transform).unwrap();

        for child in node.children() {
            self.add_node(node_key, &child);
        }
    }

    fn load_models(
        &mut self,
        resource_manager: &mut ResourceManager,
        doc: &Document,
        buffers: &[buffer::Data],
    ) {
        for m in doc.meshes() {
            let name = m.name().unwrap_or_default();
            tracing::debug!(target: logger::RESOURCES,
                "Mesh #{}: {}, primitives: {}",
                m.index(),
                name,
                m.primitives().len(),
            );

            let mut model = Model::builder(name);

            for p in m.primitives() {
                tracing::debug!(target: logger::RESOURCES,
                    "Primitive #{}, material {:?}",
                    p.index(),
                    p.material().index()
                );
                let material = match p.material().index() {
                    Some(mat_idx) => self.material_manager.instances[mat_idx],
                    None => self.material_manager.default_material_instance,
                };

                let name = format!("{}.{}", &name, p.index());
                let mut mesh_data = MeshGeometry::builder(&name);

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
                                mesh_data = mesh_data.index(idx);
                            }
                        }
                    }
                }

                if let Some(iter) = reader.read_positions() {
                    for pos in iter {
                        mesh_data = mesh_data.vertex(
                            VertexData::builder()
                                .position(pos.into())
                                .padding(self.vertex_padding)
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
                        mesh_data.vertices[i].color = Color::WHITE;
                    }
                }

                model = model.mesh(
                    mesh_data.build(),
                    Some(material),
                    resource_manager,
                    ResourceLifetime::Static,
                );
            }

            self.models.push(model.build());
        }

        tracing::info!(target: logger::RESOURCES, "{} models loaded", self.models.len());
        tracing::info!(target: logger::RESOURCES, "{} meshes loaded", self.models.iter().map(|m| m.meshes.len()).sum::<usize>());
    }

    fn load_textures(
        &mut self,
        resource_manager: &mut ResourceManager,
        doc: &Document,
        images: &[image::Data],
    ) {
        tracing::trace!(target: logger::RESOURCES, "Reading {} images", images.len());

        for t in doc.textures() {
            let name = t.name().unwrap_or_default();
            let image = t.source();
            let data = &images[image.index()];
            let sampler = t.sampler();

            let mag_filter = match sampler.mag_filter() {
                Some(filter) => match filter {
                    gltf::texture::MagFilter::Nearest => SamplerFilter::FilterNearest,
                    gltf::texture::MagFilter::Linear => SamplerFilter::FilterLinear,
                },
                None => SamplerFilter::FilterLinear,
            };

            let min_filter = match sampler.min_filter() {
                Some(filter) => match filter {
                    gltf::texture::MinFilter::Nearest => SamplerFilter::FilterNearest,
                    gltf::texture::MinFilter::Linear => SamplerFilter::FilterLinear,
                    gltf::texture::MinFilter::NearestMipmapNearest => SamplerFilter::FilterNearest,
                    gltf::texture::MinFilter::LinearMipmapNearest => SamplerFilter::FilterNearest,
                    gltf::texture::MinFilter::NearestMipmapLinear => SamplerFilter::FilterLinear,
                    gltf::texture::MinFilter::LinearMipmapLinear => SamplerFilter::FilterLinear,
                },
                None => SamplerFilter::FilterLinear,
            };

            let mut ty = TextureType::Diffuse;
            for mat in doc.materials() {
                if let Some(normal) = mat.normal_texture()
                    && normal.texture().index() == t.index()
                {
                    ty = TextureType::Normal;
                }
            }

            let name = format!("Texture #{}: {}", t.index(), name);

            tracing::trace!(target: logger::RESOURCES,
                "{}, image #{}, format: {:?}, type: {:?}",
                &name,
                image.index(),
                data.format,
                ty
            );

            match data.format {
                image::Format::R8G8B8A8 => {
                    let mut properties = TextureProperties::with_data(
                        &name,
                        data.pixels.clone(),
                        ImageExtent2D::new(data.width, data.height),
                    );
                    properties.format.mag_filter = mag_filter;
                    properties.format.min_filter = min_filter;

                    let handle = resource_manager.add(properties, ResourceLifetime::Static);

                    self.material_manager.add_texture(handle);
                }
                image::Format::R8G8B8 => {
                    let mut pixels = vec![];
                    for (i, pixel) in data.pixels.iter().enumerate() {
                        pixels.push(*pixel);
                        if i % 3 == 2 {
                            pixels.push(255);
                        }
                    }

                    let mut properties = TextureProperties::with_data(
                        &name,
                        pixels,
                        ImageExtent2D::new(data.width, data.height),
                    );
                    properties.format.mag_filter = mag_filter;
                    properties.format.min_filter = min_filter;

                    let handle = resource_manager.add(properties, ResourceLifetime::Static);

                    self.material_manager.add_texture(handle);
                }
                image::Format::R8 => {
                    let mut pixels = vec![];
                    for pixel in &data.pixels {
                        pixels.push(*pixel);
                        pixels.push(*pixel);
                        pixels.push(*pixel);
                        pixels.push(255);
                    }

                    let mut properties = TextureProperties::with_data(
                        &name,
                        pixels,
                        ImageExtent2D::new(data.width, data.height),
                    );
                    properties.format.mag_filter = mag_filter;
                    properties.format.min_filter = min_filter;

                    let handle = resource_manager.add(properties, ResourceLifetime::Static);

                    self.material_manager.add_texture(handle);
                }
                _ => {
                    self.material_manager.add_default_texture();
                    tracing::warn!(target: logger::RESOURCES,"Unsupported image format: {:?}", data.format)
                }
            };
        }

        tracing::info!(target: logger::RESOURCES,
            "{} textures loaded",
            self.material_manager.texture_manager.textures.len()
        );
    }

    fn into_blend_mode(alpha: AlphaMode) -> BlendMode {
        match alpha {
            AlphaMode::Blend => BlendMode::Alpha,
            _ => BlendMode::None,
        }
    }

    fn load_material(
        &mut self,
        resource_manager: &mut ResourceManager,
        doc: &Document,
        images: &[image::Data],
    ) {
        self.load_textures(resource_manager, doc, images);

        for mat in doc.materials() {
            let name = mat.name().unwrap_or_default();
            if let Some(idx) = mat.index() {
                tracing::trace!(target: logger::RESOURCES, "Material #{}: {}", idx, name);

                let pbr = mat.pbr_metallic_roughness();
                let diffuse = pbr.base_color_texture();

                match diffuse {
                    Some(tex_info) => {
                        tracing::trace!(target: logger::RESOURCES,
                            "Using texture #{}: {}",
                            tex_info.texture().index(),
                            tex_info.texture().name().unwrap_or_default()
                        );
                        let texture = tex_info.texture().index();
                        match mat.normal_texture() {
                            Some(normal) => {
                                let normal_texture = normal.texture().index();
                                self.material_manager.add_texture_normal_instance(
                                    name,
                                    resource_manager,
                                    Self::into_blend_mode(mat.alpha_mode()),
                                    texture,
                                    normal_texture,
                                )
                            }
                            None => self.material_manager.add_texture_instance(
                                name,
                                resource_manager,
                                Self::into_blend_mode(mat.alpha_mode()),
                                texture,
                            ),
                        };
                    }
                    None => {
                        let color: Color = pbr.base_color_factor().into();
                        tracing::trace!(target: logger::RESOURCES, "Using color material: {:?}", color);
                        self.material_manager
                            .add_color_instance(Self::into_blend_mode(mat.alpha_mode()));
                    }
                }
            } else {
                tracing::trace!(target: logger::RESOURCES, "Using default material");
            }
        }

        tracing::info!(target: logger::RESOURCES, "{} materials loaded", self.material_manager.instances.len());
    }
}
