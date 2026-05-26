# RAG 学习项目 — Rust 版优化路线图

> 面向 C++ 工程师。目标：用 RAG 项目做载体，学会 Rust 基础到进阶。

---

## 你的 C++ 背景如何加速 Rust 学习

Rust 的设计者们是从 C++ 的痛点出发来设计语言的。你已经掌握的东西不用重新学：

| C++ 概念 | Rust 对应 | 差异 |
|---------|----------|------|
| RAII / 析构函数 | `Drop` trait | 行为一致，Rust 多了编译器保证的所有权 |
| move 语义 | 默认 move（C++ 是默认 copy） | 方向相同但 Rust 的 move 是**破坏性的**——原变量失效 |
| 智能指针 | `Box<T>`, `Rc<T>`, `Arc<T>` | 行为相似，但 Rust 多了 `borrow checker` |
| `const` 正确性 | 默认 immutable，`mut` 声明可变 | Rust 更激进，默认只读 |
| 模板 `<typename T>` | 泛型 `<T>` + trait bound | 关键区别：Rust 在**定义时**做类型检查（而非实例化时），错误信息清晰得多 |
| 虚函数 / 多态 | `dyn Trait` 或 enum | 鼓励 enum 模式多于继承 |
| `nullptr` | `Option<T>` | Rust 没有 null，编译器强制处理缺失情况 |
| STL 容器 | `Vec`, `HashMap`, `BTreeMap` | 接口高度相似 |

**真正的学习难点只有三个**：借用检查器（无 C++ 对应物）、生命周期标注、trait 系统（比 C++ concept 更强但思维不同）。这三个是 Rust 的独特发明，之前的 C++ 经验帮不上忙，需要刻意练习。

---

## 前置阅读（立即开始，与 Phase 0 并行）

在写任何 RAG 代码之前，先过一遍 Rust 语法。不需要精读，**扫一遍建立印象即可**，后续编码时反复回来查。

