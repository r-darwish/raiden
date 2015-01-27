pub use self::split::split;
pub use self::merge::merge;

const CHUNK_SIZE: usize = 4;
type Chunk = [u8; CHUNK_SIZE];

fn create_chunks_vec(disks: usize) -> Vec<Chunk> {
    range(0, disks).map(|_| [0; CHUNK_SIZE]).collect()
}

mod split;
mod merge;