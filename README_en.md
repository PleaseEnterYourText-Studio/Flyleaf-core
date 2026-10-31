# Flyleaf-core

<div align="center">

<kbd>[English](README_en.md)</kbd> <kbd>[简体中文](README.md)</kbd>

</div>

<div align="center">
  <img src="https://moe-counter.ieshishinjin.workers.dev/github/PleaseEnterYourText-Studio/Flyleaf-core" title="Star!">
</div>


The core library of Flyleaf

Parses Kindle's "My Clippings" (`My Clippings.txt`)

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
