use std::sync::OnceLock;

use regex::{Captures, Regex};

use crate::error::{Error, Result};
use crate::model::{Book, Clippings, DeviceTime, Item, ItemKind, Location, NoteAttachment};

const PAIR_TOLERANCE: u32 = 2;

pub fn parse_clippings(data: &[u8]) -> Result<Clippings> {
    let text = decode(data)?;
    let mut books: Vec<Book> = Vec::new();

    for block in split_blocks(&text) {
        let Some(raw) = RawEntry::from_block(&block) else {
            continue;
        };
        let (title, author) = split_title_author(&raw.title_line);
        if title.is_empty() {
            continue;
        }
        let item = parse_item(&raw);
        match books.iter().position(|book| book.title == title) {
            Some(index) => {
                let book = &mut books[index];
                if book.author.is_none() {
                    book.author = author;
                }
                book.items.push(item);
            }
            None => books.push(Book {
                title,
                author,
                items: vec![item],
            }),
        }
    }

    for book in &mut books {
        finalize_book(book);
    }
    Ok(Clippings { books })
}

fn decode(data: &[u8]) -> Result<String> {
    let text = std::str::from_utf8(data).map_err(|_| {
        Error::InvalidData(
            "文件不是有效的 UTF-8 文本（My Clippings.txt 应为 UTF-8；请确认选择的是 Kindle 的「我的剪辑」文件）"
                .into(),
        )
    })?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    Ok(text.replace("\r\n", "\n").replace('\r', "\n"))
}

fn split_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in text.lines() {
        if is_delimiter(line) {
            blocks.push(current.join("\n"));
            current.clear();
        } else {
            current.push(line);
        }
    }
    blocks.push(current.join("\n"));
    blocks.retain(|block| !block.trim().is_empty());
    blocks
}

fn is_delimiter(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.chars().count() >= 5 && trimmed.chars().all(|c| c == '=')
}

struct RawEntry {
    title_line: String,
    meta_line: String,
    content: String,
}

impl RawEntry {
    fn from_block(block: &str) -> Option<Self> {
        let lines: Vec<&str> = block.lines().map(str::trim_end).collect();
        let start = lines.iter().position(|line| !line.trim().is_empty())?;
        let end = lines.iter().rposition(|line| !line.trim().is_empty())? + 1;
        let lines = &lines[start..end];

        let title_line = lines[0].trim().to_string();
        if lines.len() == 1 {
            return Some(Self {
                title_line,
                meta_line: String::new(),
                content: String::new(),
            });
        }

        let mut index = 1;
        while index < lines.len() && lines[index].trim().is_empty() {
            index += 1;
        }
        let meta_line = if index < lines.len() {
            lines[index].trim().to_string()
        } else {
            String::new()
        };
        if index < lines.len() {
            index += 1;
        }
        while index < lines.len() && lines[index].trim().is_empty() {
            index += 1;
        }
        let content = lines[index..].join("\n").trim().to_string();

        Some(Self {
            title_line,
            meta_line,
            content,
        })
    }
}

fn parse_item(raw: &RawEntry) -> Item {
    let meta = raw.meta_line.as_str();
    let added_at_raw = extract_added_raw(meta).unwrap_or_default();
    let added_at = non_empty(&added_at_raw).and_then(parse_device_time);
    Item {
        kind: detect_kind(meta),
        location: extract_location(meta),
        meta_raw: meta.to_string(),
        added_at_raw,
        added_at,
        text: raw.content.clone(),
        note: None,
    }
}

fn detect_kind(meta: &str) -> ItemKind {
    let lower = meta.to_ascii_lowercase();
    if meta.contains("书签") || lower.contains("bookmark") {
        ItemKind::Bookmark
    } else if meta.contains("笔记") || lower.contains("note") {
        ItemKind::Note
    } else if meta.contains("标注") || lower.contains("highlight") {
        ItemKind::Highlight
    } else {
        ItemKind::Unknown
    }
}

fn extract_location(meta: &str) -> Location {
    let mut location = Location::default();
    if let Some(caps) = re_location().captures(meta) {
        if let Some(start) = caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok()) {
            location.start = Some(start);
            location.end = caps
                .get(2)
                .and_then(|m| m.as_str().parse::<u32>().ok())
                .or(Some(start));
        }
    }
    if let Some(caps) = re_page().captures(meta) {
        location.page = caps
            .get(1)
            .or_else(|| caps.get(2))
            .and_then(|m| m.as_str().parse::<u32>().ok());
    }
    location
}

