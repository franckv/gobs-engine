#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum BindingGroupType {
    ComputeData,
    SceneData,
    MaterialData,
}

impl BindingGroupType {
    pub fn set(&self) -> u32 {
        match self {
            BindingGroupType::ComputeData => 0,
            BindingGroupType::SceneData => 0,
            BindingGroupType::MaterialData => 1,
        }
    }
}
