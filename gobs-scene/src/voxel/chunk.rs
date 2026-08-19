use std::collections::HashMap;

use crate::voxel::{map::VoxelTree, node::VoxelNode64};

pub type Tree<D> = VoxelTree<D, VoxelNode64<D>>;

pub struct Chunks<D> {
    chunks: HashMap<[i32; 3], Tree<D>>,
    order: u32,
    chunk_size: u32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VoxelPos {
    pub chunk: [i32; 3],
    pub local: [u32; 3],
}

impl<D> Chunks<D> {
    pub fn new(order: u32) -> Self {
        let chunk_size = Tree::<D>::size(order);
        Self {
            chunks: HashMap::new(),
            order,
            chunk_size,
        }
    }

    pub fn chunks(&self) -> impl Iterator<Item = [i32; 3]> {
        self.chunks.keys().copied()
    }

    pub fn get(&self, pos: [i32; 3]) -> Option<&Tree<D>> {
        self.chunks.get(&pos)
    }

    pub fn get_mut(&mut self, pos: [i32; 3]) -> Option<&mut Tree<D>> {
        self.chunks.get_mut(&pos)
    }

    pub fn get_or_create(&mut self, pos: [i32; 3]) -> &mut Tree<D> {
        self.chunks
            .entry(pos)
            .or_insert_with(|| Tree::new(self.order))
    }

    pub fn world_to_chunk(&self, world_pos: [i64; 3]) -> VoxelPos {
        let chunk_size = self.chunk_size as i64;

        let chunk = [
            world_pos[0].div_euclid(chunk_size) as i32,
            world_pos[1].div_euclid(chunk_size) as i32,
            world_pos[2].div_euclid(chunk_size) as i32,
        ];

        let local = [
            world_pos[0].rem_euclid(chunk_size) as u32,
            world_pos[1].rem_euclid(chunk_size) as u32,
            world_pos[2].rem_euclid(chunk_size) as u32,
        ];

        VoxelPos { chunk, local }
    }
}

#[cfg(test)]
mod tests {
    use tracing::Level;
    use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

    use crate::voxel::chunk::{Chunks, VoxelPos};

    struct VoxelData;

    fn setup() {
        let sub = FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .with_span_events(FmtSpan::CLOSE)
            .finish();
        tracing::subscriber::set_global_default(sub).unwrap_or_default();
    }

    #[test]
    fn test_coord() {
        setup();

        let chunks = Chunks::<VoxelData>::new(2);

        assert_eq!(
            chunks.world_to_chunk([0, 0, 0]),
            VoxelPos {
                chunk: [0, 0, 0],
                local: [0, 0, 0]
            }
        );

        assert_eq!(
            chunks.world_to_chunk([15, 15, 15]),
            VoxelPos {
                chunk: [0, 0, 0],
                local: [15, 15, 15]
            }
        );

        assert_eq!(
            chunks.world_to_chunk([16, 16, 16]),
            VoxelPos {
                chunk: [1, 1, 1],
                local: [0, 0, 0]
            }
        );

        assert_eq!(
            chunks.world_to_chunk([-1, -1, -1]),
            VoxelPos {
                chunk: [-1, -1, -1],
                local: [15, 15, 15]
            }
        );

        assert_eq!(
            chunks.world_to_chunk([-16, -16, -16]),
            VoxelPos {
                chunk: [-1, -1, -1],
                local: [0, 0, 0]
            }
        );

        assert_eq!(
            chunks.world_to_chunk([-17, -17, -17]),
            VoxelPos {
                chunk: [-2, -2, -2],
                local: [15, 15, 15]
            }
        );

        assert_eq!(
            chunks.world_to_chunk([-1, 0, 0]),
            VoxelPos {
                chunk: [-1, 0, 0],
                local: [15, 0, 0]
            }
        );

        assert_eq!(
            chunks.world_to_chunk([-1, 0, 15]),
            VoxelPos {
                chunk: [-1, 0, 0],
                local: [15, 0, 15]
            }
        );
    }
}
