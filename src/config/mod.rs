pub mod env;
pub mod parser;
mod schema;
pub mod theme;

use std::{fmt, io, path::PathBuf};

pub use schema::*;

/// Errors that can occur during configuration loading or validation
#[derive(Debug)]
pub enum ConfigError {
  /// I/O error when reading config files
  Io(io::Error),

  /// TOML parsing error
  Parse(toml::de::Error),

  /// TOML parsing error with file context
  ParseWithContext {
    file:             PathBuf,
    line:             usize,
    column:           usize,
    context:          Vec<String>,
    original_message: String,
  },
  /// General validation error with description
  Validation(String),

  /// Two options that cannot be used together
  MutuallyExclusive(String, String),

  /// Option that depends on another option
  Dependency(String),

  /// Invalid range (e.g., min_uid >= max_uid)
  InvalidRange(String),

  /// Duplicate keybindings
  DuplicateKeybindings,

  /// Invalid F-key value
  InvalidFKey(String, u8),

  /// Invalid time format string
  InvalidTimeFormat,

  /// Session wrapper executable not found
  WrapperExecutableNotFound(PathBuf),
}

impl fmt::Display for ConfigError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ConfigError::Io(err) => write!(f, "I/O error: {}", err),
      ConfigError::Parse(err) => write!(f, "TOML parse error: {}", err),
      ConfigError::ParseWithContext {
        file,
        line,
        column,
        context,
        original_message,
      } => {
        writeln!(f, "TOML parse error: {}", original_message)?;
        writeln!(f, "File: {}", file.display())?;
        writeln!(f, "Line: {}, Column: {}", line, column)?;
        writeln!(f)?;
        for line in context {
          writeln!(f, "{}", line)?;
        }
        Ok(())
      },
      ConfigError::Validation(msg) => write!(f, "Validation error: {}", msg),
      ConfigError::MutuallyExclusive(opt1, opt2) => {
        write!(
          f,
          "Options '{}' and '{}' are mutually exclusive",
          opt1, opt2
        )
      },
      ConfigError::Dependency(msg) => write!(f, "Dependency error: {}", msg),
      ConfigError::InvalidRange(msg) => write!(f, "Invalid range: {}", msg),
      ConfigError::DuplicateKeybindings => {
        write!(f, "Duplicate keybindings detected")
      },
      ConfigError::InvalidFKey(name, key) => {
        write!(
          f,
          "Invalid F-key value for '{}': F{} (must be F1-F12)",
          name, key
        )
      },
      ConfigError::InvalidTimeFormat => write!(f, "Invalid time format string"),
      ConfigError::WrapperExecutableNotFound(path) => {
        write!(
          f,
          "Session wrapper executable not found: {}",
          path.display()
        )
      },
    }
  }
}

impl std::error::Error for ConfigError {}

impl From<io::Error> for ConfigError {
  fn from(err: io::Error) -> Self {
    ConfigError::Io(err)
  }
}

impl From<toml::de::Error> for ConfigError {
  fn from(err: toml::de::Error) -> Self {
    ConfigError::Parse(err)
  }
}