| 优先级 | 资源 | 章节 |
|--------|------|------|
| ⭐必读 | [The Rust Book](https://doc.rust-lang.org/book/) | 第 1~6 章（所有权之前都简单） |
| ⭐必读 | 同上 | 第 10 章：泛型 + Trait + 生命周期 |
| ⭐必读 | 同上 | 第 15 章：智能指针 |
| ⭐必读 | 同上 | 第 16 章：并发 |
| 推荐 | [Rust by Example](https://doc.rust-lang.org/rust-by-example/) | 边看边跑，每个例子都 `cargo run` |
| 推荐 | [Rustlings](https://github.com/rust-lang/rustlings) | 小型编码练习，非常适合碎片时间 |

---

## 整体路线图（修订版）

```
Phase 0a — 同步骨架（3 天）
Phase 0b — 异步 + API（3 天）          ← 从这里开始才算真正的 pipeline
Phase 1 — Token 分词 + BM25（1 周）
Phase 2 — Embedding + 向量检索（1 周）
Phase 3 — LLM + Reranker + RAG 完整闭环（1 周）
Phase 4 — 并发 + 流式 + 持久化（1 周）
Phase 5 — 从零实现 HNSW（可选，深度挑战）
```

每个 Phase 的设计原则：
- **一个 Phase，只加 1~2 个 Rust 新概念**
- **先同步再异步**：Phase 0a 全部同步函数，Phase 0b 才引入 async
- **先内存再外部**：先在自己进程里跑 BM25、余弦相似度，确保概念正确，再接入外部 API
- **每个 Phase 有可运行的二进制**：不累积到最后

---

## Phase 0a — Rust 基础 + 文档解析 + 分块（3 天）

**目标**：用纯同步 Rust 实现 PDF 读取 + 文本切分，不涉及异步、不涉及网络、不涉及 LLM。

**Rust 学习焦点**：所有权、借用、struct、impl、Vec 操作、Result、模块系统。

### 0a.1 脚手架

```bash
cargo new rag-study
cd rag-study
```

项目结构：

```
src/
├── main.rs        # 入口
├── lib.rs         # 公共模块导出
├── types.rs        # Document, Chunk 等 struct
├── loader.rs       # PDF 加载
└── splitter.rs     # 文本分块
```

### 0a.2 依赖（Cargo.toml）

```toml
[package]
name = "rag-study"
version = "0.1.0"
edition = "2021"

[dependencies]
lopdf = "0.34"                            # PDF 解析
clap = { version = "4", features = ["derive"] }
anyhow = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = "0.3"
```

> 注意：Phase 0a 不加 reqwest/tokio/serde，全部用同步代码。

### 0a.3 任务

**Step 1 — 定义核心类型** (`types.rs`)
```rust
/// 一个文档分块
#[derive(Debug, Clone)]
pub struct Chunk {
    pub content: String,
    pub metadata: ChunkMetadata,
}

#[derive(Debug, Clone)]
pub struct ChunkMetadata {
    pub source: String,       // 原文件路径
    pub page: Option<usize>,  // 页码（从 1 开始）
    pub chunk_index: usize,   // 在文档中的序号
}
```

- [x] 实现 `Chunk::new()` 构造函数
- [x] 实现 `Default` trait

> 💡 **C++ 视角**：`derive(Debug)` ≈ 自动生成 `operator<<(ostream&)`，`derive(Clone)` ≈ 自动生成拷贝构造。Rust 不让隐式拷贝，必须显式 `.clone()`。

**Step 2 — PDF 加载** (`loader.rs`)
- [x] 用 `lopdf` 打开 PDF，逐页提取文本
- [x] 返回 `Vec<Chunk>`，每页一个 chunk，metadata 中记录页码和 source 路径
- [x] 错误处理：文件不存在通过 `?` 传播错误

```rust
pub fn load_pdf(path: &Path) -> anyhow::Result<Vec<Chunk>> {
    let doc = lopdf::Document::load(path)?;  // '?' ≈ C++ 中 try/catch 的语法糖
    // ...
}
```

> 💡 **C++ 视角**：`Result<T, E>` 替代了异常 + 错误码的两难选择。`?` 操作符让你像写异常代码一样处理错误但无运行时开销。`anyhow::Result<T>` ≈ `std::expected<T, any_error>`。

**Step 3 — 文本分块** (`splitter.rs`)
- [x] 实现按分隔符优先级切分（在 chunk_size 边界查找最近分隔符）
- [x] 支持中文分隔符 `\n\n`, `\n`, `。`, `！`, `？`, `；`, `，`
- [x] 参数化 chunk_size 和 chunk_overlap（overlap 实现重叠滑动窗口）
- [x] chunk_index 递增赋值
- [x] 6 个单元测试

```rust
pub struct Splitter {
    chunk_size: usize,
    chunk_overlap: usize,
    separators: Vec<String>,
}

impl Splitter {
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self { /* ... */ }
    pub fn split(&self, chunks: &[Chunk]) -> Vec<Chunk> { /* ... */ }
}
```

> 💡 **C++ 视角**：`self` ≈ `this`，但 Rust 的函数定义和 impl 块是分离的 —— 不像 C++ 在类内声明。`&[Chunk]` ≈ `std::span<const Chunk>`。

**Step 4 — CLI 入口** (`main.rs`)
- [x] 用 clap 定义参数：`--file` / `--chunk-size` / `--chunk-overlap`
- [x] 调用 loader → splitter → 打印 chunk 统计

```bash
cargo run -- --file data/test.pdf --chunk-size 500 --chunk-overlap 50
# 输出：加载了 12 页，切分为 37 个 chunk
```

### Phase 0a 自测

- [x] 能否用 Rust 实现一个递归函数（splitter 的核心逻辑），而不触发借用错误？
- [x] `&str` 和 `String` 的区别是什么？什么时候用哪个？→ 类比 `std::string_view` vs `std::string`
- [x] `Result` 的 `?` 操作符和 C++ 的异常有何异同？

### Phase 0a 完成总结

完成了同步版本的文档加载与切分 pipeline：`load_pdf → Splitter::split → 打印统计`。包括：

- 通过 lopdf 逐页提取 PDF 文本，记录 source 路径和页码
- Splitter 支持按分隔符优先级在 chunk_size 边界附近智能断句，overlap 实现分块间重叠
- CLI 通过 clap 参数化，可指定文件、chunk_size、chunk_overlap
- 8 个单元测试覆盖正常路径和边界情况
- 全部同步代码，未引入 tokio/reqwest

---

## Phase 0b — 异步 HTTP + API 调用（3 天）

**目标**：接入 DeepSeek API，实现"发送一段文本，获取 LLM 回答"和"获取文本的 embedding 向量"。仍然没有 RAG，先让 API 调通。

**Rust 学习焦点**：async/await、serde、reqwest、模块可见性（pub）。

### 0b.1 新增依赖

```toml
[dependencies]
# ... 保留 Phase 0a 的依赖 ...

reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dotenv = "0.15"
```

### 0b.2 项目结构（新增文件）

```
src/
├── ...
├── llm.rs          # DeepSeek API 调用
└── embeddings.rs   # Embedding API 调用
```

### 0b.3 任务

**Step 1 — serde 是什么？**

先单独练习。新建一个小文件 `tests/serde_demo.rs`，把 Rust struct 序列化为 JSON、把 API 返回的 JSON 反序列化为 Rust struct。

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}
```

> 💡 **C++ 视角**：如果用过 nlohmann/json 就能立刻理解 serde。serde 的优势是**编译期**生成序列化代码，无反射开销。`derive(Serialize)` ≈ 编译器自动写 `to_json()`。

**Step 2 — 调通 LLM API** (`llm.rs`)
- [ ] 用 `reqwest` 发 POST 请求到 `https://api.deepseek.com/v1/chat/completions`
- [ ] 从环境变量读 API key
- [ ] 请求 + 响应的 struct 定义
- [ ] 打印返回的完整 JSON 响应

**Step 3 — 调通 Embedding API** (`embeddings.rs`)
- [ ] 调 `https://api.deepseek.com/v1/embeddings`
- [ ] 返回 `Vec<f32>`（通常是 1024 或 1536 维）

**Step 4 — 改为 async main**
```rust
#[tokio::main]  // 替代 fn main()
async fn main() -> anyhow::Result<()> {
    // ...
}
```

> 💡 **C++ 视角**：`#[tokio::main]` 是 procedural macro，在编译期展开为你启动 tokio runtime + 管理 event loop。C++ 中你需要手动写 `co_await` 和 `io_context.run()`，Rust 的 macro 替你干了。

### Phase 0b 自测

- [ ] `async fn` 和普通 `fn` 的区别？`Future` trait 和 C++20 的 `std::future` 有什么不同？
- [ ] 为什么 Rust 的 `await` 是一个操作符（`.await`）而不是关键字？这和 Rust 的类型系统有什么关系？
- [ ] 能写出一个带 serde 反序列化的 API 调用，从编译失败到跑通吗？

---

## Phase 1 — BM25 全文检索（1 周）

**目标**：在自己进程里跑全文检索，不依赖任何外部 API。理解"检索"是什么，以及如何评估检索质量。

为什么先做 BM25 再做向量检索？
- BM25 不需要 embedding API，不消耗 token，不花钱
- 你可以在本地无限调优、反复实验
- 先建立检索概念的基线，之后和向量检索做对比才有意义
- BM25 的算法实现是纯 CPU 计算，C++ 工程师有优势

**Rust 学习焦点**：闭包、迭代器链、`std::collections::HashMap`、单元测试。

### 1.1 分词

- [ ] 实现简单的中文分词（可用 `unicode-segmentation` crate 做字形分割，或直接用 `char_indices` 按字符滑动窗口）
- [ ] 支持 n-gram（1-gram 到 3-gram）
- [ ] 拆成 `Tokenizer` trait，为后续替换留接口

```rust
pub trait Tokenizer {
    fn tokenize(&self, text: &str) -> Vec<String>;
}

pub struct BigramTokenizer;  // 二元组分词器
pub struct UnicodeSegmenter; // 更复杂的分词
```

> 💡 **C++ 视角**：trait ≈ C++ concept（编译期约束），但 trait 还可以做运行时分发（`dyn Trait` ≈ 虚函数表）。这是 Rust trait 比 C++ concept 更强大的地方。

### 1.2 BM25 实现

BM25 公式很简单，但实现涉及到：
- 统计词频 (TF) 和文档频率 (DF)
- 计算 IDF 权重
- 按分数排序结果

```rust
pub struct Bm25 {
    // 索引：word -> [(doc_id, term_frequency)]
    inverted_index: HashMap<String, Vec<(usize, usize)>>,
    avg_doc_len: f64,
    total_docs: usize,
    k1: f64,  // term frequency saturation
    b: f64,   // length normalization
}
```

- [ ] 实现 `Bm25::index(chunks: &[Chunk])`
- [ ] 实现 `Bm25::search(query: &str, k: usize) -> Vec<(usize, f64)>`

> 💡 **C++ 视角**：如果你写过倒排索引，这就是一样的。Rust 的 `HashMap` + 所有权会让你对"谁拥有索引数据"有新的理解。

### 1.3 第一个测试

- [ ] 写单元测试：给 3 个文档，搜索"智能指针"，验证 `shared_ptr` 的文档排第一
- [ ] 这是 Rust 测试框架的入门：`#[cfg(test)] mod tests {}`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bm25_search_basic() {
        // 构造 3 个 chunk
        // 搜索 "智能指针"
        // 断言 shared_ptr 相关的文档排名最高
    }
}
```

### Phase 1 交付物
```bash
cargo run -- --file data/test.pdf search bm25 --query "智能指针"
# 输出：BM25 检索到 5 个相关 chunk，最高分 2.34
```

- [ ] 同步版本的完整 pipeline：`load → split → bm25_search → 打印结果`
- [ ] 至少 3 个单元测试
- [ ] BM25 检索质量可自己用肉眼评估（搜几个术语，看结果是否合理）

---

## Phase 2 — Embedding + 向量检索（1 周）

**目标**：在 Phase 1 基础上加入 embedding 调用和向量相似度检索，最后实现 BM25+向量的混合检索。

**Rust 学习焦点**：泛型约束、trait bound、`Copy` vs `Clone`、`Add`/`Mul` 运算符重载。

### 2.1 向量运算（不引入 ndarray，自己写）

这是一个极好的 Rust 练习。C++ 中你可能用 Eigen/Blaze，这里自己写 200 行。

```rust
#[derive(Debug, Clone)]
pub struct Vector(Vec<f32>);

impl Vector {
    pub fn dot(&self, other: &Vector) -> f32 { /* 内积 */ }
    pub fn norm(&self) -> f32 { /* L2 范数 */ }
    pub fn cosine_similarity(&self, other: &Vector) -> f32 {
        self.dot(other) / (self.norm() * other.norm() + f32::EPSILON)
    }
}

// 让 Vector 支持 + 和 * 运算符
impl std::ops::Add for &Vector { /* ... */ }
impl std::ops::Mul<f32> for &Vector { /* ... */ }
```

> 💡 **C++ 视角**：Rust 的运算符重载通过 trait 实现（`Add`、`Mul` 等），比 C++ 的 `operator+` 更内聚——你明确知道哪个 trait 是一组相关运算符。

### 2.2 Embedding 缓存

- [ ] 实现一个内存缓存：`HashMap<String, Vector>`
- [ ] 对 chunk 文本 hash 后查缓存
- [ ] API 调用失败时重试 3 次，指数退避

### 2.3 向量检索器

```rust
pub struct VectorStore {
    vectors: Vec<Vector>,         // 所有 embedding
    chunks: Vec<Chunk>,           // 原始 chunk（与 vectors 一一对应）
    cache: HashMap<String, Vector>,
}

impl VectorStore {
    pub async fn add_chunks(&mut self, chunks: Vec<Chunk>) -> anyhow::Result<()> { ... }
    pub fn similarity_search(&self, query_vec: &Vector, k: usize) -> Vec<(usize, f32)> { ... }
}
```

### 2.4 混合检索 — RRF

```rust
pub struct HybridSearcher {
    bm25: Bm25,
    vector_store: VectorStore,
}

impl HybridSearcher {
    pub fn search(&self, query: &str, query_vec: &Vector, k: usize) -> Vec<(usize, f32)> {
        let bm25_results = self.bm25.search(query, k * 2);
        let vec_results = self.vector_store.similarity_search(query_vec, k * 2);
        Self::rrf_merge(&bm25_results, &vec_results, k, 60.0)
    }

    fn rrf_merge(
        r1: &[(usize, f64)],
        r2: &[(usize, f32)],
        k: usize,
        rrf_k: f64,
    ) -> Vec<(usize, f32)> { /* ... */ }
}
```

### Phase 2 验证标准
```bash
cargo test  # 现有测试全过
cargo run -- search hybrid --query "智能指针"  # 输出混合检索结果
```

- [ ] 对比 BM25-only vs Vector-only vs Hybrid 三者的检索质量

---

## Phase 3 — LLM 检索增强生成 + Reranker（1 周）

**目标**：完成 RAG 闭环。检索到的文档 → 送入 LLM → 生成带引用的答案。

**Rust 学习焦点**：trait object（`dyn Trait`）、async trait（用 `async-trait` crate）、模板引擎。

### 3.1 Prompt 模板

不用引入 tera/handlebars（过度工程了），用 `format!()` 自己拼。

```rust
const QA_PROMPT: &str = r#"根据以下文档内容回答问题。如果文档中没有相关信息，请直接说"文档未提供相关信息"，不要编造。

{context}

问题：{query}
答案（请标注信息来源）："#;
```

这个简单模板通过参数化 `{context}` 和 `{query}` 就已经够用。

### 3.2 Reranker（重排序）

- [ ] 调用 BGE Reranker API（或用 simple 方法：把每对 (query, doc) 喂 LLM 做相关性打分）
- [ ] 流水线：检索 top-12 → Reranker 精排 → 取 top-3 输入 QA prompt

```rust
/// Reranker trait — 切换不同 reranker 实现
pub trait Reranker {
    /// 返回 (original_index, score)，按 score 降序
    fn rerank(&self, query: &str, docs: &[Chunk]) -> Vec<(usize, f32)>;
}

/// 使用 LLM 做 pointwise 相关性评分
pub struct LlmReranker {
    llm: LlmClient,
}

impl Reranker for LlmReranker {
    fn rerank(&self, query: &str, docs: &[Chunk]) -> Vec<(usize, f32)> {
        // 对每个 doc，让 LLM 打分 1~5
    }
}
```

> 💡 **C++ 视角**：`dyn Reranker` 约等于 C++ 的纯虚基类指针。但 Rust 强制你显式标注 `dyn`，提醒你这是有运行时开销的分发。

### 3.3 QA Chain

```rust
pub struct QaChain {
    llm: LlmClient,
    reranker: Option<Box<dyn Reranker>>,
}

impl QaChain {
    pub async fn answer(
        &self,
        query: &str,
        documents: &[Chunk],
    ) -> anyhow::Result<QaResponse> {
        // 1. reranker 精排（如果有）
        // 2. 拼接 prompt
        // 3. 调用 LLM
        // 4. 返回答案 + 来源
    }
}

pub struct QaResponse {
    pub answer: String,
    pub sources: Vec<Chunk>,
}
```

### Phase 3 验证标准
```bash
cargo run -- qa --file data/test.pdf --query "C++智能指针的原理"
# 输出：带来源引用的完整答案
```

---

## Phase 4 — 工程化（1 周）

**目标**：流式输出、并发批量、SQLite 持久化。

**Rust 学习焦点**：tokio 并发原语、Stream trait、Send/Sync trait、Cargo profiles。

### 4.1 流式输出

- [ ] LLM 响应改成 streaming：`stream: true`，按 SSE 解析
- [ ] 用 `tokio::sync::mpsc` 实现生产者-消费者

### 4.2 并发批量

- [ ] 多文档同时 embedding：用 `tokio::task::JoinSet` 管理
- [ ] 用 `tokio::sync::Semaphore` 限流（API rate limit）

### 4.3 SQLite 持久化

- [ ] 用 `rusqlite` 存储 embedding，避免每次重建
- [ ] BLOB 列存 `f32` 序列化
- [ ] 启动时检测缓存有效性（文档改了 → 重新 embedding）

### 4.4 发布优化

```toml
[profile.release]
opt-level = 3      # 等价于 gcc -O3
lto = true          # 链接时优化
codegen-units = 1   # 更好的优化但编译更慢
```

对比 debug vs release 模式下检索延迟。

> 💡 **C++ 视角**：`opt-level` ≈ `-O`，`lto` ≈ `-flto`。这里是 Rust 的舒适区——零成本抽象的承诺意味着 release 模式下的向量运算和 C++ 性能持平。

---

## Phase 5 — 从零实现 HNSW（可选，深度挑战）

这是 Rust 学习的终极挑战。HNSW 是向量检索的经典算法，涉及多层图 + 贪心搜索。

**为什么用 Rust 实现特别有价值**：
- 多级跳表状的图节点之间的**所有权关系**（谁拥有邻居节点的引用？`Rc<RefCell<>>`？unsafe 指针？）
- 这迫使你在 "safe Rust 的约束" 和 "性能需求" 之间做权衡
- 实现过一遍后，你对 Rust 的理解会上一个台阶

**建议分步实现**：

1. 先实现**单层 NSW 图**（Navigable Small World）
2. 再加分层逻辑（Hierarchical NSW）
3. 最后加启发式连接选择（heuristic neighbor selection）

**参考实现**：
- C++ 版：[hnswlib](https://github.com/nmslib/hnswlib) — 你读 C++ 代码很快，看完算法再翻译成 Rust
- Rust 版：[instant-distance](https://github.com/InstantDomain/instant-distance) — 可以对照学习 safe/unsafe 的选择

> 💡 你也可以用 `unsafe` 实现一个"裸指针"版本（像 C++ 一样），然后逐段重写为 safe Rust。这种对比学习对你的 C++ 背景来说事半功倍。

---

## 每个 Phase 的 Rust 概念递进

| Phase | 新增 Rust 概念 | 核心练习 |
|-------|--------------|---------|
| **0a** | 所有权、借用、`&str` vs `String`、`Result`、struct/impl | splitter 递归函数的借用 |
| **0b** | async/await、serde、`#[tokio::main]`、`mod` 可见性 | API JSON 反序列化 |
| **1** | 闭包、迭代器链、`HashMap`、单元测试 `#[test]` | BM25 的倒排索引构建 |
| **2** | trait + trait bound、运算符重载、泛型约束 | `Vector` 的自定义类型 |
| **3** | `dyn Trait`（trait object）、`Box<>`、async trait | Reranker 的多态选择 |
| **4** | `Send/Sync`、`Stream`、`Semaphore`、`JoinSet` | 并发 embedding 的限流 |
| **5** | `unsafe`、裸指针、`Rc<RefCell<>>`、unsafe 到 safe 的渐进 | HNSW 图的所有权模型 |

---

## 关键 C++ → Rust 陷阱

看 C++ 工程师踩过的坑，提前回避：

### 陷阱 1：以为 `.clone()` 很慢

Rust 的 `.clone()` 就是深拷贝，不像 C++ 编译器可能 RVO 掉。当你看到 `.clone()` 先问自己：这里能不能用引用 `&`？

### 陷阱 2：借用的传染性

```rust
let data = vec![1, 2, 3];
let slice = &data[..];
data.push(4);  // ❌ 编译错误！data 在别处被借用了
```

这在你 C++ 经验里是正常的（`push_back` 到一个有 `span` 指向的 vector  = UB），但 Rust 在编译期就拒绝了。

### 陷阱 3：&str 的生命周期

```rust
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a } else { b }
}
```

`'a` 的生命周期标注告诉编译器：返回值的引用存活时间和两个参数中较短的那个一样久。这类似你在 C++ 里用手注释记录"这个 `string_view` 的生命期必须比源 `string` 短"，但 Rust 是编译器检查。

### 陷阱 4：tokio 的闭包里不能用 `&`

```rust
// ❌ 错误
let data = String::from("hello");
tokio::spawn(async {
    println!("{}", data);
});

// ✅ 正确：move 所有权
tokio::spawn(async move {
    println!("{}", data);
});
```

`tokio::spawn` 要求闭包是 `'static`，意味着不能持有借来的引用，必须 move。

### 陷阱 5：Rust 没有函数重载

```rust
// ❌ 不存在
fn search(query: &str) { ... }
fn search(query: &str, k: usize) { ... }

// ✅ 用不同名称，或用 Builder 模式
fn search(&self, query: &str) -> SearchBuilder { ... }
```

---

## 工作习惯建议

```bash
# 开发期用 check（不生成代码，快 10 倍），只检查类型错误
cargo check

# 修改依赖或 Cargo.toml 后
cargo update

# 持续测试
cargo test

# 格式化
cargo fmt
cargo clippy -- -D warnings   # Clippy 是 Rust 的 lint 工具，类似 clang-tidy

# release 构建
cargo build --release
```

IDE 用 VS Code + rust-analyzer 插件，或直接用 CLion + Rust 插件（你从 C++ CLion 过来最自然）。

---

## 总结

| | 原有计划 | 优化后 |
|---|---------|-------|
| 学习顺序 | 异步优先，API 先上 | **同步优先，本地先行** |
| 第一个检索 | Phase 0 做完才能看见 | **Phase 1 就能看到 BM25 结果** |
| C++ 衔接 | 少量类比 | **每个 Phase 都有 C++ 对比 + 陷阱提示** |
| 技术验证 | 无 | **修正 edition、修复代码签名、标注阅读材料** |
| Rust 概念密度 | 每个 Phase 塞入 5+ 新概念 | **每 Phase 只加 1~2 个** |
| Phase 间依赖 | 松耦合 | **每个 Phase 有可运行的二进制** |

最终你得到的不只是一个 RAG 系统，而是一步步构建它的 Rust 能力。先同步、再异步；先 BM25、再向量；先本地、再网络；先单线程、再并发。每一步都可以停下来验证你理解了。
