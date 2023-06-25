use std::collections::HashMap;
use itertools::Itertools;

use log::*;
use naga::front::wgsl::Frontend;
use naga::Module;
use naga::GlobalVariable;
use naga::Handle;
use naga::Type;
use naga::TypeInner;

use crate::resource;

pub struct Generator {
    module: Module
}

impl Generator {
    pub async fn new(shader_path: &str) -> Self {
        let shader = resource::load_string(shader_path).await.unwrap();

        let mut front = Frontend::new();
        let module = front.parse(&shader).unwrap();

        Generator {
            module
        }
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
                    min_binding_size: None
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
            TypeInner::Sampler {
                comparison: _
            } => wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            _ => {
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                }
            }
        }
    }

    pub fn bind_layouts(&self, device: &wgpu::Device) -> Vec<wgpu::BindGroupLayout> {
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

        groups.keys().sorted().map(|i| {
            let entries = groups.get(i).unwrap().iter().map(|var| {
                self.bind_layout_entry(var)
            }).collect::<Vec<_>>();

            let label = format!("Bind group {}", i);

            let layout = wgpu::BindGroupLayoutDescriptor {
                entries: &(entries.as_slice()),
                label: Some(&label)
            };

            error!("{:?}", layout);

            device.create_bind_group_layout(&layout)
        }).collect::<Vec<_>>()
    }

    fn bind_layout_entry(&self, var: &GlobalVariable) -> wgpu::BindGroupLayoutEntry {
        let GlobalVariable { name: _, space: _, binding, ty, init: _ } = var;

        let ty = self.lookup_type(ty);

        wgpu::BindGroupLayoutEntry {
            binding: binding.clone().unwrap().binding,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty,
            count: None,
        }
    }
}