use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::model::Book;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageExportOptions {
    pub template: String,
    pub width: u32,
    pub height: u32,
}

impl Default for ImageExportOptions {
    fn default() -> Self {
        Self {
            template: "quote-card".to_string(),
            width: 1080,
            height: 1440,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderOutput {
    pub svg: String,
}

pub fn render_book_image(_book: &Book, _options: &ImageExportOptions) -> Result<RenderOutput> {
    Err(Error::NotImplemented("图片排版引擎（M2）"))
}
