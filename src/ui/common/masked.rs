use zeroize::Zeroize;

/// String that can be masked for display (e.g., password fields).
#[derive(Default)]
pub struct MaskedString {
  /// Actual value
  pub value: String,
  /// Optional masked display value (e.g., "****")
  pub mask:  Option<String>,
}

impl MaskedString {
  /// Create a new masked string.
  pub fn from(value: String, mask: Option<String>) -> MaskedString {
    MaskedString { value, mask }
  }

  /// Get the display value (mask if present, otherwise actual value).
  pub fn get(&self) -> &str {
    match self.mask {
      Some(ref mask) => mask,
      None => &self.value,
    }
  }

  /// Securely erase both value and mask from memory.
  pub fn zeroize(&mut self) {
    self.value.zeroize();

    if let Some(ref mut mask) = self.mask {
      mask.zeroize();
    }

    self.mask = None;
  }
}

#[cfg(test)]
mod tests {
  use super::MaskedString;

  #[test]
  fn get_value_when_unmasked() {
    let masked = MaskedString::from("value".to_string(), None);

    assert_eq!(masked.get(), "value");
  }

  #[test]
  fn get_mask_when_masked() {
    let masked =
      MaskedString::from("value".to_string(), Some("mask".to_string()));

    assert_eq!(masked.get(), "mask");
  }
}
