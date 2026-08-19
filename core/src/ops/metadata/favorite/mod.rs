//! Favorite operations
//!
//! Marks files as favorite via the shared user metadata store.

pub mod action;
pub mod input;
pub mod output;

pub use action::SetFavoriteAction;
pub use input::SetFavoriteInput;
pub use output::SetFavoriteOutput;