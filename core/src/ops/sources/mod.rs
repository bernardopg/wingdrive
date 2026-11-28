//! Source operations for archive data.
//!
//! Sources are library-scoped archive data stores that index external content
//! like emails, notes, bookmarks, etc. from various adapters.

pub mod config;
pub mod create;
pub mod delete;
pub mod get;
pub mod get_item;
pub mod list;
pub mod list_items;
pub mod search;
pub mod sync;
pub mod update;

pub use config::*;
pub use create::*;
pub use delete::*;
pub use get::*;
pub use get_item::*;
pub use list::*;
pub use list_items::*;
pub use search::*;
pub use sync::*;
pub use update::*;
