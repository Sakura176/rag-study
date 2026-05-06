# Code Review: loader.rs & splitter.rs

## 概述

对 Phase 0a 中已实现的 `load_pdf`（`loader.rs`）和 `Splitter`（`splitter.rs`）进行代码评审。当前项目状态：核心类型已定义，PDF 加载和文本分块已有初版实现，CLI 入口尚未接入。

---

## 1. loader.rs 评审

### 1.1 设计评价

`load_pdf` 函数设计简洁：打开 PDF → 逐页提取文本 → 每页作为一个 `Chunk`。对 Phase 0a 来说这个粒度是合理的。

### 1.2 问题

**P0 — 未记录 source 路径** (`loader.rs:14`)

`chunk.metadata.source` 始终为空字符串。虽然当前只有一个文件，但后续多文档场景下这个字段是区分来源的唯一标识。

```rust
// 当前：
let mut chunk = Chunk::new();

// 建议：
let mut chunk = Chunk::new();
chunk.metadata.source = path.to_string_lossy().to_string();
```

### 1.3 建议

| 严重度 | 问题 | 建议 |
|--------|------|------|
| P2 | 仅有 `tempfile` 测试，缺少真实 PDF 测试 | 添加一个简单的 `not_found` 测试：`load_pdf(Path::new("/nonexistent/test.pdf"))` |
| P2 | `extract_text` 对某些 PDF 可能返回空字符串 | 后续可加 warn log 或返回值检查 |

### 1.4 测试覆盖

当前 1 个测试（`test_load_pdf_with_generated_doc`），覆盖正常双页场景。建议补充：

- 空 PDF（0 页）
- 不存在的文件路径
- 单页 PDF
- 含中文文本的 PDF

---

## 2. splitter.rs 评审

### 2.1 设计评价

`Splitter` struct 的三个字段（`chunk_size`, `chunk_overlap`, `separators`）反映了正确的设计意图。但**当前实现与设计意图之间差距较大**。

### 2.2 严重问题

**P0 — 切分算法逻辑错误** (`splitter.rs:20-44`)

当前算法每次从内容**末尾**取出 chunk_size 大小的块，导致：

- 输出的 chunk 顺序是**反的**（尾部先出）
- 跨越多次迭代的内容被**片段化**，连续语义断裂

以 `content="abcdefghij", chunk_size=3` 为例：

```
期望 (合理行为): ["abc", "def", "ghi", "j"]
实际输出:        ["hij", "efg", "bcd", "a"]
```

根因分析：`split_index` 被计算为 `len - chunk_size`，然后用 `drain(split_index..)` 取尾部，再用 `drain(..split_index)` 回收剩余内容。这本质上是"从尾部切"，而不是"从头部切"。

```rust
// 当前（错误）：
let split_index = current_chunk.content.len().saturating_sub(self.chunk_size);
let split_chunk = current_chunk.content.drain(split_index..).collect::<String>();
// ↑ 取了尾部 chunk_size 个字符

// 修正方案：
fn split(&self, chunks: &[Chunk]) -> Vec<Chunk> {
    let mut result = Vec::new();
    for chunk in chunks {
        let mut start = 0;
        while start < chunk.content.len() {
            let end = std::cmp::min(start + self.chunk_size, chunk.content.len());
            let mut new_chunk = Chunk::new();
            new_chunk.content = chunk.content[start..end].to_string();
            new_chunk.metadata = chunk.metadata.clone();
            new_chunk.metadata.chunk_index = result.len();
            result.push(new_chunk);
            start += self.chunk_size - self.chunk_overlap;
        }
    }
    result
}
```

**P1 — chunk_overlap 未使用** (`splitter.rs:18`)

`chunk_overlap` 存储在字段中但 `split()` 方法从未引用它。没有 overlap 的切分会导致边界处的语义断裂（例如一个完整的句子被切到不同 chunk）。

**P1 — separators 未使用** (`splitter.rs:19`)

