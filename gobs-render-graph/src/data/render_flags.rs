use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct RenderFlags: u32 {
        const ENTITY = 1 << 0;
        const TRANSPARENT = 1 << 1;
        const OPAQUE = 1 << 2;
        const UI = 1 << 3;
        const SELECTED = 1 << 4;
        const BOUNDS = 1 << 5;
    }
}