fn extract_added_raw(meta: &str) -> Option<String> {
    if let Some(pos) = meta.find("添加于") {
        return non_empty(&meta[pos + "添加于".len()..]).map(str::to_string);
    }
    if let Some(caps) = re_added_on().captures(meta) {
        if let Some(m) = caps.get(1) {
            return non_empty(m.as_str()).map(str::to_string);
        }
    }
    None
}

fn parse_device_time(raw: &str) -> Option<DeviceTime> {
    if let Some(caps) = re_time_zh().captures(raw) {
        if let Some(time) = try_time_zh(&caps) {
            return Some(time);
        }
    }
    if let Some(caps) = re_time_en_month_first().captures(raw) {
        if let Some(time) = try_time_en_month_first(&caps) {
            return Some(time);
        }
    }
    if let Some(caps) = re_time_en_day_first().captures(raw) {
        if let Some(time) = try_time_en_day_first(&caps) {
            return Some(time);
        }
    }
    None
}

fn try_time_zh(caps: &Captures) -> Option<DeviceTime> {
    let year = caps[1].parse().ok()?;
    let month = caps[2].parse().ok()?;
    let day = caps[3].parse().ok()?;
    let marker = caps.get(4).map(|m| m.as_str());
    let hour = caps[5].parse().ok()?;
    let minute = caps[6].parse().ok()?;
    let second = caps
        .get(7)
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0);
    let hour = adjust_zh_hour(marker, hour);
    make_time(year, month, day, hour, minute, second)
}

fn try_time_en_month_first(caps: &Captures) -> Option<DeviceTime> {
    let month = month_from_name(&caps[1])?;
    let day = caps[2].parse().ok()?;
    let year = caps[3].parse().ok()?;
    let hour = caps[4].parse().ok()?;
    let minute = caps[5].parse().ok()?;
    let second = caps
        .get(6)
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0);
    let hour = adjust_en_hour(caps.get(7).map(|m| m.as_str()), hour);
    make_time(year, month, day, hour, minute, second)
}

fn try_time_en_day_first(caps: &Captures) -> Option<DeviceTime> {
    let day = caps[1].parse().ok()?;
    let month = month_from_name(&caps[2])?;
    let year = caps[3].parse().ok()?;
    let hour = caps[4].parse().ok()?;
    let minute = caps[5].parse().ok()?;
    let second = caps
        .get(6)
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0);
    let hour = adjust_en_hour(caps.get(7).map(|m| m.as_str()), hour);
    make_time(year, month, day, hour, minute, second)
}

fn adjust_zh_hour(marker: Option<&str>, hour: u32) -> u32 {
    match marker {
        Some("下午") | Some("晚上") => {
            if hour < 12 {
                hour + 12
            } else {
                hour
            }
        }
        Some("上午") | Some("凌晨") => {
            if hour == 12 {
                0
            } else {
                hour
            }
        }
        _ => hour,
    }
}

fn adjust_en_hour(marker: Option<&str>, hour: u32) -> u32 {
    match marker.map(str::to_ascii_lowercase).as_deref() {
        Some(m) if m.starts_with('p') => {
            if hour < 12 {
                hour + 12
            } else {
                hour
            }
        }
        Some(m) if m.starts_with('a') => {
            if hour == 12 {
                0
            } else {
                hour
            }
        }
        _ => hour,
    }
}

fn month_from_name(name: &str) -> Option<u32> {
    let name = name.trim_end_matches('.').to_ascii_lowercase();
    let key = name.get(..3)?;
    match key {
        "jan" => Some(1),
        "feb" => Some(2),
        "mar" => Some(3),
        "apr" => Some(4),
        "may" => Some(5),
        "jun" => Some(6),
        "jul" => Some(7),
        "aug" => Some(8),
        "sep" => Some(9),
        "oct" => Some(10),
        "nov" => Some(11),
        "dec" => Some(12),
        _ => None,
    }
}

fn make_time(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Option<DeviceTime> {
    if !(1900..=2100).contains(&year)
        || !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }
    Some(DeviceTime {
        year,
        month: month as u8,
        day: day as u8,
        hour: hour as u8,
        minute: minute as u8,
        second: second as u8,
    })
}

