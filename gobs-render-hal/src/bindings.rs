use std::{
    fmt::Debug,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{DescriptorStage, DescriptorType, Handle};

pub type BindingId = Uuid;

#[derive(Clone, Copy, Debug)]
pub enum BindingLifetime {
    PerFrame,
    #[allow(unused)]
    Static,
}

#[derive(Clone, Debug)]
pub struct BindSet {
    bindings: Vec<(Handle, usize)>,
}

impl BindSet {
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    pub fn binding(mut self, handle: Handle, index: usize) -> Self {
        self.bindings.push((handle, index));

        self
    }

    pub fn get(&self, idx: usize) -> Option<Handle> {
        self.bindings
            .iter()
            .find(|(_, index)| *index == idx)
            .map(|(handle, _)| *handle)
    }

    pub fn bindings(&self) -> impl Iterator<Item = &(Handle, usize)> {
        self.bindings.iter()
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }
}

#[derive(Clone, Debug)]
pub struct BindResource {
    pub id: BindingId,
    layout: Arc<BindingGroupLayout>,
    binding_sets: Vec<BindSet>,
}

impl BindResource {
    pub fn with_resources(layout: Arc<BindingGroupLayout>, resources: Vec<Handle>) -> Self {
        let binding_sets = resources
            .into_iter()
            .map(|handle| BindSet::new().binding(handle, 0))
            .collect();

        Self {
            id: Uuid::new_v4(),
            layout,
            binding_sets,
        }
    }

    pub fn new(layout: Arc<BindingGroupLayout>) -> Self {
        Self {
            id: Uuid::new_v4(),
            layout,
            binding_sets: Vec::new(),
        }
    }

    pub fn binding(mut self, resource: Handle, index: usize) -> Self {
        if self.binding_sets.is_empty() {
            self.binding_sets.push(BindSet::new());
        }

        let bindset = self.binding_sets.pop().unwrap();
        self.binding_sets.push(bindset.binding(resource, index));

        self
    }

    pub fn add_binding(&mut self, set: usize, resource: Handle, index: usize) {
        let bindset = self
            .binding_sets
            .get_mut(set)
            .unwrap_or_else(|| panic!("BindResource has not set {}", set));

        debug_assert!(!bindset.bindings.iter().any(|(_, i)| *i == index));

        bindset.bindings.push((resource, index));
    }

    pub fn remove_binding(&mut self, set: usize, index: usize) {
        let bindset = self
            .binding_sets
            .get_mut(set)
            .unwrap_or_else(|| panic!("BindResource has not set {}", set));

        let pos = bindset
            .bindings
            .iter()
            .position(|(_, i)| *i == index)
            .unwrap_or_else(|| panic!("BindResource set {} has not index {}", set, index));

        bindset.bindings.remove(pos);
    }

    pub fn next(mut self) -> Self {
        self.binding_sets.push(BindSet::new());

        self
    }

    pub fn slot(&self, index: usize) -> Option<Handle> {
        self.binding_sets
            .get(index)
            .and_then(|set| set.bindings.first())
            .map(|b| b.0)
    }

    pub fn sets(&self) -> usize {
        self.binding_sets.len()
    }

    pub fn bindset(&self, idx: usize) -> &BindSet {
        &self.binding_sets[idx]
    }

    pub fn bindsets(&self) -> impl Iterator<Item = &BindSet> {
        self.binding_sets.iter()
    }

    pub fn layout(&self) -> &BindingGroupLayout {
        &self.layout
    }
}

#[derive(Copy, Clone, Eq, Hash, Serialize, Deserialize, PartialEq)]
pub enum BindingGroupType {
    None,
    ComputeData,
    SceneData,
    MaterialData,
    MaterialTextures,
    BindlessTextures,
}

impl Debug for BindingGroupType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::ComputeData => write!(f, "ComputeData ({})", self.set()),
            Self::SceneData => write!(f, "SceneData ({}, push)", self.set()),
            Self::MaterialData => write!(f, "MaterialData ({})", self.set()),
            Self::MaterialTextures => write!(f, "MaterialTextures ({})", self.set()),
            Self::BindlessTextures => write!(f, "BindlessTextures ({})", self.set()),
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
            // bindless / material textures are mutually exclusive
            BindingGroupType::MaterialTextures => 2,
            BindingGroupType::BindlessTextures => 2,
        }
    }

    pub fn lifetime(&self) -> BindingLifetime {
        match self {
            BindingGroupType::BindlessTextures => BindingLifetime::Static,
            _ => BindingLifetime::PerFrame,
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
    pub fn new(binding_group_type: BindingGroupType) -> Arc<Self> {
        let mut hasher = DefaultHasher::new();
        binding_group_type.hash(&mut hasher);
        let binding_group_id = hasher.finish();

        Arc::new(Self {
            binding_group_id,
            binding_group_type,
            bindings: Vec::new(),
        })
    }

    pub fn add_binding(
        self: Arc<Self>,
        ty: DescriptorType,
        stage: DescriptorStage,
        count: u32,
    ) -> Arc<Self> {
        let mut group = Arc::unwrap_or_clone(self);

        group.bindings.push((ty, stage, count));

        // generate content hash for layout
        let mut hasher = DefaultHasher::new();
        group.binding_group_type.hash(&mut hasher);
        group.bindings.hash(&mut hasher);
        group.binding_group_id = hasher.finish();

        Arc::new(group)
    }
}
