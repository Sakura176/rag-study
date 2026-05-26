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
            separators: vec![
                "\n\n".to_string(),
                "\n".to_string(),
                "。".to_string(),
                "！".to_string(),
                "？".to_string(),
                "；".to_string(),
                "，".to_string(),
            ],
        }
    }

    pub fn split(&self, chunks: &[Chunk]) -> Vec<Chunk> {
        let mut result = Vec::new();
        for chunk in chunks {
            let mut start = 0;
            while start < chunk.content.len() {
                let mut end = std::cmp::min(start + self.chunk_size, chunk.content.len());

                // 如果还没到末尾，尝试在最近的分隔符处断开
                if end < chunk.content.len()
                    && let Some(boundary) = self.find_split_boundary(&chunk.content, start, end)
                {
                    end = boundary;
                }

                let mut new_chunk = Chunk::new();
                new_chunk.content = chunk.content[start..end].trim().to_string();
                new_chunk.metadata = chunk.metadata.clone();
                new_chunk.metadata.chunk_index = result.len();
                if !new_chunk.content.is_empty() {
                    result.push(new_chunk);
                }

                let next_start = end.saturating_sub(self.chunk_overlap);
                if next_start <= start {
                    start = end;
                } else {
                    start = next_start;
                }
            }
        }
        result
    }

    /// 在 [start, end) 范围内查找最后一个分隔符的位置
    fn find_split_boundary(&self, content: &str, start: usize, end: usize) -> Option<usize> {
        let search_region = &content[start..end];
        let mut best_pos = None;

        for sep in &self.separators {
            if let Some(pos) = search_region.rfind(sep.as_str()) {
                let abs_pos = start + pos + sep.len();
                if best_pos.is_none_or(|p| abs_pos > p) {
                    best_pos = Some(abs_pos);
                }
            }
        }

        best_pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Chunk, ChunkMetadata};

    fn make_chunk(text: &str, page: u32) -> Chunk {
        Chunk {
            content: text.to_string(),
            metadata: ChunkMetadata {
                source: String::new(),
                page: Some(page),
                chunk_index: 0,
            },
        }
    }

    #[test]
    fn test_split_smaller_than_chunk_size() {
        let splitter = Splitter::new(100, 0);
        let chunks = vec![make_chunk("Hello, world!", 1)];
        let result = splitter.split(&chunks);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "Hello, world!");
    }

    #[test]
    fn test_split_larger_than_chunk_size() {
        let splitter = Splitter::new(5, 0);
        let chunks = vec![make_chunk("abcdefghij", 1)];
        let result = splitter.split(&chunks);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].content, "abcde");
        assert_eq!(result[1].content, "fghij");
    }

    #[test]
    fn test_split_with_overlap() {
        let splitter = Splitter::new(6, 2);
        let chunks = vec![make_chunk("abcdefghijklm", 1)];
        let result = splitter.split(&chunks);

        assert!(result.len() >= 3);
        for chunk in &result {
            assert!(chunk.content.len() <= 6);
        }
    }

    #[test]
    fn test_empty_content() {
        let splitter = Splitter::new(10, 0);
        let chunks = vec![make_chunk("", 1)];
        let result = splitter.split(&chunks);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_chunk_index_increments() {
        let splitter = Splitter::new(5, 0);
        let chunks = vec![make_chunk("abcdefghij", 1)];
        let result = splitter.split(&chunks);

        for (i, chunk) in result.iter().enumerate() {
            assert_eq!(chunk.metadata.chunk_index, i);
        }
    }

    #[test]
    fn test_split_with_separator_boundary() {
        // 中文每字符 3 字节，chunk_size 需要足够大才能命中分隔符
        let splitter = Splitter::new(30, 0);
        let text = "今天天气很好。我们一起去公园散步。明天还要上班。";
        let chunks = vec![make_chunk(text, 1)];
        let result = splitter.split(&chunks);

        assert!(result.len() >= 2);
        // 至少有一个非末尾 chunk 在分隔符处断开
        let has_sep_boundary = result[..result.len() - 1].iter().any(|c| {
            c.content.ends_with("。")
                || c.content.ends_with("！")
                || c.content.ends_with("？")
                || c.content.ends_with("；")
                || c.content.ends_with("，")
        });
        assert!(has_sep_boundary);
    }
}
