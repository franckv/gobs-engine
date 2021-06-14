use std::mem;

use ash::vk;

#[derive(Copy, Clone, PartialEq)]
pub enum VertexAttributeFormat {
    Vec2,
    Vec3,
    Vec4,
    Mat4
}

impl VertexAttributeFormat {
    pub fn locations(&self) -> usize {
        match self {
            VertexAttributeFormat::Vec2 => 1,
            VertexAttributeFormat::Vec3 => 1,
            VertexAttributeFormat::Vec4 => 1,
            VertexAttributeFormat::Mat4 => 4,
        }
    }

    pub fn location_size(&self) -> usize {
        mem::size_of::<[f32; 4]>()
    }
}

impl Into<vk::Format> for VertexAttributeFormat {
    fn into(self) -> vk::Format {
        match self {
            VertexAttributeFormat::Vec2 => vk::Format::R32G32_SFLOAT,
            VertexAttributeFormat::Vec3 => vk::Format::R32G32B32_SFLOAT,
            VertexAttributeFormat::Vec4 => vk::Format::R32G32B32A32_SFLOAT,
            VertexAttributeFormat::Mat4 => vk::Format::R32G32B32A32_SFLOAT,
        }
    }
}

pub struct VertexAttribute {
    pub location: usize,
    pub format: VertexAttributeFormat,
    pub offset: usize
}

#[derive(Copy, Clone)]
pub enum VertexLayoutBindingType {
    Vertex,
    Instance
}

impl Into<vk::VertexInputRate> for VertexLayoutBindingType {
    fn into(self) -> vk::VertexInputRate {
        match self {
            VertexLayoutBindingType::Vertex => vk::VertexInputRate::VERTEX,
            VertexLayoutBindingType::Instance => vk::VertexInputRate::INSTANCE,
        }
    }
}

pub struct VertexLayoutBinding {
    pub ty: VertexLayoutBindingType,
    pub binding: usize,
    pub attributes: Vec<VertexAttribute>,
    pub stride: usize
}

pub struct VertexLayoutBuilder {
    bindings: Vec<VertexLayoutBinding>,
    index: usize,
    location: usize,
}

impl VertexLayoutBuilder {
    pub fn new() -> Self {
        VertexLayoutBuilder {
            bindings: Vec::new(),
            index: 0,
            location: 0
        }
    }

    pub fn binding<T>(mut self, ty: VertexLayoutBindingType) -> Self {
        self.bindings.push(VertexLayoutBinding {
            ty,
            binding: self.index,
            attributes: Vec::new(),
            stride: mem::size_of::<T>()
        });

        self.index += 1;

        self
    }

    pub fn attribute(mut self, format: VertexAttributeFormat, offset: usize) -> Self {
        self.bindings.last_mut().unwrap().attributes.push(VertexAttribute {
            location: self.location,
            format,
            offset
        });

        self.location += format.locations();

        self
    }

    pub fn build(mut self) -> VertexLayout {
        let mut bindings = Vec::new();
        bindings.append(&mut self.bindings);

        VertexLayout {
            bindings
        }
    }
}

pub struct VertexLayout {
    pub bindings: Vec<VertexLayoutBinding>
}
