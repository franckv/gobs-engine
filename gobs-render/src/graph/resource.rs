use std::collections::HashMap;

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use gobs_core::{ImageExtent2D, ImageFormat};
use gobs_gfx::{Image, ImageUsage, Renderer};

use crate::context::Context;

pub struct ResourceManager<R: Renderer> {
    pub resources: HashMap<String, RwLock<R::Image>>,
}

impl<R: Renderer> ResourceManager<R> {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn register_image(
        &mut self,
        ctx: &Context<R>,
        label: &str,
        format: ImageFormat,
        usage: ImageUsage,
        extent: ImageExtent2D,
    ) {
        let image = R::Image::new(label, &ctx.device, format, usage, extent);

        self.resources.insert(label.to_string(), RwLock::new(image));
    }

    pub fn invalidate(&self) {
        for (_, image) in &self.resources {
            image.write().invalidate();
        }
    }

    pub fn image_read(&self, label: &str) -> RwLockReadGuard<'_, R::Image> {
        assert!(
            self.resources.contains_key(label),
            "Missing resource {}",
            label
        );

        self.resources[label].read()
    }

    pub fn image_write(&self, label: &str) -> RwLockWriteGuard<'_, R::Image> {
        assert!(
            self.resources.contains_key(label),
            "Missing resource {}",
            label
        );

        self.resources[label].write()
    }
}
