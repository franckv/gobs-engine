use crate::voxel::chunk::Chunks;

pub struct RayHit {
    pub pos: [i64; 3],
    pub face_normal: [f32; 3],
}

pub trait RayCast {
    fn raycast(&self, origin: [f32; 3], dir: [f32; 3], max_distance: f32) -> Option<RayHit>;
}

impl<D> RayCast for Chunks<D> {
    fn raycast(&self, origin: [f32; 3], dir: [f32; 3], max_distance: f32) -> Option<RayHit> {
        let mut pos = Chunks::<D>::integer_position(origin);

        let step_dir = [
            if dir[0] > 0. { 1 } else { -1 },
            if dir[1] > 0. { 1 } else { -1 },
            if dir[2] > 0. { 1 } else { -1 },
        ];

        let delta = [
            if dir[0] != 0. {
                1. / dir[0].abs()
            } else {
                f32::INFINITY
            },
            if dir[1] != 0. {
                1. / dir[1].abs()
            } else {
                f32::INFINITY
            },
            if dir[2] != 0. {
                1. / dir[2].abs()
            } else {
                f32::INFINITY
            },
        ];

        let fract = [
            origin[0] - pos[0] as f32,
            origin[1] - pos[1] as f32,
            origin[2] - pos[2] as f32,
        ];

        let mut t_max = [
            if delta[0].is_infinite() {
                f32::INFINITY
            } else {
                (if dir[0] > 0. { 1. - fract[0] } else { fract[0] }) * delta[0]
            },
            if delta[1].is_infinite() {
                f32::INFINITY
            } else {
                (if dir[1] > 0. { 1. - fract[1] } else { fract[1] }) * delta[1]
            },
            if delta[2].is_infinite() {
                f32::INFINITY
            } else {
                (if dir[2] > 0. { 1. - fract[2] } else { fract[2] }) * delta[2]
            },
        ];

        let mut distance = 0.;
        let mut face_normal = [0.; 3];

        while distance < max_distance {
            let voxel_pos = self.world_to_chunk(pos);
            let chunk = self.get_chunk(voxel_pos.chunk)?;
            if chunk.is_solid(
                voxel_pos.local[0] as i64,
                voxel_pos.local[1] as i64,
                voxel_pos.local[2] as i64,
            ) {
                return Some(RayHit { pos, face_normal });
            };

            if t_max[0] < t_max[1] && t_max[0] < t_max[2] {
                pos[0] += step_dir[0];
                distance = t_max[0];
                t_max[0] += delta[0];
                face_normal = [-step_dir[0] as f32, 0., 0.];
            } else if t_max[1] < t_max[2] {
                pos[1] += step_dir[1];
                distance = t_max[1];
                t_max[1] += delta[1];
                face_normal = [0., -step_dir[1] as f32, 0.];
            } else {
                pos[2] += step_dir[2];
                distance = t_max[2];
                t_max[2] += delta[2];
                face_normal = [0., 0., -step_dir[2] as f32];
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use tracing::Level;
    use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

    use crate::voxel::{chunk::Chunks, ray::RayCast as _};

    struct VoxelData;

    fn setup() {
        let sub = FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .with_span_events(FmtSpan::CLOSE)
            .finish();
        tracing::subscriber::set_global_default(sub).unwrap_or_default();
    }

    #[test]
    fn test_raycast_single_chunk() {
        setup();

        let mut chunks = Chunks::new(2);
        let tree = chunks.get_or_create_chunk([0, 0, 0]);
        tree.insert(VoxelData, 5, 5, 5);

        let origin = [5., 5., 10.];
        let dir = [0., 0., -1.];

        let hit = chunks.raycast(origin, dir, 20.);
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.pos, [5, 5, 5]);
        assert_eq!(hit.face_normal, [0., 0., 1.]);

        let dir = [0., 0., 1.];
        let hit = chunks.raycast(origin, dir, 20.);
        assert!(hit.is_none());

        let origin = [5., 5., 15.];
        let dir = [0., 0., -1.];
        let hit = chunks.raycast(origin, dir, 20.);
        assert!(hit.is_some());
        let hit = chunks.raycast(origin, dir, 5.);
        assert!(hit.is_none());

        let tree = chunks.get_or_create_chunk([0, 0, 0]);
        tree.insert(VoxelData, 5, 5, 7);
        let origin = [5., 5., 10.];
        let dir = [0., 0., -1.];

        let hit = chunks.raycast(origin, dir, 20.);
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.pos, [5, 5, 7]);
        assert_eq!(hit.face_normal, [0., 0., 1.]);

        let origin = [5., 5., 0.];
        let dir = [0., 0., 1.];
        let hit = chunks.raycast(origin, dir, 20.);
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.pos, [5, 5, 5]);
        assert_eq!(hit.face_normal, [0., 0., -1.]);

        let mut chunks = Chunks::new(3);
        let tree = chunks.get_or_create_chunk([0, 0, 0]);
        tree.insert(VoxelData, 10, 10, 10);

        let origin = [0., 0., 0.];
        let dir: [f32; 3] = [10., 10., 10.];
        let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        let dir = [dir[0] / len, dir[1] / len, dir[2] / len];

        let hit = chunks.raycast(origin, dir, 30.);
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.pos, [10, 10, 10]);
        assert_eq!(hit.face_normal, [-1., 0., 0.]);

