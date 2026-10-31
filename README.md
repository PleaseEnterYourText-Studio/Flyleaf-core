# Flyleaf-core

<div align="center">

<kbd>[English](README_en.md)</kbd> <kbd>[简体中文](README.md)</kbd>

</div>

<div align="center">
  <img src="https://moe-counter.ieshishinjin.workers.dev/github/PleaseEnterYourText-Studio/Flyleaf-core" title="Star!">
</div>


扉页 Flyleaf 的核心库

解析 Kindle 的「我的剪辑」（`My Clippings.txt`）

## 快速开始

```rust
let data = std::fs::read("My Clippings.txt")?;
let clippings = flyleaf_core::parse_clippings(&data)?;
let json = serde_json::to_string_pretty(&clippings)?;  // 输出：统一 JSON 文档
```

设备自动检测：

```rust
if let Ok(path) = flyleaf_core::device::find_kindle_clippings() {
    let data = std::fs::read(path)?;
    let clippings = flyleaf_core::parse_clippings(&data)?;
}
```

## 构建

```bash
cargo build                  # 核心库（rlib）
cargo build --features ffi   # C ABI
```
## 文档

- [docs/api.md](docs/api.md)：接口与数据模型一览

#### © 2026 PEYT-Studio, GPLv3 License