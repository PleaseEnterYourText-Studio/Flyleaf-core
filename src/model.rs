use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Clippings {
    pub books: Vec<Book>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Book {
    pub title: String,
    pub author: Option<String>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    pub location: Location,
    pub meta_raw: String,
    pub added_at_raw: String,
    pub added_at: Option<DeviceTime>,
    pub text: String,
    pub note: Option<NoteAttachment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    Highlight,
    Note,
    Bookmark,
    Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub start: Option<u32>,
    pub end: Option<u32>,
    pub page: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteAttachment {
    pub location: Location,
    pub added_at_raw: String,
    pub added_at: Option<DeviceTime>,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl DeviceTime {
    pub fn to_date_string(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    pub fn to_datetime_string(&self) -> String {
        format!(
            "{} {:02}:{:02}:{:02}",
            self.to_date_string(),
            self.hour,
            self.minute,
            self.second
        )
    }
}
