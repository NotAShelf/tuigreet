use ansi_to_tui::IntoText;
use tui::{
  prelude::Rect,
  text::Text,
  widgets::{Paragraph, Wrap},
};
use tuigreet_types::Mode;

use crate::Greeter;

pub fn titleize(message: &str) -> String {
  format!(" {message} ")
}

pub fn buttonize(message: &str) -> String {
  format!(" {message}")
}

// Determinew whether the cursor should be shown or hidden from the current
// mode and configuration. Usually, we will show the cursor only when expecting
// text entries from the user.
pub fn should_hide_cursor(greeter: &Greeter) -> bool {
  greeter.working
    || greeter.done
    || (greeter.user_menu.enabled
      && greeter.mode == Mode::Username
      && greeter.username.value.is_empty())
    || (greeter.mode == Mode::Password && greeter.prompt.is_none())
    || greeter.mode == Mode::Users
    || greeter.mode == Mode::Sessions
    || greeter.mode == Mode::Power
    || greeter.mode == Mode::Processing
    || greeter.mode == Mode::Action
}

// Computes the height of the main window where we display content, depending on
// the mode and spacing configuration.
//
// +------------------------+
// |                        | <- container padding
// |        Greeting        | <- greeting height
// |                        | <- auto-padding if greeting
// | Username:              | <- username
// | Password:              | <- password if prompt == Some(_)
// |                        | <- container padding
// +------------------------+
pub fn get_height(greeter: &Greeter) -> u16 {
  let (_, greeting_height) = get_greeting_height(greeter, 1, 0);
  let container_padding = greeter.layout.container_padding;
  let prompt_padding = greeter.layout.prompt_padding;

  let initial = match greeter.mode {
    Mode::Username | Mode::Action | Mode::Command => {
      container_padding.saturating_mul(2).saturating_add(1)
    },
    Mode::Password => {
      match greeter.prompt {
        Some(_) => {
          container_padding
            .saturating_mul(2)
            .saturating_add(prompt_padding)
            .saturating_add(2)
        },
        None => container_padding.saturating_mul(2).saturating_add(1),
      }
    },
    Mode::Users
    | Mode::Sessions
    | Mode::Power
    | Mode::Background
    | Mode::Processing => container_padding.saturating_mul(2),
  };

  match greeter.mode {
    Mode::Command
    | Mode::Sessions
    | Mode::Power
    | Mode::Background
    | Mode::Processing => initial,
    _ => initial.saturating_add(greeting_height),
  }
}

// Get the coordinates and size of the main window area, from the terminal size,
// and the content we need to display.
pub fn get_rect_bounds(
  greeter: &Greeter,
  area: Rect,
  items: usize,
) -> (u16, u16, u16, u16) {
  let width = greeter.layout.width.min(area.width);
  let box_height = get_height(greeter)
    .saturating_add(u16::try_from(items).unwrap_or(u16::MAX))
    .min(area.height);

  // Account for the message area rendered below the container so that the
  // combined block (box + message) is centered rather than just the box.
  let (_, message_height) =
    get_message_height(greeter, greeter.layout.container_padding, 0);
  let total_height = box_height.saturating_add(message_height);

  let x = if width < area.width {
    (area.width - width) / 2
  } else {
    0
  };
  let y = if total_height < area.height {
    (area.height - total_height) / 2
  } else {
    0
  };

  let (x, width) = if (x + width) >= area.width {
    (0, area.width)
  } else {
    (x, width)
  };
  let (y, box_height) = if y.saturating_add(total_height) >= area.height {
    (0, box_height.min(area.height))
  } else {
    (y, box_height)
  };

  (x, y, width, box_height)
}

// Computes the size of a text entry, from the container width and, if
// applicable, the prompt length.
pub fn get_input_width(
  greeter: &Greeter,
  width: u16,
  label: &Option<String>,
) -> u16 {
  let width = std::cmp::min(greeter.layout.width, width);

  let label_width = match label {
    None => 0,
    Some(label) => label.chars().count(),
  };

  width
    .saturating_sub(u16::try_from(label_width).unwrap_or(u16::MAX))
    .saturating_sub(5)
}

pub const fn get_cursor_offset(greeter: &mut Greeter, length: usize) -> i16 {
  let mut offset = length as i16 + greeter.cursor_offset;

  if offset < 0 {
    offset = 0;
    greeter.cursor_offset = -(length as i16);
  }

  if offset > length as i16 {
    offset = length as i16;
    greeter.cursor_offset = 0;
  }

  offset
}

