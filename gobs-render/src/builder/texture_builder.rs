use gobs_core::{Color, ImageExtent2D, ImageFormat};
use gobs_resource::{ResourceHandle, ResourceLifetime, ResourceManager};

use crate::{Texture, TextureProperties, TextureType};

pub struct RenderTextureBuilder<'a> {
    name: &'a str,
    resource_manager: &'a mut ResourceManager,
    lifetime: ResourceLifetime,
    properties: Option<TextureProperties>,
}

impl<'a> RenderTextureBuilder<'a> {
    pub fn new(resource_manager: &'a mut ResourceManager, name: &'a str) -> Self {
        Self {
            name,
            resource_manager,
            lifetime: ResourceLifetime::Static,
            properties: None,
        }
    }

    pub fn transient(mut self, transient: bool) -> Self {
        if transient {
            self.lifetime = ResourceLifetime::Transient;
        } else {
            self.lifetime = ResourceLifetime::Static;
        }

        self
    }

    pub fn diffuse(mut self, filename: &str, format: ImageFormat) -> Self {
        let properties = TextureProperties::with_file(self.name, format, filename);

        self.properties = Some(properties);

        self
    }

    pub fn normal(mut self, filename: &str, format: ImageFormat) -> Self {
        let mut properties = TextureProperties::with_file(self.name, format, filename);
        properties.format.ty = TextureType::Normal;

        self.properties = Some(properties);

        self
    }

    pub fn diffuse_atlas(mut self, filename: &[&str], format: ImageFormat, cols: usize) -> Self {
        let properties = TextureProperties::with_atlas(self.name, format, filename, cols);

        self.properties = Some(properties);

        self
    }

    pub fn normal_atlas(mut self, filename: &[&str], format: ImageFormat, cols: usize) -> Self {
        let mut properties = TextureProperties::with_atlas(self.name, format, filename, cols);
        properties.format.ty = TextureType::Normal;

        self.properties = Some(properties);

        self
    }

    pub fn diffuse_colors(
        mut self,
        format: ImageFormat,
        colors: &[Color],
        extent: ImageExtent2D,
    ) -> Self {
        let properties = TextureProperties::with_colors(self.name, format, colors, extent);

        self.properties = Some(properties);

        self
    }

    pub fn build(self) -> ResourceHandle<Texture> {
        self.resource_manager
            .add(self.properties.unwrap(), ResourceLifetime::Static, false)
    }
}
