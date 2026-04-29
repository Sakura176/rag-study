#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub content: String,
    pub metadata: ChunkMetadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkMetadata {
    pub source: String,
    pub page: Option<u32>,
    pub chunk_index: usize,
}

impl ChunkMetadata {
    pub fn new() -> Self {
        Self {
            source: String::new(),
            page: None,
            chunk_index: 0,
        }
    }
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            metadata: ChunkMetadata::new(),
        }
    }
}
