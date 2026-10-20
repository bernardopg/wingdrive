//! Wingdrop: send files to a paired device, which accepts or declines them.
//! The exchange and its safety rules live in `service::wingdrop`.

pub mod actions;
pub mod job;

pub use actions::*;
pub use job::WingdropSendJob;
