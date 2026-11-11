mod manager;
mod resource;

pub use manager::{ResourceManager, ResourceRegistry, SharedResourceManager};
pub use resource::{
    Resource, ResourceError, ResourceHandle, ResourceLifetime, ResourceLoader, ResourceProperties,
    ResourceType,
};

pub mod load;
