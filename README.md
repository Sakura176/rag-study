# rag-study

用 Rust 实现 RAG（Retrieval-Augmented Generation）的学习项目。面向有 C++ 背景、希望系统性学习 Rust 的开发者。

## 项目目标

- 用 RAG 作为载体，从零开始逐步掌握 Rust
- 每个 Phase 只引入 1~2 个新概念，避免认知过载
- 先同步再异步、先本地再网络、先单线程再并发
- 所有阶段都有可运行的二进制产出

## 项目结构

```
src/
├── main.rs       # 入口（CLI）
├── lib.rs        # 公共模块导出
├── types.rs      # Chunk, ChunkMetadata 等核心类型
├── loader.rs     # PDF 加载（当前: lopdf 逐页提取文本）
└── splitter.rs   # 文本分块（当前: 按 chunk_size 切分）
docs/
└── optimization_roadmap.md  # 完整学习路线图
```

## 路线图

| Phase | 主题 | Rust 新概念 |
|-------|------|------------|
| 0a | 文档解析 + 分块 | 所有权、借用、Result、struct/impl |
| 0b | 异步 HTTP + API | async/await、serde、tokio |
| 1 | BM25 全文检索 | 闭包、迭代器、HashMap、单元测试 |
| 2 | Embedding + 向量检索 | trait、运算符重载、泛型 |
| 3 | LLM + Reranker + RAG 闭环 | dyn Trait、async trait |
| 4 | 并发 + 流式 + 持久化 | Send/Sync、Stream、JoinSet |
| 5 | 从零实现 HNSW（可选） | unsafe、裸指针 |

详见 [docs/optimization_roadmap.md](docs/optimization_roadmap.md)。

## 快速开始

```bash
# 构建
cargo build

# 运行
cargo run

# 测试
cargo test

# 代码检查
cargo clippy -- -D warnings
cargo fmt
```

## 当前进度

Phase 0a 进行中：核心类型、PDF 加载、文本切分已实现，CLI 入口待接入。

## 设计原则

- **每个 Phase 只加 1~2 个 Rust 新概念**，不贪多
- **先同步再异步**：Phase 0a 全部同步，Phase 0b 引入 async
- **先内存再外部**：Phase 1 先在进程内跑 BM25，Phase 2 再接入外部 API
- **每个 Phase 有可运行的二进制**：不累积到最后才看到结果
