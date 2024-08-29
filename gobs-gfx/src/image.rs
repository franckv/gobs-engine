use gobs_core::{ImageExtent2D, ImageFormat, SamplerFilter};

use crate::ImageUsage;
use crate::Renderer;

pub trait Image<R: Renderer> {
    fn new(
        name: &str,
        device: &R::Device,
        format: ImageFormat,
        usage: ImageUsage,
        extent: ImageExtent2D,
    ) -> Self;
    fn invalidate(&mut self);
    fn extent(&self) -> ImageExtent2D;
    fn name(&self) -> &str;
    fn format(&self) -> ImageFormat;
}

pub trait Sampler<R: Renderer> {
    fn new(device: &R::Device, mag_filter: SamplerFilter, min_filter: SamplerFilter) -> Self;
    fn mag_filter(&self) -> SamplerFilter;
    fn min_filter(&self) -> SamplerFilter;
}
