use std::collections::HashMap;

use gobs_render_hal::{Handle, ImageUsage, RenderHAL};

use gobs_core::{ImageExtent2D, ImageFormat};

use crate::GfxContext;

pub struct GraphResourceManager {
    pub resources: HashMap<String, Handle>,
}

impl GraphResourceManager {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn register_image(
        &mut self,
        ctx: &mut GfxContext,
        label: &str,
        format: ImageFormat,
        usage: ImageUsage,
        extent: ImageExtent2D,
    ) {
        let image = ctx.hal_mut().create_image(label, format, usage, extent);

        self.resources.insert(label.to_string(), image);
    }

    pub fn invalidate(&self, hal: &mut dyn RenderHAL) {
        for image in self.resources.values() {
            hal.invalidate_image(*image);
        }
    }

    pub fn image(&self, label: &str) -> Handle {
        assert!(
            self.resources.contains_key(label),
            "Missing resource {label}",
        );

        self.resources[label]
    }
}

impl Default for GraphResourceManager {
    fn default() -> Self {
        Self::new()
    }
}