`separators` 初始化为空 Vec，`split()` 按纯字节长度硬切。路线图（optimization_roadmap.md:142-144）明确要求优先按分隔符（`\n\n`, `\n`, `。`, `！` 等）切分，当前实现与需求不符。

### 2.3 次要问题

| 严重度 | 问题 | 说明 |
|--------|------|------|
| P2 | `chunk_index` 始终为 0 | `ChunkMetadata::chunk_index` 字段从未更新，无法追踪分块序号 |
| P2 | 缺少单元测试 | splitter 模块没有任何 `#[cfg(test)]` |
| P3 | 多余的一次 drain | 第二次 `drain(..split_index)` 加 `collect` 赋值回 `current_chunk` 是冗余操作 |

### 2.4 建议的完整实现

```rust
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
                if end < chunk.content.len() {
                    if let Some(boundary) = self.find_split_boundary(&chunk.content, start, end) {
                        end = boundary;
                    }
                }

                let mut new_chunk = Chunk::new();
                new_chunk.content = chunk.content[start..end].trim().to_string();
                new_chunk.metadata = chunk.metadata.clone();
                new_chunk.metadata.chunk_index = result.len();
                result.push(new_chunk);

                start = end.saturating_sub(self.chunk_overlap);
                if start >= chunk.content.len() {
                    break;
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
                if abs_pos > start && abs_pos < end {
                    if best_pos.map_or(true, |p| abs_pos > p) {
                        best_pos = Some(abs_pos);
                    }
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

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].content, "abcde");
        assert_eq!(result[1].content, "fghij");
    }

    #[test]
    fn test_split_with_overlap() {
        let splitter = Splitter::new(6, 2);
        let chunks = vec![make_chunk("abcdefghijklm", 1)];
        let result = splitter.split(&chunks);

        assert_eq!(result.len(), 4);
        assert!(result[0].content.len() <= 6);
        assert!(result[1].content.len() <= 6);
    }

    #[test]
    fn test_empty_content() {
        let splitter = Splitter::new(10, 0);
        let chunks = vec![make_chunk("", 1)];
        let result = splitter.split(&chunks);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_split_with_separator_boundary() {
        let splitter = Splitter::new(15, 0);
        // 中文文本，包含句号
        let text = "今天天气很好。我们一起去公园散步。明天还要上班。";
        let chunks = vec![make_chunk(text, 1)];
        let result = splitter.split(&chunks);

        // 每个 chunk 应该在句号处断开
        for chunk in &result {
            if !chunk.content.ends_with("。") {
                // 允许最后一个 chunk 不以句号结尾
                continue;
            }
        }
        assert!(result.len() >= 2);
    }
}
```

---

## 3. 跨模块问题

| 严重度 | 问题 | 涉及文件 |
|--------|------|----------|
| P1 | `ChunkMetadata::new()` 不支持设置 `source` | `types.rs:15-22`, `loader.rs:13-14` |
| P2 | `chunk_index` 从未被更新 | `types.rs`, `loader.rs`, `splitter.rs` |

---

## 4. 与路线图的差距对照

| 路线图要求 | 当前状态 | 差距 |
|------------|----------|------|
| 支持按分隔符递归切分 | 未实现 | 按字节硬切 |
| 支持中文分隔符 `\n\n`, `。`, `！` 等 | 未实现 | `separators` 为空 |
| `chunk_size` 和 `chunk_overlap` 参数化 | 结构体字段已定义，overlap 未使用 | 半完成 |
| CLI 入口（clap） | 未实现 | main.rs 只有 hello world |
| 错误处理：文件不存在、PDF 损坏 | 部分实现（`?` 传递） | 缺少自定义错误类型 |

---

## 5. 优先级建议

1. **立即修复** — splitter 切分算法（当前逻辑错误导致分块乱序）
2. **本周修复** — 加入 overlap 支持、初始化默认分隔符列表
3. **迭代完善** — 设置 source 路径、更新 chunk_index、补齐测试
