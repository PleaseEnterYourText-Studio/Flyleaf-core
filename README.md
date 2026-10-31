# flyleaf-core

扉页 Flyleaf 的核心库（Rust）：解析 Kindle 的「我的剪辑」（`My Clippings.txt`），输出统一数据模型，渲染 Markdown 文档，并（规划中）提供图片排版。命令行、macOS、Web 三端共用本库，业务逻辑只此一份，仓库内不含界面代码。

## 快速开始

```rust
let data = std::fs::read("My Clippings.txt")?;
let clippings = flyleaf_core::parse_clippings(&data)?;
let markdown = flyleaf_core::export::render_book_markdown(&clippings.books[0]);
```

设备自动检测（仅原生端）：

```rust
if let Ok(path) = flyleaf_core::device::find_kindle_clippings() {
    let data = std::fs::read(path)?;
    let clippings = flyleaf_core::parse_clippings(&data)?;
}
```

## 构建

```bash
cargo build                  # 核心库（rlib）
cargo build --features ffi   # C ABI（macOS / WinUI 桥接用）
```

## 模块与接口位置

| 路径 | 职责 |
| --- | --- |
| `src/model.rs` | 统一数据模型：`Clippings` / `Book` / `Item` / `ItemKind` / `Location` / `NoteAttachment` / `DeviceTime` |
| `src/parse/clippings.rs` | My Clippings.txt 解析器：`parse_clippings` |
| `src/export/markdown.rs` | Markdown 渲染：`render_book_markdown` |
| `src/device.rs` | 本地 Kindle 设备自动检测（仅原生端）：`detect_kindles` / `find_kindle_clippings` |
| `src/ffi.rs` | C ABI 导出（`--features ffi`） |
| `src/layout.rs` | 图片排版引擎（规划中） |
| `src/error.rs` | 统一错误类型与稳定错误码 |

## 文档

- [docs/integration.md](docs/integration.md)：如何接入核心库（Rust crate / C ABI / WASM）
- [docs/api.md](docs/api.md)：接口与数据模型一览

## 许可证

GPLv3，见 [LICENSE.txt](LICENSE.txt)。
