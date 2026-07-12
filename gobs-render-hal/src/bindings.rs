use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{DescriptorStage, DescriptorType, Handle};

#[derive(Clone, Eq, PartialEq)]
pub struct BindResource {
    pub layout: BindingGroupLayout,
    pub resources: Vec<Handle>,
}

impl BindResource {
    pub fn new(layout: BindingGroupLayout, resources: Vec<Handle>) -> Self {
        Self { layout, resources }
    }

    pub fn slot(&self, index: usize) -> Option<Handle> {
        self.resources.get(index).cloned()
    }
}

#[derive(Copy, Clone, Eq, Hash, Serialize, Deserialize, PartialEq)]
pub enum BindingGroupType {
    None,
    ComputeData,
    SceneData,
    MaterialData,
    MaterialTextures,
}

impl Debug for BindingGroupType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::ComputeData => write!(f, "ComputeData ({})", self.set()),
            Self::SceneData => write!(f, "SceneData ({}, push)", self.set()),
            Self::MaterialData => write!(f, "MaterialData ({})", self.set()),
            Self::MaterialTextures => write!(f, "MaterialTextures ({})", self.set()),
        }
    }
}

impl BindingGroupType {
    // TODO: should be in vulkan backend
    #[allow(clippy::match_like_matches_macro)]
    pub fn is_push(&self) -> bool {
        match self {
            BindingGroupType::SceneData => true,
            _ => false,
        }
    }

    // TODO: should be in vulkan backend
    pub fn set(&self) -> u32 {
        match self {
            BindingGroupType::None => panic!("Invalid binding group"),
            BindingGroupType::ComputeData => 0,
            BindingGroupType::SceneData => 0,
            BindingGroupType::MaterialData => 1,
            BindingGroupType::MaterialTextures => 2,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct BindingGroupLayout {
    pub binding_group_type: BindingGroupType,
    pub bindings: Vec<(DescriptorType, DescriptorStage)>,
}

impl BindingGroupLayout {
    pub fn new(binding_group_type: BindingGroupType) -> Self {
        Self {
            binding_group_type,
            bindings: Vec::new(),
        }
    }

    pub fn add_binding(mut self, ty: DescriptorType, stage: DescriptorStage) -> Self {
        self.bindings.push((ty, stage));

        self
    }
}
