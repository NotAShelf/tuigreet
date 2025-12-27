//! Internal library for tuigreet binary
//!
//! This library is NOT intended for external use or as a public API.
//! It exists solely to organize code within the tuigreet project, separating
//! reusable components (config, types, theme) from binary-specific logic.
//!
//! There is no guarantee of stability, and you are discouraged from ever
//! interacting with this "library."
pub mod config;
pub mod theme;
pub mod types;

pub use crate::{
  config::Config,
  theme::{Theme, Themed},
  types::*,
};
