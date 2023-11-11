use itertools::Itertools;
use std::collections::HashMap;

use log::*;
use naga::front::wgsl::Frontend;
use naga::{Binding, GlobalVariable, Handle, Module, Type, TypeInner};

use gobs_utils as utils;

use crate::context::Gfx;
use crate::model::{InstanceData, InstanceFlag, VertexData, VertexFlag};
use utils::load::{self, AssetType};

pub struct Generator {
    module: Module,
}

impl Generator {
    pub async fn new(shader_path: &str) -> Self {
        let shader = load::load_string(shader_path, AssetType::SHADER)
            .await
            .unwrap();

        let mut front = Frontend::new();
        let module = front.parse(&shader).unwrap();

        Generator { module }
    }

    fn lookup_type_format(&self, ty: &Handle<Type>) -> Option<(wgpu::VertexFormat, u64)> {
        let ty = self.module.types.get_handle(*ty).unwrap();

        if let Type {
            inner: TypeInner::Vector { size, kind, width },
            ..
        } = ty
        {
            match kind {
                naga::ScalarKind::Float => match size {
                    naga::VectorSize::Bi => {
                        if *width == 4 {
                            return Some((wgpu::VertexFormat::Float32x2, 8));
                        }
                    }
                    naga::VectorSize::Tri => {
                        if *width == 4 {
                            return Some((wgpu::VertexFormat::Float32x3, 12));
                        }
                    }
                    naga::VectorSize::Quad => {
                        if *width == 4 {
                            return Some((wgpu::VertexFormat::Float32x4, 16));
                        }
                    }
                },
                _ => (),
            }
        } else if let Type {
            inner:
                TypeInner::Scalar {
                    kind: naga::ScalarKind::Float,
                    width,
                },
            ..
        } = ty
        {
            if *width == 4 {
                return Some((wgpu::VertexFormat::Float32, 4));
            }
        }

        None
    }

    fn lookup_type(&self, ty: &Handle<Type>) -> wgpu::BindingType {
        let ty = self.module.types.get_handle(*ty).unwrap();

        match &ty.inner {
            TypeInner::Struct {
                members: _,
                span: _,
            } => wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            TypeInner::Image {
                dim: _,
                arrayed: _,
                class: _,
            } => wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            TypeInner::Sampler { comparison: _ } => {
                wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
            }
            _ => wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        }
    }

    pub fn vertex_layout<'a>(
        &'a self,
        attributes: &'a Vec<wgpu::VertexAttribute>,
        instance: bool,
        instance_flags: InstanceFlag,
        vertex_flags: VertexFlag,
    ) -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: if instance {
                InstanceData::size(instance_flags) as wgpu::BufferAddress
            } else {
                VertexData::size(vertex_flags) as wgpu::BufferAddress
            },
            step_mode: if instance {
                wgpu::VertexStepMode::Instance
            } else {
                wgpu::VertexStepMode::Vertex
            },
            attributes: &attributes.as_slice(),
        }
    }

    pub fn vertex_layout_attributes(&self, name: &str) -> Vec<wgpu::VertexAttribute> {
        let mut attributes: Vec<wgpu::VertexAttribute> = Vec::new();

        let mut offset: u64 = 0;
        for ty in self.module.types.iter() {
            if let Some(n) = &ty.1.name {
                if name == n {
                    info!("found {}", n);
                    if let TypeInner::Struct { members, .. } = &ty.1.inner {
                        for member in members {
                            if let Some(Binding::Location { location, .. }) = member.binding {
                                let format = self.lookup_type_format(&member.ty).unwrap();
                                attributes.push(wgpu::VertexAttribute {
                                    format: format.0,
                                    offset: offset as u64,
                                    shader_location: location,
                                });
                                offset += format.1;
                            }
                        }
                    }
                }
            }
        }

        attributes
    }

    /// Define layout to bind resources (e.g. uniform buffers) to shader
    /// Used in shader as follow:
    ///     @group(0) @binding(0)
    ///     var<uniform> camera: Camera;
    /// Return a list of binding groups (@group)
    pub fn bind_layouts(&self, gfx: &Gfx) -> Vec<wgpu::BindGroupLayout> {
        info!("Generate bind group layouts");

        let mut groups: HashMap<u32, Vec<&GlobalVariable>> = HashMap::new();

        for var in self.module.global_variables.iter() {
            if let Some(binding) = &var.1.binding {
                let gid = binding.clone().group;

                if !groups.contains_key(&gid) {
                    groups.insert(gid, Vec::new());
                }

                groups.get_mut(&gid).unwrap().push(&var.1);
            }
        }

        groups
            .keys()
            .sorted()
            .map(|i| {
                let entries = groups
                    .get(i)
                    .unwrap()
                    .iter()
                    .map(|var| self.bind_layout_entry(var))
                    .collect::<Vec<_>>();

                let label = format!("Bind group {}", i);

                let layout = wgpu::BindGroupLayoutDescriptor {
                    entries: &(entries.as_slice()),
                    label: Some(&label),
                };

                info!("[{}] {}", i, label);
                gfx.create_bind_group_layout(&layout)
            })
            .collect::<Vec<_>>()
    }

    fn bind_layout_entry(&self, var: &GlobalVariable) -> wgpu::BindGroupLayoutEntry {
        let GlobalVariable {
            name: _,
            space: _,
            binding,
            ty,
            init: _,
        } = var;

        let ty = self.lookup_type(ty);

        wgpu::BindGroupLayoutEntry {
            binding: binding.clone().unwrap().binding,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty,
            count: None,
        }
    }
}
