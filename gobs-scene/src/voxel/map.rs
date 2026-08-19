use std::{marker::PhantomData, sync::Arc};

use gobs_core::{Color, logger, utils::timer::Timer};
use gobs_render::{MeshGeometry, ShapeBuilder};

use crate::voxel::node::VoxelNode;

pub struct VoxelTree<D, N: VoxelNode<D>> {
    root: usize,
    order: u32,
    nodes: Vec<N>,
    marker: PhantomData<D>,
}

impl<D, N: VoxelNode<D>> VoxelTree<D, N> {
    pub fn new(order: u32) -> Self {
        let root_node = N::new();

        Self {
            root: 0,
            order,
            nodes: vec![root_node],
            marker: PhantomData,
        }
    }

    pub fn size(order: u32) -> u32 {
        N::SUBDIVISION.pow(order)
    }

    pub fn allocate_node(&mut self) -> usize {
        self.nodes.push(N::new());

        self.nodes.len() - 1
    }

    /*
     * Insert a voxel in a tree at a specific position.
     * x, y, z is the relative position of a voxel within a tree (chunk).
     * Coordinate range is (0, 0, 0) to (max, max, max) where max = SUBDIVISION^order
     */
    pub fn insert(&mut self, data: D, x: u32, y: u32, z: u32) -> Option<D> {
        let mut node_idx = self.root;

        for level in 0..=self.order {
            self.nodes[node_idx].set_dirty(true);

            if level == self.order {
                return self.nodes[node_idx].set_data(data);
            }

            let child_idx = Self::index(x, y, z, level, self.order);

            if let Some(child_node_idx) = self.nodes[node_idx].child(child_idx) {
                node_idx = child_node_idx;
            } else {
                let child_node_idx = self.allocate_node();
                self.nodes[node_idx].add_child(child_idx, child_node_idx);
                node_idx = child_node_idx;
            }
        }

        None
    }

    pub fn get(&self, x: u32, y: u32, z: u32) -> Option<&D> {
        let mut node_idx = self.root;

        for level in 0..=self.order {
            if level == self.order {
                return self.nodes[node_idx].get_data();
            }

            let child_idx = Self::index(x, y, z, level, self.order);

            let child_node_idx = self.nodes[node_idx].child(child_idx)?;
            node_idx = child_node_idx;
        }

        None
    }

    pub fn is_solid(&self, x: i64, y: i64, z: i64) -> bool {
        if x < 0 || y < 0 || z < 0 {
            return false;
        }

        let voxel_count = Self::size(self.order) as i64;
        if x >= voxel_count || y >= voxel_count || z >= voxel_count {
            return false;
        }

        self.get(x as u32, y as u32, z as u32).is_some()
    }

    fn check_bounds(x: u32, y: u32, z: u32, level: u32, order: u32) -> bool {
        debug_assert!(level <= order);

        let voxel_count = Self::size(order);
        debug_assert!(x < voxel_count);
        debug_assert!(y < voxel_count);
        debug_assert!(z < voxel_count);

        true
    }

    /*
     * Return the coordinate of a voxel at a specific level
     * x, y, z is the relative position of a voxel within a tree (chunk)
     */
    pub fn location(x: u32, y: u32, z: u32, level: u32, order: u32) -> (u32, u32, u32) {
        debug_assert!(Self::check_bounds(x, y, z, level, order));

        let voxel_per_level = Self::size(order - level);

        (
            x / voxel_per_level,
            y / voxel_per_level,
            z / voxel_per_level,
        )
    }

