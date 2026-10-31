<div align="center">

<h1>Flyleaf-core</h1>

<p>
  <a href="README_en.md"><kbd>English</kbd></a>
  <a href="README.md"><kbd>简体中文</kbd></a>
</p>

<img src="..." title="Star!">

<p>The core library of Flyleaf</p>
<p>Parses Kindle's "My Clippings"（<code>My Clippings.txt</code>）</p>

</div>

## Quick Start

```rust
let data = std::fs::read("My Clippings.txt")?;
let clippings = flyleaf_core::parse_clippings(&data)?;
let json = serde_json::to_string_pretty(&clippings)?;  // Output: unified JSON document
```

Automatic device detection:

```rust
if let Ok(path) = flyleaf_core::device::find_kindle_clippings() {
    let data = std::fs::read(path)?;
    let clippings = flyleaf_core::parse_clippings(&data)?;
}
```

## Build

```bash
cargo build                  # Core library (rlib)
cargo build --features ffi   # C ABI
```
## Docs

- [docs/api.md](docs/api.md): API and data model reference

#### © 2026 PEYT-Studio, GPLv3 License