        let mut chunks = Chunks::new(2);
        let tree = chunks.get_or_create_chunk([0, 0, 0]);
        tree.insert(VoxelData, 5, 0, 5);

        let origin = [5., 3., 5.];
        let dir = [0., -1., 0.];

        let hit = chunks.raycast(origin, dir, 20.);
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.pos, [5, 0, 5]);
        assert_eq!(hit.face_normal, [0., 1., 0.]);

        let mut chunks = Chunks::new(2);
        let tree = chunks.get_or_create_chunk([0, 0, 0]);
        tree.insert(VoxelData, 5, 5, 5);

        let origin = [5., 5., 5.];
        let dir = [1., 0., 0.];

        let hit = chunks.raycast(origin, dir, 20.);
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.pos, [5, 5, 5]);
        assert_eq!(hit.face_normal, [0., 0., 0.]);

        let origin = [5.49, 5., 10.];
        let dir = [0., 0., -1.];
        let hit = chunks.raycast(origin, dir, 20.);
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.pos, [5, 5, 5]);
        assert_eq!(hit.face_normal, [0., 0., 1.]);

        let tree = chunks.get_or_create_chunk([0, 0, 0]);
        tree.insert(VoxelData, 6, 5, 5);

        let origin = [4.6, 5., 5.];
        let dir = [1., 0., 0.];
        let hit = chunks.raycast(origin, dir, 20.);
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.pos, [5, 5, 5]);
        assert_eq!(hit.face_normal, [0., 0., 0.]);

        let origin = [5.4, 5., 5.];
        let dir = [1., 0., 0.];
        let hit = chunks.raycast(origin, dir, 20.);
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.pos, [5, 5, 5]);
        assert_eq!(hit.face_normal, [0., 0., 0.]);
    }

    #[test]
    fn test_raycast_cross_chunks() {
        setup();

        let mut chunks = Chunks::new(1);
        chunks.get_or_create_chunk([-1, 0, 0]);
        let chunk = chunks.get_or_create_chunk([0, 0, 0]);
        chunk.insert(VoxelData, 1, 1, 1);

        let origin = [-2., 1., 1.];
        let dir = [1., 0., 0.];

        let hit = chunks.raycast(origin, dir, 20.);
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.pos, [1, 1, 1]);
        assert_eq!(hit.face_normal, [-1., 0., 0.]);

        let mut chunks = Chunks::new(1);
        chunks.get_or_create_chunk([-1, 0, 0]);
        chunks.get_or_create_chunk([0, 0, 0]);
        let chunk = chunks.get_or_create_chunk([1, 0, 0]);
        chunk.insert(VoxelData, 1, 1, 1);

        let origin = [-2., 1., 1.];
        let dir = [1., 0., 0.];

        let hit = chunks.raycast(origin, dir, 20.);
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.pos, [5, 1, 1]);
        assert_eq!(hit.face_normal, [-1., 0., 0.]);

        let mut chunks = Chunks::new(1);
        chunks.get_or_create_chunk([-3, 0, 0]);
        chunks.get_or_create_chunk([-2, 0, 0]);
        let chunk = chunks.get_or_create_chunk([-1, 0, 0]);
        chunk.insert(VoxelData, 3, 1, 1);

        let origin = [-10., 1., 1.];
        let dir = [1., 0., 0.];

        let hit = chunks.raycast(origin, dir, 20.);
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.pos, [-1, 1, 1]);
        assert_eq!(hit.face_normal, [-1., 0., 0.]);
    }
}
