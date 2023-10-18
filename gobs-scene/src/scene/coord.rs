use cgmath::Point3;
use std::ops::AddAssign;

#[derive(Clone, Copy)]
pub struct SphericalCoord<T> {
    r: T,
    theta: T,
    phi: T,
}

impl<T> SphericalCoord<T>
where
    T: AddAssign + Clone,
{
    pub fn new(r: T, theta: T, phi: T) -> Self {
        SphericalCoord {
            r: r,
            theta: theta,
            phi: phi,
        }
    }

    pub fn update(&mut self, dr: T, dtheta: T, dphi: T) {
        self.r += dr;
        self.theta += dtheta;
        self.phi += dphi;
    }

    pub fn set(&mut self, r: T, theta: T, phi: T) {
        self.r = r;
        self.theta = theta;
        self.phi = phi;
    }
}

impl From<SphericalCoord<f32>> for Point3<f32> {
    fn from(coord: SphericalCoord<f32>) -> Point3<f32> {
        let posx = coord.r * coord.theta.sin() * coord.phi.cos();
        let posy = coord.r * coord.phi.sin();
        let posz = coord.r * coord.theta.cos() * coord.phi.cos();

        Point3::new(posx, posy, posz)
    }
}
