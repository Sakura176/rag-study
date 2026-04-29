use crate::types::Chunk;

pub struct Splitter {
    chunk_size: usize,
    chunk_overlap: usize,
    separators: Vec<String>,
}

impl Splitter {
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
            separators: Vec::new(),
        }
    }
    pub fn split(&self, chunks: &[Chunk]) -> Vec<Chunk> {}
}
