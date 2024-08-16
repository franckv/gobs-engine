use gobs_core::ImageExtent2D;

use crate::{ImageFormat, ImageUsage, SamplerFilter};

pub trait Image {
    type GfxDevice;

    fn new(
        name: &str,
        device: &Self::GfxDevice,
        format: ImageFormat,
        usage: ImageUsage,
        extent: ImageExtent2D,
    ) -> Self;
    fn invalidate(&mut self);
    fn extent(&self) -> ImageExtent2D;
    fn name(&self) -> &str;
    fn format(&self) -> ImageFormat;
}

pub trait Sampler {
    type GfxDevice;

    fn new(device: &Self::GfxDevice, mag_filter: SamplerFilter, min_filter: SamplerFilter) -> Self;
    fn mag_filter(&self) -> SamplerFilter;
    fn min_filter(&self) -> SamplerFilter;
}