    /*
     * Return the child index (0-63) of a voxel at a specific level
     * x, y, z is the relative position of a voxel within a tree (chunk)
     */
    pub fn index(x: u32, y: u32, z: u32, level: u32, order: u32) -> usize {
        debug_assert!(Self::check_bounds(x, y, z, level, order));
        debug_assert!(level < order);

        let voxel_per_level = Self::size(order - level);
        let voxel_per_child = Self::size(order - level - 1);

        let (rel_x, rel_y, rel_z) = (
            (x % voxel_per_level) / voxel_per_child,
            (y % voxel_per_level) / voxel_per_child,
            (z % voxel_per_level) / voxel_per_child,
        );

        (rel_x + N::SUBDIVISION * rel_y + N::SUBDIVISION * N::SUBDIVISION * rel_z) as usize
    }

    pub fn offset(idx: usize) -> [u32; 3] {
        debug_assert!(idx < N::CHILDREN_NUMBER);

        let mut idx = idx as u32;

        let x = idx % N::SUBDIVISION;
        idx /= N::SUBDIVISION;
        let y = idx % N::SUBDIVISION;
        idx /= N::SUBDIVISION;
        let z = idx % N::SUBDIVISION;

        [x, y, z]
    }

    pub fn is_dirty(&self) -> bool {
        self.nodes[self.root].is_dirty()
    }

    pub fn visit<F>(&mut self, clean: bool, visitor: &mut F)
    where
        F: FnMut([u32; 3], &D),
    {
        self.visit_local(self.root, [0; 3], 0, clean, visitor);
    }

    pub fn visit_local<F>(
        &mut self,
        root: usize,
        pos: [u32; 3],
        level: u32,
        clean: bool,
        visitor: &mut F,
    ) where
        F: FnMut([u32; 3], &D),
    {
        if let Some(data) = self.nodes[root].get_data() {
            visitor(pos, data);
            if clean {
                self.nodes[root].set_dirty(false);
            }
        } else if self.nodes[root].has_children() {
            for idx in 0..N::CHILDREN_NUMBER {
                if let Some(child_idx) = self.nodes[root].child(idx) {
                    let scale = Self::size(self.order - level - 1);
                    let offset = Self::offset(idx);
                    let child_pos = [
                        pos[0] + scale * offset[0],
                        pos[1] + scale * offset[1],
                        pos[2] + scale * offset[2],
                    ];
                    self.visit_local(child_idx, child_pos, level + 1, clean, visitor);
                }
            }
            if clean {
                self.nodes[root].set_dirty(false);
            }
        } else {
            debug_assert!(!self.nodes[root].is_dirty());
        }
    }

