<div align="center">

<h1>Flyleaf-core</h1>

<p>
  <a href="README_en.md"><kbd>English</kbd></a>
  <a href="README.md"><kbd>简体中文</kbd></a>
</p>

<img src="https://moe-counter.ieshishinjin.workers.dev/github/PleaseEnterYourText-Studio/Flyleaf-core" title="Star!">

<p>扉页 Flyleaf 的核心库</p>
<p>解析 Kindle 的「我的剪辑」（<code>My Clippings.txt</code>）</p>

</div>

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