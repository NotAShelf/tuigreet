use std::{fmt, path::PathBuf};

/// Errors that can occur during configuration loading or validation
#[derive(Debug)]
pub enum ConfigError {
  /// Error surfaced by figment while merging/deserializing TOML, env, or
  /// CLI-baseline layers (covers I/O errors reading `config.toml`, TOML
  /// parse errors, and type-mismatch deserialization errors).
  Figment(figment::Error),

  /// General validation error with description
  Validation(String),

  /// Two options that cannot be used together
  MutuallyExclusive(String, String),

  /// Option that depends on another option being set
  Dependency(String),

  /// Invalid range (e.g., `min_uid` >= `max_uid`)
  InvalidRange(String),

  /// Duplicate keybindings
  DuplicateKeybindings,

  /// Invalid F-key value
  InvalidFKey(String, u8),

  /// Invalid time format string
  InvalidTimeFormat,

  /// Warning, non fatal error
  Warning(String),

  /// Session wrapper executable not found
  WrapperExecutableNotFound(PathBuf),
}

impl fmt::Display for ConfigError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Figment(err) => write!(f, "config error: {err}"),
      Self::Validation(msg) => write!(f, "validation error: {msg}"),
      Self::MutuallyExclusive(opt1, opt2) => {
        write!(f, "options '{opt1}' and '{opt2}' are mutually exclusive")
      },
      Self::Dependency(msg) => write!(f, "dependency error: {msg}"),
      Self::InvalidRange(msg) => write!(f, "invalid range: {msg}"),
      Self::DuplicateKeybindings => {
        write!(f, "duplicate keybindings detected")
      },
      Self::InvalidFKey(name, key) => {
        write!(
          f,
          "invalid F-key value for '{name}': F{key} (must be F1-F12)"
        )
      },
      Self::InvalidTimeFormat => write!(f, "invalid time format string"),
      Self::Warning(msg) => {
        write!(f, "non-fatal config issue found: {msg}")
      },
      Self::WrapperExecutableNotFound(path) => {
        write!(
          f,
          "session wrapper executable not found: {}",
          path.display()
        )
      },
    }
  }
}

impl From<figment::Error> for ConfigError {
  fn from(err: figment::Error) -> Self {
    Self::Figment(err)
  }
}
