use std::{
    fmt::Debug,
    hash::{DefaultHasher, Hash, Hasher},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{DescriptorStage, DescriptorType, Handle};

pub type BindingId = Uuid;

#[derive(Clone)]
pub struct BindResource {
    pub id: BindingId,
    pub layout: BindingGroupLayout,
    pub resources: Vec<Handle>,
}

impl BindResource {
    pub fn new(layout: BindingGroupLayout, resources: Vec<Handle>) -> Self {
        Self {
            id: Uuid::new_v4(),
            layout,
            resources,
        }
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingGroupLayout {
    pub binding_group_id: u64,
    pub binding_group_type: BindingGroupType,
    pub bindings: Vec<(DescriptorType, DescriptorStage, u32)>,
}

impl BindingGroupLayout {
    pub fn new(binding_group_type: BindingGroupType) -> Self {
        let mut hasher = DefaultHasher::new();
        binding_group_type.hash(&mut hasher);
        let binding_group_id = hasher.finish();

        Self {
            binding_group_id,
            binding_group_type,
            bindings: Vec::new(),
        }
    }

    pub fn add_binding(mut self, ty: DescriptorType, stage: DescriptorStage, count: u32) -> Self {
        self.bindings.push((ty, stage, count));

        // generate content hash for layout
        let mut hasher = DefaultHasher::new();
        self.binding_group_type.hash(&mut hasher);
        self.bindings.hash(&mut hasher);
        self.binding_group_id = hasher.finish();

        self
    }
}
