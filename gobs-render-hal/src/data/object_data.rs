use serde::{Deserialize, Serialize};

use crate::data::{AlignMode, Attribute, UniformLayout, uniform::UniformData};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ObjectDataProp {
    WorldMatrix,
    VertexBufferAddress,
    InstanceBufferAddress,
    MaterialOffset,
}

#[derive(Clone, Debug)]
pub struct ObjectDataLayout {
    layout: Vec<ObjectDataProp>,
    uniform_layout: UniformLayout,
    pub instancing: bool,
}

impl ObjectDataLayout {
    pub fn new(instancing: bool) -> Self {
        Self {
            layout: Vec::new(),
            uniform_layout: UniformLayout::new(AlignMode::Std430),
            instancing,
        }
    }
}

impl UniformData<ObjectDataProp> for ObjectDataLayout {
    fn prop(mut self, prop: ObjectDataProp) -> Self {
        self.layout.push(prop);

        match prop {
            ObjectDataProp::WorldMatrix => {
                self.uniform_layout = self.uniform_layout.prop("world_matrix", Attribute::Mat4F);
            }
            ObjectDataProp::VertexBufferAddress => {
                self.uniform_layout = self.uniform_layout.prop("buffer_reference", Attribute::U64);
            }
            ObjectDataProp::InstanceBufferAddress => {
                self.uniform_layout = self.uniform_layout.prop("buffer_reference", Attribute::U64);
            }
            ObjectDataProp::MaterialOffset => {
                self.uniform_layout = self.uniform_layout.prop("material_offset", Attribute::U32);
            }
        }

        self
    }

    fn layout(&self) -> &[ObjectDataProp] {
        &self.layout
    }

    fn uniform_layout(&self) -> &UniformLayout {
        &self.uniform_layout
    }
}
