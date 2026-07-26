use std::{
  collections::HashSet,
  fs,
  path::{Path, PathBuf},
};

use crate::{
  config::{Config, SecretMode, WidgetPosition},
  error::ConfigError,
};

impl Config {
  /// Validates rules on the fully-resolved config, after TOML,
  /// env, and CLI are merged
  ///
  /// Returns non-fatal configuration warnings when validation succeeds.
  ///
  /// # Errors
  ///
  /// Returns an error when mutually exclusive settings, invalid ranges,
  /// malformed values, or invalid wrapper commands are configured.
  pub fn validate(
    &self,
    validate_wrappers: bool,
  ) -> Result<Vec<ConfigError>, (Vec<ConfigError>, Vec<ConfigError>)> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Check mutually exclusive options
    if self.display.issue && self.display.greeting.is_some() {
      errors.push(ConfigError::MutuallyExclusive(
        "display.issue".to_string(),
        "display.greeting".to_string(),
      ));
    }

    // Check dependencies
    if self.remember.user_session && !self.remember.username {
      errors.push(ConfigError::Dependency(
        "remember.user_session requires remember.username".to_string(),
      ));
    }

    // Check UID ranges
    if self.user_menu.min_uid > self.user_menu.max_uid {
      errors.push(ConfigError::InvalidRange(
        "user_menu.min_uid must not exceed user_menu.max_uid".to_string(),
      ));
    }

    if self.layout.width == 0 {
      errors.push(ConfigError::Validation(
        "layout.width must be greater than 0".to_string(),
      ));
    }
    for (name, padding) in [
      ("layout.window_padding", self.layout.window_padding),
      ("layout.container_padding", self.layout.container_padding),
      ("layout.prompt_padding", self.layout.prompt_padding),
    ] {
      if padding > 10 {
        errors.push(ConfigError::Validation(format!(
          "{name} must not exceed 10"
        )));
      }
    }

    // Check keybindings are distinct
    let keys = [
      self.keybindings.command,
      self.keybindings.sessions,
      self.keybindings.power,
      self.keybindings.background,
    ];
    if keys.iter().collect::<HashSet<_>>().len() != keys.len() {
      errors.push(ConfigError::DuplicateKeybindings);
    }

    // Check F-key ranges
    for (name, key) in [
      ("command", self.keybindings.command),
      ("sessions", self.keybindings.sessions),
      ("power", self.keybindings.power),
      ("background", self.keybindings.background),
    ] {
      if !(1..=12).contains(&key) {
        errors.push(ConfigError::InvalidFKey(name.to_string(), key));
      }
    }

    // Validate time format if provided
    if let Some(ref format) = self.display.time_format
      && chrono::format::StrftimeItems::new(format)
        .any(|item| matches!(item, chrono::format::Item::Error))
    {
      errors.push(ConfigError::InvalidTimeFormat);
    }

    // Validate session wrapper executables if requested
    if validate_wrappers {
      if let Some(ref wrapper) = self.session.session_wrapper
        && let Some(error) = self.validate_wrapper_command(wrapper)
      {
        errors.push(error);
      }
      if !self.session.xsession_wrapper.is_empty()
        && let Some(error) =
          self.validate_wrapper_command(&self.session.xsession_wrapper)
      {
        errors.push(error);
      }
    }

    // Validate [[outputs]] entries
    {
      let primary_count = self.outputs.iter().filter(|o| o.primary).count();
      if primary_count > 1 {
        errors.push(ConfigError::Validation(format!(
          "At most one output may be marked `primary = true`, but \
           {primary_count} are"
        )));
      }

      for output in &self.outputs {
        if output.connector.contains('/') || output.connector.contains("..") {
          errors.push(ConfigError::Validation(format!(
            "Output connector name '{}' must not contain path separators",
            output.connector
          )));
        }
        if output.connector.is_empty() {
          errors.push(ConfigError::Validation(
            "Output connector name must not be empty".to_string(),
          ));
        }
      }

      // Warn if [[outputs]] is configured but all are disabled
      if !self.outputs.is_empty() && self.outputs.iter().all(|o| !o.enabled) {
        warnings.push(ConfigError::Warning(
          "All [[outputs]] entries have `enabled = false`; no output will be \
           used for terminal sizing"
            .to_string(),
        ));
      }
    }
    if self.secret.mode == SecretMode::Characters
      && self.secret.characters.is_empty()
    {
      errors.push(ConfigError::Validation(
        "When using characters secret mode, cannot have empty secret \
         characters"
          .to_string(),
      ));
    }

    for env in &self.session.environments {
      if !env.contains('=') {
        errors.push(ConfigError::Validation(format!(
          "malformed environment variable definition for '{env}'"
        )));
      }
    }

    // Validate [terminal].
    // Both cols and rows must be set together
    match (self.terminal.cols, self.terminal.rows) {
      (Some(_), None) => {
        errors.push(ConfigError::Validation(
          "`terminal.cols` is set but `terminal.rows` is missing; both must \
           be provided together"
            .to_string(),
        ));
      },
      (None, Some(_)) => {
        errors.push(ConfigError::Validation(
          "`terminal.rows` is set but `terminal.cols` is missing; both must \
           be provided together"
            .to_string(),
        ));
      },
      (Some(0), Some(_)) => {
        errors.push(ConfigError::Validation(
          "`terminal.cols` must be greater than 0".to_string(),
        ));
      },
      (Some(_), Some(0)) => {
        errors.push(ConfigError::Validation(
          "`terminal.rows` must be greater than 0".to_string(),
        ));
      },
      _ => (),
    }

