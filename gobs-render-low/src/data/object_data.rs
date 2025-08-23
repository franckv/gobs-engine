use glam::Mat3;
use serde::{Deserialize, Serialize};

use gobs_gfx::Buffer;

use crate::{
    GfxContext, RenderObject,
    data::{UniformLayout, UniformProp, UniformPropData, uniform::UniformData},
};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ObjectDataProp {
    WorldMatrix,
    NormalMatrix,
    VertexBufferAddress,
}

#[derive(Clone, Debug, Default)]
pub struct ObjectDataLayout {
    layout: Vec<ObjectDataProp>,
    uniform_layout: UniformLayout,
}

impl UniformData<ObjectDataProp, RenderObject> for ObjectDataLayout {
    fn prop(mut self, prop: ObjectDataProp) -> Self {
        self.layout.push(prop);

        match prop {
            ObjectDataProp::WorldMatrix => {
                self.uniform_layout = self.uniform_layout.prop("world_matrix", UniformProp::Mat4F);
            }
            ObjectDataProp::NormalMatrix => {
                self.uniform_layout = self
                    .uniform_layout
                    .prop("normal_matrix", UniformProp::Mat3F);
            }
            ObjectDataProp::VertexBufferAddress => {
                self.uniform_layout = self
                    .uniform_layout
                    .prop("buffer_reference", UniformProp::U64);
            }
        }

        self
    }

    fn uniform_layout(&self) -> &UniformLayout {
        &self.uniform_layout
    }

    fn copy_data(
        &self,
        ctx: Option<&GfxContext>,
        render_object: &RenderObject,
        buffer: &mut Vec<u8>,
    ) {
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
                        render_object.vertex_buffer.address(&ctx.unwrap().device)
                            + render_object.vertices_offset,
                    ));
                }
            }
        }

        layout.copy_data(&props, buffer);
    }

    fn is_empty(&self) -> bool {
        self.layout.is_empty()
    }
}