    pub fn meshify(&mut self) -> Arc<MeshGeometry> {
        let mut timer = Timer::new();

        let mut builder = ShapeBuilder::new("voxel").with_colors(&[Color::RED, Color::BLUE]);

        let size = 1.;
        let (top, bottom, left, right, front, back) = (
            size / 2.,
            -size / 2.,
            -size / 2.,
            size / 2.,
            size / 2.,
            -size / 2.,
        );

        let mut positions = Vec::new();

        self.visit(true, &mut |pos, _| {
            positions.push(pos);
        });

        let count = positions.len();
        let mut faces = 0;

        for pos in positions {
            let top = top + pos[1] as f32;
            let bottom = bottom + pos[1] as f32;
            let left = left + pos[0] as f32;
            let right = right + pos[0] as f32;
            let front = front + pos[2] as f32;
            let back = back + pos[2] as f32;

            let v = [
                [left, top, front],
                [right, top, front],
                [left, bottom, front],
                [right, bottom, front],
                [left, top, back],
                [right, top, back],
                [left, bottom, back],
                [right, bottom, back],
            ];

            if !self.is_solid(pos[0] as i64, pos[1] as i64, pos[2] as i64 + 1) {
                builder = builder.add_quad([v[2], v[0], v[3], v[1]]); // F
                faces += 1;
            }
            if !self.is_solid(pos[0] as i64, pos[1] as i64, pos[2] as i64 - 1) {
                builder = builder.add_quad([v[7], v[5], v[6], v[4]]); // B
                faces += 1;
            }
            if !self.is_solid(pos[0] as i64 - 1, pos[1] as i64, pos[2] as i64) {
                builder = builder.add_quad([v[6], v[4], v[2], v[0]]); // L
                faces += 1;
            }
            if !self.is_solid(pos[0] as i64 + 1, pos[1] as i64, pos[2] as i64) {
                builder = builder.add_quad([v[3], v[1], v[7], v[5]]); // R
                faces += 1;
            }
            if !self.is_solid(pos[0] as i64, pos[1] as i64 + 1, pos[2] as i64) {
                builder = builder.add_quad([v[0], v[4], v[1], v[5]]); // U
                faces += 1;
            }
            if !self.is_solid(pos[0] as i64, pos[1] as i64 - 1, pos[2] as i64) {
                builder = builder.add_quad([v[6], v[2], v[7], v[3]]); // D
                faces += 1;
            }
        }

        tracing::info!(target: logger::RENDER, "Meshify {} voxels ({} faces) in {}ms", count, faces, 1000. * timer.delta());

        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use tracing::Level;
    use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

    use crate::voxel::{
        map::{VoxelNode as _, VoxelTree},
        node::{VoxelChildData, VoxelNode64},
    };

    struct VoxelData;

    type Tree = VoxelTree<VoxelData, VoxelNode64<VoxelData>>;

    fn setup() {
        let sub = FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .with_span_events(FmtSpan::CLOSE)
            .finish();
        tracing::subscriber::set_global_default(sub).unwrap_or_default();
    }

    #[test]
    fn test_location() {
        setup();

        // order-1 tree: 4x4x4 voxels
        // ||00|01|02|03||
        assert_eq!(Tree::location(3, 3, 3, 0, 1), (0, 0, 0));

        // order-2 tree: 16x16x16 voxels
        // ||00|01|02|03| |04|05|06|07| |08|09|10|11| |12|13|14|15||
        assert_eq!(Tree::location(2, 13, 9, 0, 2), (0, 0, 0));
        assert_eq!(Tree::location(2, 13, 9, 1, 2), (0, 3, 2));

        // order-3 tree: 64x64x64 voxels
        assert_eq!(Tree::location(42, 17, 63, 0, 3), (0, 0, 0));
        assert_eq!(Tree::location(42, 17, 63, 1, 3), (2, 1, 3));
        assert_eq!(Tree::location(42, 17, 63, 2, 3), (10, 4, 15));
    }

    #[test]
    fn test_index() {
        setup();

        // order-1 tree: 4x4x4 voxels
        // ||00|01|02|03||
        assert_eq!(Tree::index(0, 0, 0, 0, 1), 0);
        assert_eq!(Tree::index(3, 3, 3, 0, 1), 63);
        assert_eq!(Tree::index(1, 3, 2, 0, 1), 45);

        // order-2 tree: 16x16x16 voxels
        // ||00|01|02|03| |04|05|06|07| |08|09|10|11| |12|13|14|15||
        assert_eq!(Tree::index(2, 13, 9, 0, 2), 44);
        assert_eq!(Tree::index(2, 13, 9, 1, 2), 22);

        // order-3 tree: 64x64x64 voxels
        assert_eq!(Tree::index(42, 17, 63, 0, 3), 54);
        assert_eq!(Tree::index(42, 17, 63, 1, 3), 50);
        assert_eq!(Tree::index(42, 17, 63, 2, 3), 54);
    }

    #[test]
    fn test_location_panic() {
        setup();

        let result = std::panic::catch_unwind(|| {
            Tree::location(2, 13, 9, 3, 2);
        });
        assert!(result.is_err());

        let result = std::panic::catch_unwind(|| {
            Tree::location(2, 16, 9, 2, 2);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_offset() {
        setup();

        assert_eq!(Tree::offset(0), [0, 0, 0]);
        assert_eq!(Tree::offset(1), [1, 0, 0]);
        assert_eq!(Tree::offset(4), [0, 1, 0]);
        assert_eq!(Tree::offset(16), [0, 0, 1]);
        assert_eq!(Tree::offset(42), [2, 2, 2]);
        assert_eq!(Tree::offset(63), [3, 3, 3]);
    }

    #[test]
    fn test_insert() {
        setup();

        let mut tree = Tree::new(0);
        tree.insert(VoxelData, 0, 0, 0);
        assert!(matches!(
            tree.nodes[tree.root].data,
            VoxelChildData::Leaf(_)
        ));

        let mut tree = Tree::new(1);
        tree.insert(VoxelData, 2, 1, 3);
        let idx = Tree::index(2, 1, 3, 0, 1);
        assert!(matches!(
            tree.nodes[tree.root].data,
            VoxelChildData::Children(_)
        ));
        assert!(tree.nodes[tree.root].has_child(idx));
        if let VoxelChildData::Children(children) = &tree.nodes[tree.root].data {
            let child_idx = children[idx];
            assert!(matches!(
                &tree.nodes[child_idx].data,
                VoxelChildData::Leaf(_)
            ));
        }

        let mut tree = Tree::new(2);
        tree.insert(VoxelData, 0, 0, 0);
        tree.insert(VoxelData, 1, 0, 0);
        assert!(matches!(
            tree.nodes[tree.root].data,
            VoxelChildData::Children(_)
        ));
        let idx1 = Tree::index(0, 0, 0, 0, 2);
        let idx2 = Tree::index(1, 0, 0, 0, 2);
        assert_eq!(idx1, idx2);
        assert!(tree.nodes[tree.root].has_child(idx1));
        if let VoxelChildData::Children(children) = &tree.nodes[tree.root].data {
            let l1 = &tree.nodes[children[idx1]];
            let idx1 = Tree::index(0, 0, 0, 1, 2);
            let idx2 = Tree::index(1, 0, 0, 1, 2);
            assert_ne!(idx1, idx2);
            assert!(l1.has_child(idx1));
            assert!(l1.has_child(idx2));
        }

        let mut tree = Tree::new(3);
        tree.insert(VoxelData, 63, 63, 63);
        let mut node_idx = tree.root;
        for level in 0..3 {
            let idx = Tree::index(63, 63, 63, level, 3);
            assert_eq!(idx, 63);
            assert!(tree.nodes[node_idx].has_child(idx));
            node_idx = match &tree.nodes[node_idx].data {
                VoxelChildData::Children(children) => children[idx],
                _ => panic!("Missing children data, level={}", level),
            };
        }
        assert!(matches!(tree.nodes[node_idx].data, VoxelChildData::Leaf(_)));

        let mut tree = Tree::new(3);
        tree.insert(VoxelData, 0, 0, 0);
        tree.insert(VoxelData, 63, 63, 63);
        let mut node_idx1 = tree.root;
        let mut node_idx2 = tree.root;
        for level in 0..3 {
            let idx1 = Tree::index(0, 0, 0, level, 3);
            let idx2 = Tree::index(63, 63, 63, level, 3);
            assert_ne!(idx1, idx2);
            assert!(tree.nodes[node_idx1].has_child(idx1));
            assert!(tree.nodes[node_idx2].has_child(idx2));
            node_idx1 = match &tree.nodes[node_idx1].data {
                VoxelChildData::Children(children) => children[idx1],
                _ => panic!("Missing children data, level={}", level),
            };
            node_idx2 = match &tree.nodes[node_idx2].data {
                VoxelChildData::Children(children) => children[idx2],
                _ => panic!("Missing children data, level={}", level),
            };
        }
    }

    #[test]
    fn test_get() {
        setup();

        let mut tree = Tree::new(0);
        assert!(tree.get(0, 0, 0).is_none());
        tree.insert(VoxelData, 0, 0, 0);
        assert!(tree.get(0, 0, 0).is_some());

        let mut tree = Tree::new(2);
        assert!(tree.get(0, 0, 0).is_none());
        tree.insert(VoxelData, 0, 0, 0);
        assert!(tree.get(0, 0, 0).is_some());
        assert!(tree.get(1, 0, 0).is_none());
        tree.insert(VoxelData, 0, 0, 0);
        assert!(tree.get(0, 0, 0).is_some());

        let mut tree = Tree::new(3);
        assert!(tree.get(63, 63, 63).is_none());
        tree.insert(VoxelData, 63, 63, 63);
        assert!(tree.get(63, 63, 63).is_some());
        assert!(tree.get(62, 63, 63).is_none());

        let mut tree = Tree::new(3);
        tree.insert(VoxelData, 0, 0, 0);
        assert!(tree.get(63, 63, 63).is_none());
    }

    #[test]
    fn test_dirty() {
        setup();

        let mut tree = Tree::new(2);
        tree.insert(VoxelData, 0, 0, 0);
        let idx = Tree::index(0, 0, 0, 0, 2);
        let node_idx = tree.nodes[tree.root].child(idx).unwrap();

        assert!(tree.nodes[tree.root].is_dirty());
        assert!(tree.nodes[node_idx].is_dirty());

        for node in &mut tree.nodes {
            node.set_dirty(false);
        }

        assert!(!tree.nodes[tree.root].is_dirty());
        assert!(!tree.nodes[node_idx].is_dirty());

        tree.insert(VoxelData, 1, 0, 0);

        assert!(tree.nodes[tree.root].is_dirty());
        assert!(tree.nodes[node_idx].is_dirty());

        let idx1 = Tree::index(0, 0, 0, 1, 2);
        let idx2 = Tree::index(1, 0, 0, 1, 2);
        let node_idx1 = tree.nodes[node_idx].child(idx1).unwrap();
        let node_idx2 = tree.nodes[node_idx].child(idx2).unwrap();
        assert!(!tree.nodes[node_idx1].is_dirty());
        assert!(tree.nodes[node_idx2].is_dirty());
    }

    #[test]
    fn test_visit() {
        setup();

        let tests = vec![
            [0, 0, 0],
            [2, 1, 3],
            [42, 0, 0],
            [0, 42, 0],
            [0, 0, 42],
            [42, 42, 42],
            [63, 63, 63],
        ];
        let mut tree = Tree::new(3);

        for test in &tests {
            let data = tree.insert(VoxelData, test[0], test[1], test[2]);
            assert!(data.is_none());
        }

        assert!(tree.nodes[tree.root].is_dirty());

        tree.visit(false, &mut |_, _| {});
        assert!(tree.nodes[tree.root].is_dirty());

        let mut visited = Vec::new();
        tree.visit(true, &mut |pos, _| {
            visited.push(pos);
        });

        assert!(!tree.nodes[tree.root].is_dirty());

        assert_eq!(visited.len(), tests.len());
        for test in &tests {
            assert!(visited.contains(test));
        }

        let data = tree.insert(VoxelData, tests[0][0], tests[0][1], tests[0][2]);
        assert_eq!(visited.len(), tests.len());
        assert!(data.is_some());

        let mut tree = Tree::new(0);
        tree.insert(VoxelData, 0, 0, 0);
        let mut visited = Vec::new();
        tree.visit(true, &mut |pos, _| visited.push(pos));
        assert_eq!(visited, vec![[0, 0, 0]]);

        let mut tree = Tree::new(1);
        tree.insert(VoxelData, 3, 3, 3);
        tree.insert(VoxelData, 1, 2, 3);
        let mut visited = Vec::new();
        tree.visit(false, &mut |pos, _| visited.push(pos));
        assert_eq!(visited.len(), 2);
        assert!(visited.contains(&[3, 3, 3]));
        assert!(visited.contains(&[1, 2, 3]));
    }
}
