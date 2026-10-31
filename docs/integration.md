# 接入指南

核心库是纯计算库：**不做文件读写、不含界面**。输入字节 → 统一数据模型 → 导出文本或布局结果；文件读写与展示由各壳负责。

## 1. Rust crate 依赖（CLI 的方式）

Cargo.toml：

```toml
[dependencies]
flyleaf-core = { path = "core" }  # 本地目录或 git 子模块检出的源码
```

调用：

```rust
use flyleaf_core::{export::render_book_markdown, parse_clippings};

let data = std::fs::read("My Clippings.txt")?;
let clippings = parse_clippings(&data)?;
for book in &clippings.books {
    let markdown = render_book_markdown(book);
    std::fs::write(format!("{}.md", book.title), markdown)?;
}
```

设备自动检测（仅原生端；`wasm32` 目标下该模块不编译）：

```rust
if let Ok(path) = flyleaf_core::device::find_kindle_clippings() {
    let data = std::fs::read(path)?;
    // ...
}
```

## 2. C ABI（macOS SwiftUI / WinUI）

构建：

```bash
cargo build --release --features ffi
# macOS 产物：
#   target/release/libflyleaf_core.dylib（动态库）
#   target/release/libflyleaf_core.a    （静态库）
```

导出函数（4 个）：

| 函数 | 说明 |
| --- | --- |
| `const char *flyleaf_core_version(void)` | 版本字符串，静态内存，无需释放 |
| `uint32_t flyleaf_abi_version(void)` | ABI 版本号，破坏性变更时递增 |
| `char *flyleaf_parse_clippings(const uint8_t *data, size_t len)` | 解析 My Clippings.txt，返回 JSON 信封 |
| `void flyleaf_string_free(char *s)` | 释放上面返回的字符串 |

约定：

- 复杂载荷一律走 JSON 字符串：成功 `{"ok":true,"data":{...}}`，失败 `{"ok":false,"error":{"code":"...","message":"..."}}`。
- 所有权单向：库内分配，调用方用 `flyleaf_string_free` 释放，不要用 `free()`。
- 错误码稳定：`unsupported_format` / `invalid_data` / `io` / `not_implemented` / `device` / `internal` / `panic`。

C 调用示例：

```c
#include <stdio.h>
#include <string.h>

extern const char *flyleaf_core_version(void);
extern char *flyleaf_parse_clippings(const unsigned char *data, size_t len);
extern void flyleaf_string_free(char *s);

int main(void) {
    const char *text =
        "书名 (作者)\n"
        "- 您在位置 #1-2 的标注 | 添加于 2024年5月26日星期日 下午5:52:21\n"
        "\n正文\n"
        "==========\n";
    printf("version=%s\n", flyleaf_core_version());
    char *json = flyleaf_parse_clippings((const unsigned char *)text, strlen(text));
    puts(json);
    flyleaf_string_free(json);
    return 0;
}
```

```bash
clang demo.c -L target/release -lflyleaf_core -o demo
DYLD_LIBRARY_PATH=target/release ./demo   # 或改用 .a 静态链接
```

Swift：把库（`.a` 或 `.dylib`）加入工程，通过 C 互操作调用上述函数，返回的 JSON 用 `Codable` 解码（数据结构见 [api.md](api.md)）。

## 3. WebAssembly（规划中）

- 用 wasm-bindgen 导出与 C ABI 相同的 JSON 函数面；
- `device` 模块不参与编译（浏览器没有文件系统）；
- 文件选择由前端负责，把字节交给解析函数。
