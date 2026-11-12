//! # Google integration
//!
//! Connects Google accounts to a library and exposes their services: Drive as
//! a cloud volume (`drive`) and Photos upload, albums and picker import
//! (`photos`). `oauth` runs the browser sign-in, `tokens` keeps access tokens
//! fresh and retries API calls, and `endpoints` holds every Google URL so tests
//! can run the whole stack against a local fake server.

pub mod drive;
pub mod endpoints;
pub mod error;
pub mod oauth;
pub mod photos;
pub mod tokens;

pub use drive::GoogleDriveBackend;
pub use endpoints::GoogleEndpoints;
pub use error::{GoogleError, GoogleResult};
pub use oauth::{GoogleService, OAuthClient};
pub use photos::PhotosClient;
pub use tokens::{GoogleApi, GoogleTokenSource};
