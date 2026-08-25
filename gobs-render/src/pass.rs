use crate::pass::{compute::ComputePassData, material::MaterialPassData, present::PresentPassData};

pub mod compute;
pub mod material;
pub mod pass_loader;
pub mod present;

pub enum PassData {
    Material(MaterialPassData),
    Present(PresentPassData),
    Compute(ComputePassData),
}