pub fn get_greeting_height(
  greeter: &Greeter,
  padding: u16,
  fallback: u16,
) -> (Option<Paragraph<'_>>, u16) {
  if let Some(greeting) = &greeter.greeting {
    let width = greeter.layout.width;

    let text = match greeting.clone().into_text() {
      Ok(text) => text,
      Err(_) => Text::raw(greeting),
    };

    let paragraph = Paragraph::new(text.clone()).wrap(Wrap { trim: false });
    let height = paragraph
      .line_count(width.saturating_sub(padding.saturating_mul(2)).max(1))
      .saturating_add(1);

    (Some(paragraph), height as u16)
  } else {
    (None, fallback)
  }
}

pub fn get_message_height(
  greeter: &Greeter,
  padding: u16,
  fallback: u16,
) -> (Option<Paragraph<'_>>, u16) {
  if let Some(message) = &greeter.message {
    let width = greeter.layout.width;
    let paragraph =
      Paragraph::new(message.trim_end()).wrap(Wrap { trim: true });
    let height = paragraph.line_count(width.saturating_sub(4).max(1));

    (
      Some(paragraph),
      u16::try_from(height)
        .unwrap_or(u16::MAX)
        .saturating_add(padding),
    )
  } else {
    (None, fallback)
  }
}

#[cfg(test)]
mod test {
  use tui::{
    prelude::Rect,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
  };
  use tuigreet_types::Mode;

  use super::{get_input_width, get_rect_bounds};
  use crate::{
    Greeter,
    ui::util::{get_greeting_height, get_height},
  };

  // +-----------+
  // | Username: |
  // +-----------+
  #[test]
  fn test_container_height_username_padding_zero() {
    let mut greeter = Greeter::default();
    greeter.layout.container_padding = 0;
    greeter.mode = Mode::Username;

    assert_eq!(get_height(&greeter), 1);
  }

  #[test]
  fn tiny_width_has_no_underflow() {
    let greeter = Greeter::default();
    assert_eq!(get_input_width(&greeter, 1, &Some("Username".into())), 0);
    assert_eq!(
      get_rect_bounds(&greeter, Rect::new(0, 0, 1, 1), 0),
      (0, 0, 1, 1)
    );
  }

  // +-----------+
  // |           |
  // | Username: |
  // |           |
  // +-----------+
  #[test]
  fn test_container_height_username_padding_one() {
    let mut greeter = Greeter::default();
    greeter.layout.container_padding = 1;
    greeter.mode = Mode::Username;

    assert_eq!(get_height(&greeter), 3);
  }

  // +-----------+
  // |           |
  // | Greeting  |
  // |           |
  // | Username: |
  // |           |
  // +-----------+
  #[test]
  fn test_container_height_username_greeting_padding_one() {
    let mut greeter = Greeter::default();
    greeter.layout.container_padding = 1;
    greeter.greeting = Some("Hello".into());
    greeter.mode = Mode::Username;

    assert_eq!(get_height(&greeter), 5);
  }

  // +-----------+
  // |           |
  // | Greeting  |
  // |           |
  // | Username: |
  // |           |
  // | Password: |
  // |           |
  // +-----------+
  #[test]
  fn test_container_height_password_greeting_padding_one_prompt_padding_1() {
    let mut greeter = Greeter::default();
    greeter.layout.container_padding = 1;
    greeter.greeting = Some("Hello".into());
    greeter.mode = Mode::Password;
    greeter.prompt = Some("Password:".into());

    assert_eq!(get_height(&greeter), 7);
  }

  // +-----------+
  // |           |
  // | Greeting  |
  // |           |
  // | Username: |
  // | Password: |
  // |           |
  // +-----------+
  #[test]
  fn test_container_height_password_greeting_padding_one_prompt_padding_0() {
    let mut greeter = Greeter::default();
    greeter.layout.container_padding = 1;
    greeter.layout.prompt_padding = 0;
    greeter.greeting = Some("Hello".into());
    greeter.mode = Mode::Password;
    greeter.prompt = Some("Password:".into());

    assert_eq!(get_height(&greeter), 6);
  }