fn split_title_author(line: &str) -> (String, Option<String>) {
    let text = line.trim();

    if let Some(open) = text.rfind(|c| c == '(' || c == '（') {
        let after_open = &text[open..];
        let tail = after_open
            .strip_prefix('(')
            .or_else(|| after_open.strip_prefix('（'))
            .unwrap_or(after_open);
        if let Some(inner) = tail.trim_end().strip_suffix(|c| c == ')' || c == '）') {
            if !inner.contains('(')
                && !inner.contains('（')
                && !inner.contains(')')
                && !inner.contains('）')
            {
                let title = text[..open].trim();
                let author = inner.trim();
                if !title.is_empty() && !author.is_empty() {
                    return (title.to_string(), Some(author.to_string()));
                }
            }
        }
    }

    if let Some(open) = text.find(|c| c == '(' || c == '（') {
        let tail = &text[open..];
        if is_balanced_group(tail) {
            let title = text[..open].trim();
            let body = tail
                .strip_prefix('(')
                .or_else(|| tail.strip_prefix('（'))
                .unwrap_or(tail);
            let body = body.trim_end();
            let body = body
                .strip_suffix(')')
                .or_else(|| body.strip_suffix('）'))
                .unwrap_or(body);
            let author = body.trim();
            if !title.is_empty() && !author.is_empty() {
                return (title.to_string(), Some(author.to_string()));
            }
        }
    }

    (text.to_string(), None)
}

fn is_balanced_group(s: &str) -> bool {
    let s = s.trim_end();
    if !s.ends_with(')') && !s.ends_with('）') {
        return false;
    }
    let mut depth: i32 = 0;
    for c in s.chars() {
        match c {
            '(' | '（' => depth += 1,
            ')' | '）' => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

fn finalize_book(book: &mut Book) {
    pair_notes(book);
    book.items.sort_by_key(|item| {
        (
            item.location.start.unwrap_or(u32::MAX),
            item.location.page.unwrap_or(u32::MAX),
        )
    });
}

fn pair_notes(book: &mut Book) {
    let mut highlights: Vec<Item> = Vec::new();
    let mut notes: Vec<Item> = Vec::new();
    let mut others: Vec<Item> = Vec::new();
    for item in book.items.drain(..) {
        match item.kind {
            ItemKind::Highlight => highlights.push(item),
            ItemKind::Note => notes.push(item),
            _ => others.push(item),
        }
    }

    for note in notes {
        let target = note.location.start.and_then(|note_start| {
            highlights
                .iter()
                .enumerate()
                .filter(|(_, highlight)| highlight.note.is_none())
                .filter_map(|(index, highlight)| {
                    let start = highlight.location.start?;
                    let end = highlight.location.end.unwrap_or(start);
                    let distance = if note_start < start {
                        start - note_start
                    } else if note_start > end {
                        note_start - end
                    } else {
                        0
                    };
                    (distance <= PAIR_TOLERANCE).then_some((index, distance))
                })
                .min_by_key(|(_, distance)| *distance)
                .map(|(index, _)| index)
        });

        match target {
            Some(index) => {
                let highlight = &mut highlights[index];
                highlight.note = Some(NoteAttachment {
                    location: note.location,
                    added_at: note.added_at,
                    added_at_raw: note.added_at_raw,
                    text: note.text,
                });
            }
            None => others.push(note),
        }
    }

    let mut all = highlights;
    all.append(&mut others);
    book.items = all;
}

fn non_empty(s: &str) -> Option<&str> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn re_location() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)(?:#\s*|(?:位置|location)\s*#?\s*)(\d+)(?:\s*[-–—]\s*(\d+))?")
            .expect("位置正则合法")
    })
}

fn re_page() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?:第\s*(\d+)\s*页)|(?i:page)\s+(\d+)").expect("页码正则合法")
    })
}

fn re_added_on() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)added\s+on\s+(.+)$").expect("Added on 正则合法"))
}

fn re_time_zh() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(\d{4})年\s*(\d{1,2})月\s*(\d{1,2})日\s*(?:星期[一二三四五六日天]\s*)?(上午|下午|凌晨|晚上|中午)?\s*(\d{1,2})\s*[:：]\s*(\d{1,2})(?:\s*[:：]\s*(\d{1,2}))?",
        )
        .expect("中文日期正则合法")
    })
}

fn re_time_en_month_first() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b([A-Za-z]{3,9})\.?\s+(\d{1,2}),?\s+(\d{4})\s+(\d{1,2}):(\d{2})(?::(\d{2}))?(?:\s*([APap])\.?[Mm]\.?)?",
        )
        .expect("英文日期（月在前）正则合法")
    })
}

fn re_time_en_day_first() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(\d{1,2})\.?\s+([A-Za-z]{3,9})\.?,?\s+(\d{4})\s+(\d{1,2}):(\d{2})(?::(\d{2}))?(?:\s*([APap])\.?[Mm]\.?)?",
        )
        .expect("英文日期（日在前）正则合法")
    })
}
