use log::*;

use crate::resource;

pub struct Pipeline {
    pub pipeline: wgpu::RenderPipeline,
}

#[derive(Default)]
pub struct PipelineBuilder<'a> {
    name: Option<&'a str>,
    device: Option<&'a wgpu::Device>,
    bind_layouts: Vec<&'a wgpu::BindGroupLayout>,
    color_format: Option<wgpu::TextureFormat>,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: Vec<wgpu::VertexBufferLayout<'a>>,
    shader: Option<wgpu::ShaderModuleDescriptor<'a>>
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(device: &'a wgpu::Device, name: &'a str) -> Self {
        PipelineBuilder {
            name: Some(name),
            device: Some(device),
            bind_layouts: Vec::new(),
            ..Default::default()
        }
    }

    pub async fn shader(mut self, shader_path: &'a str) -> PipelineBuilder<'a> {
        let shader_txt = resource::load_string(shader_path).await.unwrap();

        let shader = wgpu::ShaderModuleDescriptor {
            label: Some(shader_path),
            source: wgpu::ShaderSource::Wgsl(shader_txt.into())
        };

        self.shader = Some(shader);

        self
    }

    pub fn bind_layout(mut self, bind_layouts: &[&'a wgpu::BindGroupLayout]) -> Self {
        self.bind_layouts.extend_from_slice(bind_layouts);

        self
    }

    pub fn vertex_layout(mut self, vertex_layout: wgpu::VertexBufferLayout<'a>) -> Self {
        self.vertex_layouts.push(vertex_layout);

        self
    }

    pub fn color_format(mut self, color_format: wgpu::TextureFormat) -> Self {
        self.color_format = Some(color_format);

        self
    }

    pub fn depth_format(mut self, depth_format: wgpu::TextureFormat) -> Self {
        self.depth_format = Some(depth_format);

        self
    }

    pub fn build(self) -> Pipeline {
        let device = self.device.unwrap();

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &self.bind_layouts.as_slice(),
            push_constant_ranges: &[]
        });

        let shader = device.create_shader_module(self.shader.unwrap());
    
        let vertex_state = wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: self.vertex_layouts.as_slice()
        };
    
        let fragment_state = wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: self.color_format.unwrap(),
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE
                }),
                write_mask: wgpu::ColorWrites::ALL
            })]
        };
    
        let primitive_state = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false
        };
    
        let depth_stencil = self.depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default()
        });
    
        let multisample_state = wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false
        };

        Pipeline {
            pipeline: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(self.name.unwrap()),
                layout: Some(&layout),
                vertex: vertex_state,
                fragment: Some(fragment_state),
                primitive: primitive_state,
                depth_stencil,
                multisample: multisample_state,
                multiview: None
            })
        }
    }
}