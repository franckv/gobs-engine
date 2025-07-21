use std::collections::HashMap;

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use gobs_core::{ImageExtent2D, ImageFormat};
use gobs_gfx::{GfxImage, Image, ImageUsage};

use crate::GfxContext;

pub struct GraphResourceManager {
    pub resources: HashMap<String, RwLock<GfxImage>>,
}

impl GraphResourceManager {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn register_image(
        &mut self,
        ctx: &GfxContext,
        label: &str,
        format: ImageFormat,
        usage: ImageUsage,
        extent: ImageExtent2D,
    ) {
        let image = GfxImage::new(label, &ctx.device, format, usage, extent);

        self.resources.insert(label.to_string(), RwLock::new(image));
    }

    pub fn invalidate(&self) {
        for image in self.resources.values() {
            image.write().invalidate();
        }
    }

    pub fn image_read(&self, label: &str) -> RwLockReadGuard<'_, GfxImage> {
        assert!(
            self.resources.contains_key(label),
            "Missing resource {label}",
        );

        self.resources[label].read()
    }

    pub fn image_write(&self, label: &str) -> RwLockWriteGuard<'_, GfxImage> {
        assert!(
            self.resources.contains_key(label),
            "Missing resource {label}",
        );

        self.resources[label].write()
    }
}

impl Default for GraphResourceManager {
    fn default() -> Self {
        Self::new()
    }
}
