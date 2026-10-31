#[cfg(not(target_arch = "wasm32"))]
pub mod device;
pub mod error;
pub mod model;
pub mod parse;

#[cfg(feature = "ffi")]
pub mod ffi;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use error::{Error, Result};
pub use model::{Book, Clippings, DeviceTime, Item, ItemKind, Location, NoteAttachment};
pub use parse::parse_clippings;
