use serde::{Deserialize, Serialize};

use crate::data::{
    AlignMode, Attribute, UniformLayout, align::AttributeData, uniform::UniformData,
};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ObjectDataProp {
    WorldMatrix,
    NormalMatrix,
    VertexBufferAddress,
}

#[derive(Clone, Debug)]
pub struct ObjectDataLayout {
    layout: Vec<ObjectDataProp>,
    uniform_layout: UniformLayout,
}

impl ObjectDataLayout {
    pub fn new(mode: AlignMode) -> Self {
        Self {
            layout: Vec::new(),
            uniform_layout: UniformLayout::new(mode),
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
            ObjectDataProp::NormalMatrix => {
                self.uniform_layout = self.uniform_layout.prop("normal_matrix", Attribute::Mat3F);
            }
            ObjectDataProp::VertexBufferAddress => {
                self.uniform_layout = self.uniform_layout.prop("buffer_reference", Attribute::U64);
            }
        }

        self
    }

    fn uniform_layout(&self) -> &UniformLayout {
        &self.uniform_layout
    }

    fn copy_data<F>(&self, buffer: &mut Vec<u8>, get_data: F)
    where
        F: Fn(&ObjectDataProp) -> AttributeData,
    {
        let layout = self.uniform_layout();

        let mut props = Vec::new();

        for prop in &self.layout {
            props.push(get_data(prop));
        }

        layout.copy_data(&props, buffer);
    }

    fn is_empty(&self) -> bool {
        self.layout.is_empty()
    }
}
