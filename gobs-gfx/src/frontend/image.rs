use crate::{GfxDevice, ImageExtent2D, ImageFormat, ImageUsage, SamplerFilter};

pub trait Image {
    fn new(
        name: &str,
        device: &GfxDevice,
        format: ImageFormat,
        usage: ImageUsage,
        extent: ImageExtent2D,
    ) -> Self;
    fn invalidate(&mut self);
    fn extent(&self) -> ImageExtent2D;
}

pub trait Sampler {
    fn new(device: &GfxDevice, mag_filter: SamplerFilter, min_filter: SamplerFilter) -> Self;
}
