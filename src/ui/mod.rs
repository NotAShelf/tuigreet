//! UI rendering and component modules

mod command;
pub mod common;
mod i18n;
pub mod power;
mod processing;
mod prompt;
pub mod sessions;
#[cfg(test)] mod tests;
pub mod users;
mod util;

use std::{
  borrow::Cow,
  error::Error,
  io::{self, Write},
  sync::Arc,
};

use chrono::prelude::*;
use sessions::SessionSource;
use tokio::sync::RwLock;
use tui::{
  Frame as CrosstermFrame,
  Terminal,
  layout::{Alignment, Constraint, Direction, Layout},
  style::Modifier,
  text::{Line, Span},
  widgets::Paragraph,
};
use tuigreet::{Mode, config::WidgetPosition};
use util::buttonize;

use self::common::style::{Theme, Themed};
pub use self::i18n::MESSAGES;
use crate::{Greeter, info::capslock_status, ui::util::should_hide_cursor};

const STATUSBAR_LEFT_INDEX: usize = 1;
const STATUSBAR_RIGHT_INDEX: usize = 2;

pub(super) type Frame<'a> = CrosstermFrame<'a>;

enum Button {
  Command,
  Session,
  Power,
  Other,
}

/// Get widget position from config
fn get_widget_position(greeter: &Greeter, widget_name: &str) -> WidgetPosition {
  if let Some(ref config) = greeter.loaded_config {
    match widget_name {
      "time" => config.layout.widgets.time_position.clone(),
      "status" => config.layout.widgets.status_position.clone(),
      _ => WidgetPosition::Default,
    }
  } else {
    WidgetPosition::Default
  }
}

pub async fn draw<B>(
  greeter: Arc<RwLock<Greeter>>,
  terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn Error>>
where
  B: tui::backend::Backend,
{
  let mut greeter = greeter.write().await;
  let hide_cursor = should_hide_cursor(&greeter);

  terminal.draw(|f| {
    let theme = &greeter.theme;
    let size = f.area();
    let time_position = get_widget_position(&greeter, "time");
    let status_position = get_widget_position(&greeter, "status");

    // Dynamic layout
    let mut constraints = vec![];
    let mut time_slot = None;
    let mut status_slot = None;

    // Top padding
    constraints.push(Constraint::Length(greeter.window_padding()));

    // Time at top (default behavior)
    if greeter.time
      && !matches!(
        time_position,
        WidgetPosition::Hidden | WidgetPosition::Bottom
      )
    {
      time_slot = Some(constraints.len());
      constraints.push(Constraint::Length(1));
    }

    // Status at top (if configured)
    if matches!(status_position, WidgetPosition::Top) {
      status_slot = Some(constraints.len());
      constraints.push(Constraint::Length(1));
    }

    // Main content area
    let main_slot = constraints.len();
    constraints.push(Constraint::Min(1));

    // Status at bottom (default behavior)
    if matches!(
      status_position,
      WidgetPosition::Default | WidgetPosition::Bottom
    ) {
      status_slot = Some(constraints.len());
      constraints.push(Constraint::Length(1));
    }

    // Time at bottom (if configured)
    if greeter.time && matches!(time_position, WidgetPosition::Bottom) {
      time_slot = Some(constraints.len());
      constraints.push(Constraint::Length(1));
    }

    // Bottom padding
    constraints.push(Constraint::Length(greeter.window_padding()));

    let chunks = Layout::default().constraints(constraints).split(size);

    // Render time widget if enabled and not hidden
    if let Some(slot) = time_slot {
      let time_text = Span::from(get_time(&greeter));
      let time = Paragraph::new(time_text)
        .alignment(Alignment::Center)
        .style(theme.of(&[Themed::Time]));

      f.render_widget(time, chunks[slot]);
    }

    // Render status bar if not hidden
    if let Some(slot) = status_slot {
      let status_block_size_right = 1
        + greeter.window_padding()
        + fl!("status_caps").chars().count() as u16;
      let status_block_size_left =
        (size.width - greeter.window_padding()) - status_block_size_right;

      let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
          [
            Constraint::Length(greeter.window_padding()),
            Constraint::Length(status_block_size_left),
            Constraint::Length(status_block_size_right),
            Constraint::Length(greeter.window_padding()),
          ]
          .as_ref(),
        )
        .split(chunks[slot]);

      let session_source_label = match greeter.session_source {
        SessionSource::Session(_) => fl!("status_session"),
        _ => fl!("status_command"),
      };

      let session_source =
        greeter.session_source.label(&greeter).unwrap_or("-");

      let status_left_text = Line::from(vec![
        status_label(theme, "ESC"),
        status_value(&greeter, theme, Button::Other, fl!("action_reset")),
        Span::from(" "),
        status_label(theme, format!("F{}", greeter.kb_command)),
        status_value(&greeter, theme, Button::Command, fl!("action_command")),
        Span::from(" "),
        status_label(theme, format!("F{}", greeter.kb_sessions)),
        status_value(&greeter, theme, Button::Session, fl!("action_session")),
        Span::from(" "),
        status_label(theme, format!("F{}", greeter.kb_power)),
        status_value(&greeter, theme, Button::Power, fl!("action_power")),
        Span::from(" "),
        status_label(theme, session_source_label),
        status_value(&greeter, theme, Button::Other, session_source),
      ]);
      let status_left = Paragraph::new(status_left_text);

      f.render_widget(status_left, status_chunks[STATUSBAR_LEFT_INDEX]);

      if capslock_status() {
        let status_right_text = status_label(theme, fl!("status_caps"));
        let status_right =
          Paragraph::new(status_right_text).alignment(Alignment::Right);

        f.render_widget(status_right, status_chunks[STATUSBAR_RIGHT_INDEX]);
      }
    }

    // Get the main content area and create a sub-frame for content drawing
    let main_area = chunks[main_slot];

    let cursor = match greeter.mode {
      Mode::Command => {
        self::command::draw_with_area(&mut greeter, f, main_area).ok()
      },
      Mode::Sessions => {
        greeter.sessions.draw_with_area(&greeter, f, main_area).ok()
      },
      Mode::Power => greeter.powers.draw_with_area(&greeter, f, main_area).ok(),
      Mode::Users => greeter.users.draw_with_area(&greeter, f, main_area).ok(),
      Mode::Processing => {
        self::processing::draw_with_area(&mut greeter, f, main_area).ok()
      },
      _ => self::prompt::draw_with_area(&mut greeter, f, main_area).ok(),
    };

    if !hide_cursor && let Some(cursor) = cursor {
      f.set_cursor_position((cursor.0 - 1, cursor.1 - 1));
    }
  })?;

  io::stdout().flush()?;

  Ok(())
}

