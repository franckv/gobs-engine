use std::marker::PhantomData;

use crate::voxel::node::VoxelNode;

pub struct VoxelTree<D, N: VoxelNode<D>> {
    root: usize,
    order: u32,
    data: Vec<N>,
    marker: PhantomData<D>,
}

impl<D, N: VoxelNode<D>> VoxelTree<D, N> {
    pub fn new(order: u32) -> Self {
        let root_node = N::new();

        Self {
            root: 0,
            order,
            data: vec![root_node],
            marker: PhantomData,
        }
    }

    pub fn allocate_node(&mut self) -> usize {
        self.data.push(N::new());

        self.data.len() - 1
    }

    pub fn insert(&mut self, data: D, x: u32, y: u32, z: u32) -> Option<D> {
        let mut node_idx = self.root;

        for level in 0..=self.order {
            if level == self.order {
                return self.data[node_idx].set_data(data);
            }

            let idx = Self::index(x, y, z, level, self.order);

            if let Some(child_node_idx) = self.data[node_idx].child(idx) {
                node_idx = child_node_idx;
            } else {
                let child_node_idx = self.allocate_node();
                self.data[node_idx].add_child(idx, child_node_idx);
                node_idx = child_node_idx;
            }
        }

        None
    }

    fn check_bounds(x: u32, y: u32, z: u32, level: u32, order: u32) -> bool {
        debug_assert!(level <= order);

        let voxel_count = N::SUBDIVISION.pow(order);
        debug_assert!(x < voxel_count);
        debug_assert!(y < voxel_count);
        debug_assert!(z < voxel_count);

        true
    }

    /*
     * Return the coordinate of a voxel at a specific level
     */
    pub fn location(x: u32, y: u32, z: u32, level: u32, order: u32) -> (u32, u32, u32) {
        debug_assert!(Self::check_bounds(x, y, z, level, order));

        let voxel_per_level = N::SUBDIVISION.pow(order - level);

        (
            x / voxel_per_level,
            y / voxel_per_level,
            z / voxel_per_level,
        )
    }

    /*
     * Return the child index (0-63) of a voxel at a specific level
     */
    pub fn index(x: u32, y: u32, z: u32, level: u32, order: u32) -> usize {
        debug_assert!(Self::check_bounds(x, y, z, level, order));
        debug_assert!(level < order);

        let voxel_per_level = N::SUBDIVISION.pow(order - level);
        let voxel_per_child = N::SUBDIVISION.pow(order - level - 1);

        let (rel_x, rel_y, rel_z) = (
            (x % voxel_per_level) / voxel_per_child,
            (y % voxel_per_level) / voxel_per_child,
            (z % voxel_per_level) / voxel_per_child,
        );

        (rel_x + N::SUBDIVISION * rel_y + N::SUBDIVISION * N::SUBDIVISION * rel_z) as usize
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
    fn test_insert() {
        let mut tree = Tree::new(0);
        tree.insert(VoxelData, 0, 0, 0);
        assert!(matches!(tree.data[tree.root].data, VoxelChildData::Leaf(_)));

        let mut tree = Tree::new(1);
        tree.insert(VoxelData, 2, 1, 3);
        let idx = Tree::index(2, 1, 3, 0, 1);
        assert!(matches!(
            tree.data[tree.root].data,
            VoxelChildData::Children(_)
        ));
        assert!(tree.data[tree.root].has_child(idx));
        if let VoxelChildData::Children(children) = &tree.data[tree.root].data {
            let child_idx = children[idx];
            assert!(matches!(
                &tree.data[child_idx].data,
                VoxelChildData::Leaf(_)
            ));
        }

        let mut tree = Tree::new(2);
        tree.insert(VoxelData, 0, 0, 0);
        tree.insert(VoxelData, 1, 0, 0);
        assert!(matches!(
            tree.data[tree.root].data,
            VoxelChildData::Children(_)
        ));
        let idx1 = Tree::index(0, 0, 0, 0, 2);
        let idx2 = Tree::index(1, 0, 0, 0, 2);
        assert_eq!(idx1, idx2);
        assert!(tree.data[tree.root].has_child(idx1));
        if let VoxelChildData::Children(children) = &tree.data[tree.root].data {
            let l1 = &tree.data[children[idx1]];
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
            assert!(tree.data[node_idx].has_child(idx));
            node_idx = match &tree.data[node_idx].data {
                VoxelChildData::Children(children) => children[idx],
                _ => panic!("Missing children data, level={}", level),
            };
        }
        assert!(matches!(tree.data[node_idx].data, VoxelChildData::Leaf(_)));

        let mut tree = Tree::new(3);
        tree.insert(VoxelData, 0, 0, 0);
        tree.insert(VoxelData, 63, 63, 63);
        let mut node_idx1 = tree.root;
        let mut node_idx2 = tree.root;
        for level in 0..3 {
            let idx1 = Tree::index(0, 0, 0, level, 3);
            let idx2 = Tree::index(63, 63, 63, level, 3);
            assert_ne!(idx1, idx2);
            assert!(tree.data[node_idx1].has_child(idx1));
            assert!(tree.data[node_idx2].has_child(idx2));
            node_idx1 = match &tree.data[node_idx1].data {
                VoxelChildData::Children(children) => children[idx1],
                _ => panic!("Missing children data, level={}", level),
            };
            node_idx2 = match &tree.data[node_idx2].data {
                VoxelChildData::Children(children) => children[idx2],
                _ => panic!("Missing children data, level={}", level),
            };
        }
    }
}