  #[test]
  fn test_rect_bounds() {
    let mut greeter = Greeter::default();
    greeter.layout.width = 50;

    let (x, y, width, height) =
      get_rect_bounds(&greeter, Rect::new(0, 0, 100, 100), 1);

    assert_eq!(x, 25);
    assert_eq!(y, 47);
    assert_eq!(width, 50);
    assert_eq!(height, 6);
  }

  #[test]
  fn test_rect_bounds_with_message() {
    // With a message present, the combined block (box + message) should be
    // centered, not just the box. The box itself is unchanged; only `y` shifts
    // upward to leave room for the message below.
    //
    // Setup: default config (width=80, container_padding=2), Mode::Username,
    //        message="Wrong password" (fits on one line at width 76).
    //
    // box_height     = (2*2) + 1 = 5
    // message_height = line_count(76) + padding = 1 + 2 = 3
    // total_height   = 8
    // y = (100 - 8) / 2 = 46  (vs. (100 - 5) / 2 = 47 without the fix)
    let mut greeter = Greeter::default();
    greeter.message = Some("Wrong password".into());

    let (x, y, width, height) =
      get_rect_bounds(&greeter, Rect::new(0, 0, 100, 100), 0);

    assert_eq!(x, 10); // (100 - 80) / 2
    assert_eq!(y, 46); // (100 - 8) / 2, not (100 - 5) / 2 = 47
    assert_eq!(width, 80);
    assert_eq!(height, 5);
  }

  #[test]
  fn input_width() {
    let mut greeter = Greeter::default();
    greeter.layout.width = 40;
    greeter.layout.container_padding = 1;

    let input_width = get_input_width(&greeter, 40, &Some("Username:".into()));

    assert_eq!(input_width, 26);
  }

  #[test]
  fn greeting_height_one_line() {
    let mut greeter = Greeter::default();
    greeter.layout.width = 15;
    greeter.layout.container_padding = 1;
    greeter.greeting = Some("Hello World".into());

    let (_, height) = get_greeting_height(&greeter, 1, 0);

    assert_eq!(height, 2);
  }

  #[test]
  fn greeting_height_two_lines() {
    let mut greeter = Greeter::default();
    greeter.layout.width = 8;
    greeter.layout.container_padding = 1;
    greeter.greeting = Some("Hello World".into());

    let (_, height) = get_greeting_height(&greeter, 1, 0);

    assert_eq!(height, 3);
  }

  #[test]
  fn ansi_greeting_height_one_line() {
    let mut greeter = Greeter::default();
    greeter.layout.width = 15;
    greeter.layout.container_padding = 1;
    greeter.greeting = Some("\x1b[31mHello\x1b[0m World".into());

    let (text, height) = get_greeting_height(&greeter, 1, 0);

    let expected = Paragraph::new(Text::from(vec![Line::from(vec![
      Span::styled("Hello", Style::default().fg(Color::Red)),
      Span::styled(" World", Style::reset()),
    ])]))
    .wrap(Wrap { trim: false });

    assert_eq!(text, Some(expected));
    assert_eq!(height, 2);
  }

  #[test]
  fn ansi_greeting_height_two_lines() {
    let mut greeter = Greeter::default();
    greeter.layout.width = 8;
    greeter.layout.container_padding = 1;
    greeter.greeting = Some("\x1b[31mHello\x1b[0m World".into());

    let (text, height) = get_greeting_height(&greeter, 1, 0);

    let expected = Paragraph::new(Text::from(vec![Line::from(vec![
      Span::styled("Hello", Style::default().fg(Color::Red)),
      Span::styled(" World", Style::reset()),
    ])]))
    .wrap(Wrap { trim: false });

    assert_eq!(text, Some(expected));
    assert_eq!(height, 3);
  }

  #[test]
  fn greeting_preserves_trailing_whitespace() {
    let mut greeter = Greeter::default();
    greeter.layout.width = 30;
    greeter.layout.container_padding = 1;
    // Simulate /etc/issue with trailing spaces for ASCII art alignment
    greeter.greeting = Some("Hello     \nWorld    ".into());

    let (text, height) = get_greeting_height(&greeter, 1, 0);

    let expected = Paragraph::new(Text::from(vec![
      Line::from("Hello     "),
      Line::from("World    "),
    ]))
    .wrap(Wrap { trim: false });

    assert_eq!(text, Some(expected));
    assert_eq!(height, 3);
  }
}
