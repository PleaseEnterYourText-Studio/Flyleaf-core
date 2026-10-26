use flyleaf_core::export::render_book_markdown;
use flyleaf_core::{parse_clippings, Clippings, DeviceTime, ItemKind};

fn read_fixture() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_my_clippings.txt");
    std::fs::read(&path).unwrap_or_else(|e| panic!("读取测试夹具失败 {}：{e}", path.display()))
}

fn parse_fixture() -> Clippings {
    parse_clippings(&read_fixture()).expect("解析夹具失败")
}

#[test]
fn parses_fixture_into_unified_model() {
    let clippings = parse_fixture();

    let titles: Vec<&str> = clippings.books.iter().map(|b| b.title.as_str()).collect();
    assert_eq!(
        titles,
        [
            "三体",
            "人类简史",
            "Sapiens: A Brief History of Humankind",
            "月亮与六便士"
        ]
    );

    let santi = &clippings.books[0];
    assert_eq!(santi.author.as_deref(), Some("刘慈欣"));
    assert_eq!(santi.items.len(), 1);
    let item = &santi.items[0];
    assert_eq!(item.kind, ItemKind::Highlight);
    assert_eq!(item.location.start, Some(123));
    assert_eq!(item.location.end, Some(125));
    assert_eq!(
        item.added_at,
        Some(DeviceTime {
            year: 2024,
            month: 5,
            day: 26,
            hour: 17,
            minute: 52,
            second: 21,
        })
    );
    let note = item.note.as_ref().expect("笔记应已配对");
    assert_eq!(note.text, "这句话每次重读都让人起鸡皮疙瘩。");

    let renlei = &clippings.books[1];
    assert_eq!(renlei.author.as_deref(), Some("尤瓦尔·赫拉利"));
    let kinds: Vec<ItemKind> = renlei.items.iter().map(|item| item.kind).collect();
    assert_eq!(
        kinds,
        [ItemKind::Highlight, ItemKind::Bookmark, ItemKind::Note]
    );
    assert_eq!(renlei.items[0].location.page, Some(42));
    assert_eq!(renlei.items[1].location.start, Some(700));
    assert_eq!(renlei.items[2].location.start, Some(900));

    let sapiens = &clippings.books[2];
    assert_eq!(sapiens.author.as_deref(), Some("Yuval Noah Harari"));
    assert_eq!(sapiens.items[0].kind, ItemKind::Highlight);
    assert_eq!(sapiens.items[0].location.page, Some(42));
    assert_eq!(sapiens.items[0].added_at.map(|t| t.hour), Some(10));

    let yueliang = &clippings.books[3];
    assert_eq!(yueliang.author.as_deref(), Some("（英）毛姆"));
    assert!(yueliang.items[0].text.contains('\n'));
}

#[test]
fn renders_book_markdown_golden() {
    let clippings = parse_fixture();
    let markdown = render_book_markdown(&clippings.books[0]);
    assert_eq!(
        markdown,
        "# 三体\n\n*刘慈欣 · 1 条标注*\n\n## 标注\n\n- 宇宙就是一座黑暗森林，每个文明都是带枪的猎人。\n  - 位置 123-125 · 2024-05-26\n  - 笔记：这句话每次重读都让人起鸡皮疙瘩。\n"
    );
}

#[test]
fn renders_sections_in_fixed_order() {
    let clippings = parse_fixture();
    let markdown = render_book_markdown(&clippings.books[1]);
    let positions: Vec<usize> = ["## 标注", "## 笔记", "## 书签"]
        .iter()
        .map(|section| {
            markdown
                .find(section)
                .unwrap_or_else(|| panic!("缺少分节 {section}"))
        })
        .collect();
    assert!(positions[0] < positions[1] && positions[1] < positions[2]);
    assert!(markdown.contains("第 42 页 · 位置 640-641"));
    assert!(markdown.contains("*尤瓦尔·赫拉利 · 1 条标注 · 1 条笔记 · 1 个书签*"));
}