fn get_time(greeter: &Greeter) -> String {
  let format = match &greeter.time_format {
    Some(format) => Cow::Borrowed(format),
    None => Cow::Owned(fl!("date")),
  };

  Local::now()
    .format_localized(&format, greeter.locale)
    .to_string()
}

fn status_label<'s, S>(theme: &Theme, text: S) -> Span<'s>
where
  S: Into<String>,
{
  Span::styled(
    text.into(),
    theme
      .of(&[Themed::ActionButton])
      .add_modifier(Modifier::REVERSED),
  )
}

fn status_value<'s, S>(
  greeter: &Greeter,
  theme: &Theme,
  button: Button,
  text: S,
) -> Span<'s>
where
  S: Into<String>,
{
  let relevant_mode = match button {
    Button::Command => Mode::Command,
    Button::Session => Mode::Sessions,
    Button::Power => Mode::Power,

    _ => {
      return Span::from(buttonize(&text.into()))
        .style(theme.of(&[Themed::Action]));
    },
  };

  let style = match greeter.mode == relevant_mode {
    true => {
      theme
        .of(&[Themed::ActionButton])
        .add_modifier(Modifier::REVERSED)
    },
    false => theme.of(&[Themed::Action]),
  };

  Span::from(buttonize(&text.into())).style(style)
}

fn prompt_value<'s, S>(theme: &Theme, text: Option<S>) -> Span<'s>
where
  S: Into<String>,
{
  match text {
    Some(text) => {
      Span::styled(
        text.into(),
        theme.of(&[Themed::Prompt]).add_modifier(Modifier::BOLD),
      )
    },
    None => Span::from(""),
  }
}
