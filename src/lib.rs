pub mod error;
pub mod layout;
pub mod model;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use error::{Error, Result};
pub use model::{Book, Clippings, DeviceTime, Item, ItemKind, Location, NoteAttachment};
