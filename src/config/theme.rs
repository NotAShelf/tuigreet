use crate::{config::schema::ThemeConfig, theme::Theme};

/// Parse theme from TOML configuration.
///
/// # Arguments
///
/// * `theme_config` - Theme configuration from TOML file
///
/// # Returns
///
/// Theme with colors specified in config, defaulting unspecified elements
pub fn theme_from_config(theme_config: &ThemeConfig) -> Theme {
  let mut theme = Theme::default();

  // Build theme spec string from config
  let mut spec_parts = Vec::new();

  if let Some(ref color) = theme_config.border {
    spec_parts.push(format!("border={}", color));
  }
  if let Some(ref color) = theme_config.text {
    spec_parts.push(format!("text={}", color));
  }
  if let Some(ref color) = theme_config.time {
    spec_parts.push(format!("time={}", color));
  }
  if let Some(ref color) = theme_config.container {
    spec_parts.push(format!("container={}", color));
  }
  if let Some(ref color) = theme_config.title {
    spec_parts.push(format!("title={}", color));
  }
  if let Some(ref color) = theme_config.greet {
    spec_parts.push(format!("greet={}", color));
  }
  if let Some(ref color) = theme_config.prompt {
    spec_parts.push(format!("prompt={}", color));
  }
  if let Some(ref color) = theme_config.input {
    spec_parts.push(format!("input={}", color));
  }
  if let Some(ref color) = theme_config.action {
    spec_parts.push(format!("action={}", color));
  }
  if let Some(ref color) = theme_config.button {
    spec_parts.push(format!("button={}", color));
  }

  if !spec_parts.is_empty() {
    let spec = spec_parts.join(";");
    theme = Theme::parse(&spec);
  }

  theme
}

/// Apply CLI theme string over config theme.
///
/// CLI theme completely overrides config theme if present.
///
/// # Arguments
///
/// * `base_theme` - Theme from config file
/// * `cli_theme_spec` - Optional theme string from CLI (e.g., "--theme
///   border=blue;text=white")
///
/// # Returns
///
/// CLI theme if specified, otherwise base theme
pub fn apply_cli_theme(
  mut base_theme: Theme,
  cli_theme_spec: Option<&str>,
) -> Theme {
  if let Some(spec) = cli_theme_spec {
    // CLI theme completely overrides config theme
    base_theme = Theme::parse(spec);
  }
  base_theme
}
