# 接口与数据模型

## 公开函数

| 函数 | 位置 | 说明 |
| --- | --- | --- |
| `parse_clippings(data: &[u8]) -> Result<Clippings>` | `src/parse/clippings.rs` | 解析 UTF-8 的 My Clippings.txt |
| `detect_kindles() -> Vec<KindleDevice>` | `src/device.rs` | 列出检测到的 Kindle（仅原生端） |
| `find_kindle_clippings() -> Result<PathBuf>` | `src/device.rs` | 唯一设备时返回其 My Clippings.txt 路径 |
| `scanned_locations() -> &'static str` | `src/device.rs` | 本平台扫描位置（用于提示信息） |

## C ABI（`--features ffi`，src/ffi.rs）

| 函数 | 说明 |
| --- | --- |
| `flyleaf_core_version(void)` | 版本字符串，静态内存，无需释放 |
| `flyleaf_abi_version(void)` | ABI 版本号，破坏性变更时递增 |
| `flyleaf_parse_clippings(data, len)` | 解析 My Clippings.txt，返回 JSON 信封 |
| `flyleaf_string_free(s)` | 释放本库返回的字符串 |

成功 `{"ok":true,"data":{...}}`，失败 `{"ok":false,"error":{"code":"...","message":"..."}}`；库内分配，调用方用 `flyleaf_string_free` 释放。

## 输出与渲染边界

- 核心库输出 = 统一数据模型（`Clippings`，可直接用 `serde_json` 序列化为 JSON 文档）；库内不含任何渲染逻辑。
- Markdown 渲染：由命令行仓库（`flyleaf-cli`）实现，输入为上述模型 / JSON 文档。
- 图片排版：规划中，将由各壳独立实现。

## 数据模型（src/model.rs）

```
Clippings { books: [Book] }                   # books 按文件中首次出现的顺序
Book      { title, author?, items: [Item] }   # items 已按位置排序
Item      { kind, location, meta_raw, added_at_raw, added_at?, text, note? }
ItemKind  = highlight | note | bookmark | unknown
Location  { start?, end?, page? }
NoteAttachment { location, added_at_raw, added_at?, text }
DeviceTime { year, month, day, hour, minute, second }
```

JSON 示例：

```json
{
  "books": [
    {
      "title": "三体",
      "author": "刘慈欣",
      "items": [
        {
          "kind": "highlight",
          "location": { "start": 123, "end": 125, "page": null },
          "meta_raw": "- 您在位置 #123-125 的标注 | 添加于 2024年5月26日星期日 下午5:52:21",
          "added_at_raw": "2024年5月26日星期日 下午5:52:21",
          "added_at": { "year": 2024, "month": 5, "day": 26, "hour": 17, "minute": 52, "second": 21 },
          "text": "……",
          "note": {
            "location": { "start": 123, "end": 123, "page": null },
            "added_at_raw": "……",
            "added_at": null,
            "text": "……"
          }
        }
      ]
    }
  ]
}
```

## 行为约定

- 输入 UTF-8：自动剥离 BOM，CRLF/CR 归一；非 UTF-8 返回 `invalid_data`。
- 书名行可带作者：`书名 (作者)`，全角/半角括号均可；作者名自身含括号也能识别。
- 类型：标注 / Highlight、笔记 / Note、书签 / Bookmark；无法识别归 `unknown`，原文保留不丢数据。
- 位置：`#123-125`、`位置72-74`、`Location 640-641`；页码：`第 42 页`、`page 42`（两者可并存）。
- 日期：中文（`2024年5月26日星期日 下午5:52:21`）与英文（`May 26, 2024 5:52:21 PM`、`26 May 2024 17:52:21`）归一化为 `DeviceTime`；解析失败时保留 `added_at_raw` 原文。
- 归并：按书名精确匹配合并；书内按位置升序（稳定排序，位置缺失时用页码）。
- 配对：同一本书内「笔记」与「标注」位置距离 ≤2 时自动配对；未配对笔记保留为独立条目。

## 错误码

| code | 触发场景 |
| --- | --- |
| `unsupported_format` | 输入格式无法识别 |
| `invalid_data` | 数据无效（非 UTF-8、空指针等） |
| `io` | I/O 错误 |
| `not_implemented` | 尚未实现 |
| `device` | 设备检测失败（未找到 / 多台 / 无剪辑文件） |
| `internal` / `panic` | 内部错误（不应出现） |
