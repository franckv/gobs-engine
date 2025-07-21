use std::sync::Arc;

use glam::Mat3;
use gobs_gfx::Buffer;

use crate::{
    GfxContext, RenderObject,
    data::{UniformLayout, UniformProp, UniformPropData},
};

#[derive(Clone, Debug)]
pub enum ObjectDataProp {
    WorldMatrix,
    NormalMatrix,
    VertexBufferAddress,
}

#[derive(Clone, Debug)]
pub struct ObjectDataLayout {
    layout: Vec<ObjectDataProp>,
    uniform_layout: Arc<UniformLayout>,
}

impl ObjectDataLayout {
    pub fn builder() -> ObjectDataLayoutBuilder {
        ObjectDataLayoutBuilder::new()
    }

    pub fn copy_data(&self, ctx: &GfxContext, render_object: &RenderObject, buffer: &mut Vec<u8>) {
        let layout = self.uniform_layout();

        let mut props = Vec::new();

        for prop in &self.layout {
            match prop {
                ObjectDataProp::WorldMatrix => {
                    props.push(UniformPropData::Mat4F(
                        render_object.transform.matrix().to_cols_array_2d(),
                    ));
                }
                ObjectDataProp::NormalMatrix => {
                    props.push(UniformPropData::Mat3F(
                        Mat3::from_quat(render_object.transform.rotation()).to_cols_array_2d(),
                    ));
                }
                ObjectDataProp::VertexBufferAddress => {
                    props.push(UniformPropData::U64(
                        render_object.vertex_buffer.address(&ctx.device)
                            + render_object.vertices_offset,
                    ));
                }
            }
        }

        layout.copy_data(&props, buffer);
    }

    pub fn uniform_layout(&self) -> Arc<UniformLayout> {
        self.uniform_layout.clone()
    }
}

pub struct ObjectDataLayoutBuilder {
    layout: Vec<ObjectDataProp>,
}

impl ObjectDataLayoutBuilder {
    pub fn new() -> Self {
        Self {
            layout: Default::default(),
        }
    }

    pub fn prop(mut self, prop: ObjectDataProp) -> Self {
        self.layout.push(prop);

        self
    }

    pub fn build(self) -> ObjectDataLayout {
        let mut layout = UniformLayout::builder();

        for prop in &self.layout {
            match prop {
                ObjectDataProp::WorldMatrix => {
                    layout = layout.prop("world_matrix", UniformProp::Mat4F);
                }
                ObjectDataProp::NormalMatrix => {
                    layout = layout.prop("normal_matrix", UniformProp::Mat3F);
                }
                ObjectDataProp::VertexBufferAddress => {
                    layout = layout.prop("buffer_reference", UniformProp::U64);
                }
            }
        }

        let uniform_layout = layout.build();

        ObjectDataLayout {
            layout: self.layout,
            uniform_layout,
        }
    }
}
