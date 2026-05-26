#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub content: String,
    pub metadata: ChunkMetadata,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChunkMetadata {
    pub source: String,
    pub page: Option<u32>,
    pub chunk_index: usize,
}

impl ChunkMetadata {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            content: String::new(),
            metadata: ChunkMetadata::new(),
        }
    }
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }
}
