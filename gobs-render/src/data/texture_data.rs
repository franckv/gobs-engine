use serde::{Deserialize, Serialize};

// TODO: Emissive, Specular, Opacity, Glossiness, ...
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum TextureDataProp {
    Diffuse,
    Normal,
    Emission,
    Specular,
}

#[derive(Clone, Debug, Default)]
pub struct TextureDataLayout {
    pub(crate) layout: Vec<TextureDataProp>,
    pub(crate) texture_indexing: bool,
}

impl TextureDataLayout {
    pub fn is_empty(&self) -> bool {
        self.layout.is_empty()
    }

    pub fn prop(mut self, prop: TextureDataProp) -> Self {
        self.layout.push(prop);

        self
    }
}