    // Add validation warnings for potentially problematic configurations
    warnings.extend(self.check_warnings());

    if errors.is_empty() {
      Ok(warnings)
    } else {
      Err((errors, warnings))
    }
  }

  /// Check for configuration warnings
  fn check_warnings(&self) -> Vec<ConfigError> {
    let mut warnings = Vec::new();
    // Warn about excessively high padding values
    if self.layout.window_padding > 10 {
      warnings.push(ConfigError::Warning(format!(
        "window_padding is very high ({}), this may cause display issues",
        self.layout.window_padding
      )));
    }

    if self.layout.container_padding > 10 {
      warnings.push(ConfigError::Warning(format!(
        "container_padding is very high ({}), this may cause display issues",
        self.layout.container_padding
      )));
    }

    // Warn about very wide width settings
    if self.layout.width > 200 {
      warnings.push(ConfigError::Warning(format!(
        "width is very high ({}), this may cause display issues on smaller \
         terminals",
        self.layout.width
      )));
    }

    // Warn if user menu is enabled but UID range might be empty
    if self.user_menu.enabled && self.user_menu.min_uid > 65000 {
      warnings.push(ConfigError::Warning(
        "user_menu.min_uid is very high, you may not see any users".to_string(),
      ));
    }

    // Warn about potentially conflicting session directories
    let mut all_session_dirs = self.session.sessions_dirs.clone();
    all_session_dirs.extend(self.session.xsessions_dirs.clone());

    if all_session_dirs.len()
      != all_session_dirs
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len()
    {
      warnings.push(ConfigError::Warning(
        "Duplicate session directories detected in sessions_dirs and \
         xsessions_dirs"
          .to_string(),
      ));
    }

    // Warn about hidden widgets but enabled features
    if matches!(self.layout.widgets.time_position, WidgetPosition::Hidden)
      && self.display.show_time
    {
      warnings.push(ConfigError::Warning(
        "time widget is hidden but show_time is enabled".to_string(),
      ));
    }

    // Warn about potential security issues with power commands
    if let Some(ref cmd) = self.power.shutdown
      && !self.power.use_setsid
      && !cmd.contains("sudo")
      && !cmd.contains("doas")
    {
      warnings.push(ConfigError::Warning(
        "shutdown command without setsid or privilege escalation may fail"
          .to_string(),
      ));
    }

    if let Some(ref cmd) = self.power.reboot
      && !self.power.use_setsid
      && !cmd.contains("sudo")
      && !cmd.contains("doas")
    {
      warnings.push(ConfigError::Warning(
        "reboot command without setsid or privilege escalation may fail"
          .to_string(),
      ));
    }

    if let Some(ref cmd) = self.power.suspend
      && !self.power.use_setsid
      && !cmd.contains("sudo")
      && !cmd.contains("doas")
    {
      warnings.push(ConfigError::Warning(
        "suspend command without setsid or privilege escalation may fail"
          .to_string(),
      ));
    }

    if let Some(ref cmd) = self.power.hibernate
      && !self.power.use_setsid
      && !cmd.contains("sudo")
      && !cmd.contains("doas")
    {
      warnings.push(ConfigError::Warning(
        "hibernate command without setsid or privilege escalation may fail"
          .to_string(),
      ));
    }

    // Warn about empty session directories
    if self.session.sessions_dirs.is_empty()
      && self.session.xsessions_dirs.is_empty()
    {
      warnings.push(ConfigError::Warning(
        "No session directories configured, users may not be able to select \
         sessions"
          .to_string(),
      ));
    }

    // Warn about potentially invalid time formats
    if let Some(ref format) = self.display.time_format
      && format.is_empty()
    {
      warnings.push(ConfigError::Warning(
        "time_format is empty, this will result in no time display".to_string(),
      ));
    }

    // Warn about conflicting remember options
    if self.remember.session && self.remember.user_session {
      // This should be caught as an error above, but just in case
      warnings.push(ConfigError::Warning(
        "Both remember.session and remember.user_session are enabled"
          .to_string(),
      ));
    }

    // Warn about invalid fps settings
    if self.background.fps == 0 {
      warnings.push(ConfigError::Warning(
        "Background fps is set to 0, this will be ignored".to_string(),
      ));
    }
    warnings
  }

  /// Validate that a wrapper command exists and is executable
  fn validate_wrapper_command(&self, command: &str) -> Option<ConfigError> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
      return Some(ConfigError::WrapperExecutableNotFound(PathBuf::from(
        command,
      )));
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if let Some(executable) = parts.first() {
      // Check if it's an absolute path
      let path = PathBuf::from(executable);
      if path.is_absolute() {
        if !path.exists() || !is_executable(&path) {
          return Some(ConfigError::WrapperExecutableNotFound(path));
        }
      } else {
        // Search in PATH
        if !command_exists(executable) {
          return Some(ConfigError::WrapperExecutableNotFound(PathBuf::from(
            executable,
          )));
        }
      }
    }
    None
  }
}

/// Check if a file is executable
fn is_executable(path: &Path) -> bool {
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = fs::metadata(path) {
      let permissions = metadata.permissions();
      return permissions.mode() & 0o111 != 0;
    }
  }

  #[cfg(not(unix))]
  {
    // On non-Unix systems, just check if file exists
    return path.exists();
  }

  false
}

/// Check if a command exists in PATH
fn command_exists(command: &str) -> bool {
  if let Ok(path) = std::env::var("PATH") {
    for dir in std::env::split_paths(&path) {
      let full_path = dir.join(command);
      if full_path.exists() && is_executable(&full_path) {
        return true;
      }
    }
  }
  false
}
