use std::str::FromStr;

use tui::style::{Color, Style};
use tuigreet_config::config::ThemeConfig;

/// Color component (foreground or background)
#[derive(Clone)]
enum Component {
  Bg,
  Fg,
}

/// UI element that can be themed
pub enum Themed {
  Container,
  Time,
  Text,
  Border,
  Title,
  Greet,
  Prompt,
  Input,
  Action,
  ActionButton,
}

/// Color theme for all UI elements
#[derive(Default)]
pub struct Theme {
  container: Option<(Component, Color)>,
  time:      Option<(Component, Color)>,
  text:      Option<(Component, Color)>,
  border:    Option<(Component, Color)>,
  title:     Option<(Component, Color)>,
  greet:     Option<(Component, Color)>,
  prompt:    Option<(Component, Color)>,
  input:     Option<(Component, Color)>,
  action:    Option<(Component, Color)>,
  button:    Option<(Component, Color)>,
}

impl Theme {
  /// Parse theme from ThemeConfig.
  ///
  /// * `theme_config` - ThemeConfig struct
  ///
  /// # Returns
  ///
  /// Theme with parsed colors, using fallbacks for unspecified elements
  #[must_use]
  pub fn from_config(theme_config: &ThemeConfig) -> Self {
    use Component::{Bg, Fg};

    let mut style = Self::default();

    if let Ok(color) =
      Color::from_str(theme_config.container.as_deref().unwrap_or_default())
    {
      style.container = Some((Bg, color));
    }
    if let Ok(color) =
      Color::from_str(theme_config.time.as_deref().unwrap_or_default())
    {
      style.time = Some((Fg, color));
    }
    if let Ok(color) =
      Color::from_str(theme_config.border.as_deref().unwrap_or_default())
    {
      style.border = Some((Fg, color));
    }
    if let Ok(color) =
      Color::from_str(theme_config.title.as_deref().unwrap_or_default())
    {
      style.title = Some((Fg, color));
    }
    if let Ok(color) =
      Color::from_str(theme_config.greet.as_deref().unwrap_or_default())
    {
      style.greet = Some((Fg, color));
    }
    if let Ok(color) =
      Color::from_str(theme_config.prompt.as_deref().unwrap_or_default())
    {
      style.prompt = Some((Fg, color));
    }
    if let Ok(color) =
      Color::from_str(theme_config.input.as_deref().unwrap_or_default())
    {
      style.input = Some((Fg, color));
    }
    if let Ok(color) =
      Color::from_str(theme_config.action.as_deref().unwrap_or_default())
    {
      style.action = Some((Fg, color));
    }
    if let Ok(color) =
      Color::from_str(theme_config.button.as_deref().unwrap_or_default())
    {
      style.button = Some((Fg, color));
    }

    if style.time.is_none() {
      style.time.clone_from(&style.text);
    }
    if style.greet.is_none() {
      style.greet.clone_from(&style.text);
    }
    if style.title.is_none() {
      style.title.clone_from(&style.border);
    }
    if style.button.is_none() {
      style.button.clone_from(&style.action);
    }

    style
  }

  /// Builds a style by applying each target's configured color in order.
  ///
  /// Later targets override earlier targets when they affect the same style
  /// property.
  #[must_use]
  pub fn of(&self, targets: &[Themed]) -> Style {
    targets
      .iter()
      .fold(Style::default(), |style, target| self.apply(style, target))
  }

  const fn apply(&self, style: Style, target: &Themed) -> Style {
    use Themed::{
      Action,
      ActionButton,
      Border,
      Container,
      Greet,
      Input,
      Prompt,
      Text,
      Time,
      Title,
    };

    let color = match target {
      Container => &self.container,
      Time => &self.time,
      Text => &self.text,
      Border => &self.border,
      Title => &self.title,
      Greet => &self.greet,
      Prompt => &self.prompt,
      Input => &self.input,
      Action => &self.action,
      ActionButton => &self.button,
    };

    match color {
      Some((component, color)) => {
        match component {
          Component::Fg => style.fg(*color),
          Component::Bg => style.bg(*color),
        }
      },

      None => style,
    }
  }
}
