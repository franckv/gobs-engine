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
    BlendMode, ImageExtent2D, SamplerFilter,
};
use gobs_scene::graph::scenegraph::{NodeId, NodeValue, SceneGraph};

pub struct GLTFLoader {
    texture_manager: TextureManager,
    material_manager: MaterialManager,
    pub models: Vec<Arc<Model>>,
    pub scene: SceneGraph,
}

impl GLTFLoader {
    pub fn new(ctx: &Context, pass: Arc<dyn RenderPass>) -> Self {
        let texture_manager = TextureManager::new(ctx);
        let material_manager = MaterialManager::new(ctx, pass, &texture_manager);

        GLTFLoader {
            texture_manager,
            material_manager,
            models: vec![],
            scene: SceneGraph::new(),
        }
    }

    pub fn load<P>(&mut self, ctx: &Context, file: P)
    where
        P: AsRef<Path>,
    {
        let (doc, buffers, images) = gltf::import(file).unwrap();

        self.texture_manager.load(ctx, &doc, &images);
        self.material_manager.load(&doc, &self.texture_manager);

        self.load_models(&doc, &buffers);
        self.load_scene(&doc);
    }

    fn load_scene(&mut self, doc: &Document) {
        if let Some(scene) = doc.default_scene() {
            for node in scene.nodes() {
                self.add_node(self.scene.root, &node);
            }
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

    fn load_models(&mut self, doc: &Document, buffers: &[buffer::Data]) {
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
                    Some(mat_idx) => self.material_manager.instances[mat_idx].clone(),
                    None => self.material_manager.default_material_instance.clone(),
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

            self.models.push(model.build());
        }

        log::info!("{} models loaded", self.models.len());
    }
}

struct MaterialManager {
    pub instances: Vec<Arc<MaterialInstance>>,
    pub default_material_instance: Arc<MaterialInstance>,
    pub texture: Arc<Material>,
    pub transparent_texture: Arc<Material>,
    pub texture_normal: Arc<Material>,
    pub transparent_texture_normal: Arc<Material>,
    pub color_instance: Arc<MaterialInstance>,
    pub transparent_color_instance: Arc<MaterialInstance>,
}

impl MaterialManager {
    fn new(ctx: &Context, pass: Arc<dyn RenderPass>, texture_manager: &TextureManager) -> Self {
        let vertex_flags = VertexFlag::POSITION
            | VertexFlag::TEXTURE
            | VertexFlag::NORMAL
            | VertexFlag::TANGENT
            | VertexFlag::BITANGENT;

        let texture = Material::builder("gltf.texture.vert.spv", "gltf.texture.frag.spv")
            .vertex_flags(vertex_flags)
            .prop("diffuse", MaterialProperty::Texture)
            .build(ctx, pass.clone());

        let transparent_texture =
            Material::builder("gltf.texture.vert.spv", "gltf.texture.frag.spv")
                .vertex_flags(vertex_flags)
                .prop("diffuse", MaterialProperty::Texture)
                .blend_mode(BlendMode::Alpha)
                .build(ctx, pass.clone());

        let texture_normal = Material::builder("gltf.texture.vert.spv", "gltf.texture_n.frag.spv")
            .vertex_flags(vertex_flags)
            .prop("diffuse", MaterialProperty::Texture)
            .prop("normal", MaterialProperty::Texture)
            .build(ctx, pass.clone());

        let transparent_texture_normal =
            Material::builder("gltf.texture.vert.spv", "gltf.texture_n.frag.spv")
                .vertex_flags(vertex_flags)
                .prop("diffuse", MaterialProperty::Texture)
                .prop("normal", MaterialProperty::Texture)
                .blend_mode(BlendMode::Alpha)
                .build(ctx, pass.clone());

        let vertex_flags = VertexFlag::POSITION
            | VertexFlag::COLOR
            | VertexFlag::NORMAL
            | VertexFlag::TANGENT
            | VertexFlag::BITANGENT;

        let color = Material::builder("gltf.color_light.vert.spv", "gltf.color_light.frag.spv")
            .vertex_flags(vertex_flags)
            .build(ctx, pass.clone());

        let transparent_color =
            Material::builder("gltf.color_light.vert.spv", "gltf.color_light.frag.spv")
                .vertex_flags(vertex_flags)
                .blend_mode(BlendMode::Alpha)
                .build(ctx, pass.clone());

        let default_material_instance = texture
            .clone()
            .instantiate(vec![texture_manager.default_texture.clone()]);
        log::debug!("Default material id: {}", default_material_instance.id);

        let color_instance = color.instantiate(vec![]);
        log::debug!("Color material id: {}", color_instance.id);

        let transparent_color_instance = transparent_color.instantiate(vec![]);
        log::debug!("Color material id: {}", transparent_color_instance.id);

        MaterialManager {
            instances: vec![],
            default_material_instance,
            texture,
            transparent_texture,
            texture_normal,
            transparent_texture_normal,
            color_instance,
            transparent_color_instance,
        }
    }

