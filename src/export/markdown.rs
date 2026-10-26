use crate::model::{Book, Item, ItemKind, Location};

pub fn render_book_markdown(book: &Book) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", book.title.trim()));

    let (highlights, notes, bookmarks, _others) = counts(book);
    let mut parts: Vec<String> = Vec::new();
    if let Some(author) = book
        .author
        .as_deref()
        .map(str::trim)
        .filter(|author| !author.is_empty())
    {
        parts.push(author.to_string());
    }
    if highlights > 0 {
        parts.push(format!("{highlights} 条标注"));
    }
    if notes > 0 {
        parts.push(format!("{notes} 条笔记"));
    }
    if bookmarks > 0 {
        parts.push(format!("{bookmarks} 个书签"));
    }
    if !parts.is_empty() {
        out.push_str(&format!("*{}*\n\n", parts.join(" · ")));
    }

    render_section(&mut out, "## 标注", book, |item| {
        item.kind == ItemKind::Highlight
    });
    render_section(&mut out, "## 笔记", book, |item| item.kind == ItemKind::Note);
    render_section(&mut out, "## 其他", book, |item| item.kind == ItemKind::Unknown);
    render_section(&mut out, "## 书签", book, |item| {
        item.kind == ItemKind::Bookmark
    });

    let mut result = out.trim_end().to_string();
    result.push('\n');
    result
}

fn counts(book: &Book) -> (usize, usize, usize, usize) {
    let mut highlights = 0;
    let mut notes = 0;
    let mut bookmarks = 0;
    let mut others = 0;
    for item in &book.items {
        match item.kind {
            ItemKind::Highlight => highlights += 1,
            ItemKind::Note => notes += 1,
            ItemKind::Bookmark => bookmarks += 1,
            ItemKind::Unknown => others += 1,
        }
    }
    (highlights, notes, bookmarks, others)
}

fn render_section(out: &mut String, title: &str, book: &Book, predicate: impl Fn(&Item) -> bool) {
    let matched: Vec<&Item> = book.items.iter().filter(|item| predicate(item)).collect();
    if matched.is_empty() {
        return;
    }
    out.push_str(title);
    out.push_str("\n\n");
    for item in matched {
        render_item(out, item);
        out.push('\n');
    }
}

fn render_item(out: &mut String, item: &Item) {
    let has_text = !item.text.is_empty();
    if has_text {
        push_bullet(out, &item.text, "");
        if let Some(meta) = meta_line(item) {
            push_bullet(out, &meta, "  ");
        }
    } else if let Some(meta) = meta_line(item) {
        push_bullet(out, &meta, "");
    } else {
        push_bullet(out, "（无内容）", "");
    }
    if let Some(note) = &item.note {
        if !note.text.is_empty() {
            let indent = if has_text { "  " } else { "" };
            push_bullet(out, &format!("笔记：{}", note.text), indent);
        }
    }
}

fn meta_line(item: &Item) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    if let Some(location) = location_label(&item.location) {
        parts.push(location);
    }
    if let Some(date) = date_label(item) {
        parts.push(date);
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    }
}

fn location_label(location: &Location) -> Option<String> {
    let mut text = String::new();
    if let Some(page) = location.page {
        text.push_str(&format!("第 {page} 页"));
    }
    if let Some(start) = location.start {
        let range = match location.end {
            Some(end) if end > start => format!("位置 {start}-{end}"),
            _ => format!("位置 {start}"),
        };
        if !text.is_empty() {
            text.push_str(" · ");
        }
        text.push_str(&range);
    }
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn date_label(item: &Item) -> Option<String> {
    if let Some(time) = item.added_at {
        return Some(time.to_date_string());
    }
    let raw = item.added_at_raw.trim();
    if raw.is_empty() {
        None
    } else {
        Some(raw.to_string())
    }
}

fn push_bullet(out: &mut String, text: &str, indent: &str) {
    let mut lines = text.lines();
    let first = lines.next().unwrap_or("");
    out.push_str(indent);
    out.push_str("- ");
    out.push_str(first);
    out.push('\n');
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        out.push_str(indent);
        out.push_str("  ");
        out.push_str(line);
        out.push('\n');
    }
}
