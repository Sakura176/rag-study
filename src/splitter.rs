use crate::types::Chunk;

pub struct Splitter {
    chunk_size: usize,
    chunk_overlap: usize,
    separators: Vec<String>,
}

impl Splitter {
    // Creates a new Splitter with the specified chunk size and overlap.
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
            separators: Vec::new(),
        }
    }

    // Splits the chunks into smaller chunks based on the chunk size and overlap.
    pub fn split(&self, chunks: &[Chunk]) -> Vec<Chunk> {
        let mut result = Vec::new();
        for chunk in chunks {
            let mut current_chunk = chunk.clone();
            while current_chunk.content.len() > self.chunk_size {
                let split_index = current_chunk.content.len().saturating_sub(self.chunk_size);
                let split_chunk = current_chunk
                    .content
                    .drain(split_index..)
                    .collect::<String>();
                let mut new_chunk = Chunk::new();
                new_chunk.content = split_chunk;
                new_chunk.metadata.page = chunk.metadata.page;
                result.push(new_chunk);

                current_chunk.content = current_chunk
                    .content
                    .drain(..split_index)
                    .collect::<String>();
            }
            if !current_chunk.content.is_empty() {
                result.push(current_chunk);
            }
        }
        result
    }
}
