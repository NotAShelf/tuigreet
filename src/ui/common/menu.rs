use std::{borrow::Cow, error::Error};

use tui::{
  prelude::Rect,
  style::{Modifier, Style},
  text::Span,
  widgets::{Block, BorderType, Borders, Paragraph},
};

use super::style::Themed;
use crate::{
  Greeter,
  ui::{
    Frame,
    util::{get_rect_bounds, titleize},
  },
};

/// Item that can be displayed in a menu.
pub trait MenuItem {
  /// Format the item for display.
  fn format(&self) -> Cow<'_, str>;
}

/// Generic menu widget for displaying selectable options.
#[derive(Default)]
pub struct Menu<T>
where
  T: MenuItem,
{
  /// Menu title
  pub title:    String,
  /// List of menu items
  pub options:  Vec<T>,
  /// Currently selected index
  pub selected: usize,
}

impl<T> Menu<T>
where
  T: MenuItem,
{
  /// Draw the menu within a specified area.
  ///
  /// # Returns
  ///
  /// Tuple of `(cursor_x, cursor_y)` for the selected item
  pub fn draw_with_area(
    &self,
    greeter: &Greeter,
    f: &mut Frame,
    area: Rect,
  ) -> Result<(u16, u16), Box<dyn Error>> {
    let theme = &greeter.theme;

    let size = area;
    let (x, y, width, height) =
      get_rect_bounds(greeter, size, self.options.len());

    let container = Rect::new(x, y, width, height);

    let title = Span::from(titleize(&self.title));
    let block = Block::default()
      .title(title)
      .title_style(theme.of(&[Themed::Title]))
      .style(theme.of(&[Themed::Container]))
      .borders(Borders::ALL)
      .border_type(BorderType::Plain)
      .border_style(theme.of(&[Themed::Border]));

    for (index, option) in self.options.iter().enumerate() {
      let name = option.format();
      let name = format!("{:1$}", name, greeter.width() as usize - 4);

      let frame = Rect::new(x + 2, y + 2 + index as u16, width - 4, 1);
      let option_text = self.get_option(name, index);
      let option = Paragraph::new(option_text);

      f.render_widget(option, frame);
    }

    f.render_widget(block, container);

    Ok((1, 1))
  }

  fn get_option<'g, S>(&self, name: S, index: usize) -> Span<'g>
  where
    S: Into<String>,
  {
    if self.selected == index {
      Span::styled(
        name.into(),
        Style::default().add_modifier(Modifier::REVERSED),
      )
    } else {
      Span::from(name.into())
    }
  }
}
