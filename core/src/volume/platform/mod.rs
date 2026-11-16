//! Platform-specific volume detection helpers

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "ios")]
pub mod ios;

// Also built for tests on other Unix systems: its detection takes the storage root as a parameter.
#[cfg(any(target_os = "android", all(test, unix)))]
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub mod android;
