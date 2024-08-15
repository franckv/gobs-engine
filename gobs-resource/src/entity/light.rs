use uuid::Uuid;

use gobs_core::Color;

pub type LightId = Uuid;

#[derive(Clone, Debug)]
pub struct Light {
    pub id: LightId,
    pub colour: Color,
}

impl Light {
    pub fn new(colour: Color) -> Self {
        Light {
            id: Uuid::new_v4(),
            colour,
        }
    }
}