    fn load(&mut self, doc: &Document, texture_manager: &TextureManager) {
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
                        let texture = texture_manager.textures[tex_info.texture().index()].clone();
                        match mat.normal_texture() {
                            Some(normal) => {
                                let normal_texture =
                                    texture_manager.textures[normal.texture().index()].clone();
                                self.add_texture_normal_instance(
                                    mat.alpha_mode(),
                                    texture,
                                    normal_texture,
                                )
                            }
                            None => self.add_texture_instance(mat.alpha_mode(), texture),
                        };
                    }
                    None => {
                        let color: Color = pbr.base_color_factor().into();
                        log::info!("Using color material: {:?}", color);
                        self.add_color_instance(mat.alpha_mode());
                    }
                }
            } else {
                log::info!("Using default material");
            }
        }

        log::info!("{} materials loaded", self.instances.len());
    }

    fn add_texture_instance(
        &mut self,
        alpha: AlphaMode,
        texture: Texture,
    ) -> Arc<MaterialInstance> {
        let material_instance = match alpha {
            AlphaMode::Blend => self.transparent_texture.instantiate(vec![texture]),
            _ => self.texture.instantiate(vec![texture]),
        };
        self.instances.push(material_instance.clone());

        material_instance
    }

    fn add_texture_normal_instance(
        &mut self,
        alpha: AlphaMode,
        diffuse: Texture,
        normal: Texture,
    ) -> Arc<MaterialInstance> {
        let material_instance = match alpha {
            AlphaMode::Blend => self
                .transparent_texture_normal
                .instantiate(vec![diffuse, normal]),
            _ => self.texture_normal.instantiate(vec![diffuse, normal]),
        };
        self.instances.push(material_instance.clone());

        material_instance
    }

    fn add_color_instance(&mut self, alpha: AlphaMode) -> Arc<MaterialInstance> {
        let material_instance = match alpha {
            AlphaMode::Blend => self.transparent_color_instance.clone(),
            _ => self.color_instance.clone(),
        };
        self.instances.push(material_instance.clone());

        material_instance
    }
}

struct TextureManager {
    pub textures: Vec<Texture>,
    pub default_texture: Texture,
}

impl TextureManager {
    fn new(ctx: &Context) -> Self {
        let default_texture = Texture::default(ctx);

        TextureManager {
            textures: vec![],
            default_texture,
        }
    }

    fn load(&mut self, ctx: &Context, doc: &Document, images: &[image::Data]) {
        log::info!("Reading {} images", images.len());

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
                if let Some(normal) = mat.normal_texture() {
                    if normal.texture().index() == t.index() {
                        ty = TextureType::Normal;
                    }
                }
            }

            let name = format!("Texture #{}: {}", t.index(), name);

            log::info!(
                "{}, image #{}, format: {:?}, type: {:?}",
                &name,
                image.index(),
                data.format,
                ty
            );

            match data.format {
                image::Format::R8G8B8A8 => {
                    let texture = Texture::new(
                        ctx,
                        &name,
                        &data.pixels,
                        ImageExtent2D::new(data.width, data.height),
                        ty,
                        ty.into(),
                        mag_filter,
                        min_filter,
                    );

                    self.add(texture);
                }
                image::Format::R8G8B8 => {
                    let mut pixels = vec![];
                    for (i, pixel) in data.pixels.iter().enumerate() {
                        pixels.push(*pixel);
                        if i % 3 == 2 {
                            pixels.push(255);
                        }
                    }

                    let texture = Texture::new(
                        ctx,
                        &name,
                        &pixels,
                        ImageExtent2D::new(data.width, data.height),
                        ty.into(),
                        ty.into(),
                        mag_filter,
                        min_filter,
                    );

                    self.add(texture);
                }
                _ => {
                    self.add_default();
                    log::warn!("Unsupported image format: {:?}", data.format)
                }
            };
        }

        log::info!("{} textures loaded", self.textures.len());
    }

    fn add(&mut self, texture: Texture) {
        self.textures.push(texture);
    }

    fn add_default(&mut self) {
        self.textures.push(self.default_texture.clone());
    }
}
