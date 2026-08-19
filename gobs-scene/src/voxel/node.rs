pub trait VoxelNode<D> {
    const SUBDIVISION: u32;
    const CHILDREN_NUMBER: usize = Self::SUBDIVISION.pow(3) as usize;

    fn new() -> Self;
    fn has_children(&self) -> bool;
    fn has_child(&self, idx: usize) -> bool;
    fn add_child(&mut self, idx: usize, node_idx: usize);
    fn child(&self, idx: usize) -> Option<usize>;
    fn set_data(&mut self, data: D) -> Option<D>;
    fn get_data(&self) -> Option<&D>;
    fn set_dirty(&mut self, dirty: bool);
    fn is_dirty(&self) -> bool;
}

pub enum VoxelChildData<D> {
    Empty,
    Leaf(D),
    Children(Vec<usize>),
}

pub struct VoxelNode64<D> {
    childmask: u64,
    pub(crate) data: VoxelChildData<D>,
    dirty: bool,
}

// each voxel is subdivided in 4x4x4 sections
impl<D> VoxelNode<D> for VoxelNode64<D> {
    const SUBDIVISION: u32 = 4;

    fn new() -> Self {
        const {
            debug_assert!(Self::CHILDREN_NUMBER <= 64);
        }

        Self {
            childmask: 0,
            data: VoxelChildData::Empty,
            dirty: false,
        }
    }

    fn has_children(&self) -> bool {
        self.childmask != 0
    }

    fn has_child(&self, idx: usize) -> bool {
        debug_assert!(idx < Self::CHILDREN_NUMBER);

        self.childmask & (1 << idx) != 0
    }

    fn add_child(&mut self, idx: usize, node_idx: usize) {
        debug_assert!(idx < Self::CHILDREN_NUMBER);
        debug_assert!(!self.has_child(idx));

        self.childmask |= 1 << idx;

        match &mut self.data {
            VoxelChildData::Empty => {
                let mut children = vec![0; Self::CHILDREN_NUMBER];
                children[idx] = node_idx;
                self.data = VoxelChildData::Children(children);
            }
            VoxelChildData::Children(children) => {
                debug_assert!(children[idx] == 0);
                children[idx] = node_idx;
            }
            VoxelChildData::Leaf(_) => todo!(),
        }
    }

    fn child(&self, idx: usize) -> Option<usize> {
        if self.has_child(idx)
            && let VoxelChildData::Children(children) = &self.data
        {
            Some(children[idx])
        } else {
            None
        }
    }

    fn set_data(&mut self, data: D) -> Option<D> {
        match std::mem::replace(&mut self.data, VoxelChildData::Leaf(data)) {
            VoxelChildData::Empty => None,
            VoxelChildData::Leaf(data) => Some(data),
            VoxelChildData::Children(_) => panic!("Replacing children node with leaf"),
        }
    }

    fn get_data(&self) -> Option<&D> {
        if let VoxelChildData::Leaf(data) = &self.data {
            Some(data)
        } else {
            None
        }
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }
}
