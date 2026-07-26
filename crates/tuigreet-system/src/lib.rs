/// DRM output enumeration and terminal-sizing support.
///
/// Outputs (connectors) are discovered through the sysfs DRM class
/// (`/sys/class/drm/`), which is world-readable and requires no special
/// privileges.  Terminal sizing is applied via the `TIOCSWINSZ` ioctl so
/// that the TUI fills the physical monitor rather than the TTY's default
/// dimensions.
use std::{fs, process};

#[cfg(not(test))]
use tuigreet_config::config::{OutputConfig, TerminalConfig};

/// Raw FFI bindings for the two terminal-sizing ioctls.
///
/// These are Linux-specific and only compiled outside the test harness to avoid
/// needing a libc dependency.
#[cfg(not(test))]
mod ffi {
  use std::ffi::{c_int, c_ulong};

  /// `ioctl(TIOCGWINSZ, ...)` request code (Linux, most architectures).
  pub const TIOCGWINSZ: c_ulong = 0x5413;

  /// `ioctl(TIOCSWINSZ, ...)` request code (Linux, most architectures).
  pub const TIOCSWINSZ: c_ulong = 0x5414;

  #[repr(C)]
  pub struct WinSize {
    pub ws_row:    u16,
    pub ws_col:    u16,
    pub ws_xpixel: u16,
    pub ws_ypixel: u16,
  }

  unsafe extern "C" {
    pub fn ioctl(fd: c_int, request: c_ulong, arg: *mut WinSize) -> c_int;
  }
}

/// Information about a single DRM connector as read from sysfs.
pub struct DrmOutput {
  /// Connector name without the card prefix (e.g. `"DP-1"`, `"HDMI-A-1"`).
  pub connector: String,

  /// DRM card device name (e.g. `"card0"`, `"card1"`).
  pub card: String,

  /// Whether a display is physically connected.
  pub connected: bool,

  /// The connector's preferred/native resolution in pixels `(width, height)`,
  /// taken from the first line of the sysfs `modes` file.
  pub native_resolution: Option<(u16, u16)>,
}

/// Enumerate all DRM connectors present in `/sys/class/drm/`.
///
/// Returns an empty list if the path does not exist or cannot be read.
/// Entries are sorted by card name then connector name.
pub fn enumerate_outputs() -> Vec<DrmOutput> {
  enumerate_outputs_from(std::path::Path::new("/sys/class/drm"))
}

/// Inner implementation of [`enumerate_outputs`] that accepts an arbitrary
/// sysfs DRM root path, allowing unit tests to use a temporary directory.
pub fn enumerate_outputs_from(drm_path: &std::path::Path) -> Vec<DrmOutput> {
  let mut outputs = Vec::new();

  let Ok(entries) = fs::read_dir(drm_path) else {
    return outputs;
  };

  for entry in entries.flatten() {
    let name = entry.file_name().to_string_lossy().into_owned();

    // Connector directories are named like "card0-DP-1"; skip plain "cardN"
    // and any non-connector entries.
    let Some(dash_pos) = name.find('-') else {
      continue;
    };

    let card = name[..dash_pos].to_string();
    let connector = name[dash_pos + 1..].to_string();

    if !card.starts_with("card") {
      continue;
    }

    let entry_path = entry.path();

    // Only process directories that have a `status` file (i.e. connector
    // directories, not e.g. the `version` file).
    let status_path = entry_path.join("status");
    if !status_path.exists() {
      continue;
    }

    let connected =
      fs::read_to_string(&status_path).is_ok_and(|s| s.trim() == "connected");

    // First line of `modes` is the preferred/native resolution.
    let modes_path = entry_path.join("modes");
    let native_resolution = fs::read_to_string(modes_path)
      .ok()
      .and_then(|content| content.lines().next().and_then(parse_resolution));

    outputs.push(DrmOutput {
      connector,
      card,
      connected,
      native_resolution,
    });
  }

  outputs
    .sort_by(|a, b| a.card.cmp(&b.card).then(a.connector.cmp(&b.connector)));
  outputs
}

/// Parse a resolution string of the form `"WIDTHxHEIGHT"` into `(u16, u16)`.
pub fn parse_resolution(mode: &str) -> Option<(u16, u16)> {
  let mode = mode.trim();
  let (w_str, h_str) = mode.split_once('x')?;
  let w = w_str.trim().parse().ok()?;
  let h = h_str.trim().parse().ok()?;

  Some((w, h))
}

/// Print available DRM outputs to stdout and exit.
///
/// Outputs are grouped by card. The format is human-readable:
/// ```text
/// DRM outputs (card1):
///   DP-1             connected      1920x1080
///   HDMI-A-1         disconnected
/// ```
pub fn list_outputs() -> ! {
  let outputs = enumerate_outputs();

  if outputs.is_empty() {
    println!("No DRM outputs found (is /sys/class/drm/ available?).");
    process::exit(0);
  }

  let mut current_card = String::new();
  for output in &outputs {
    if output.card != current_card {
      println!("DRM outputs ({}):", output.card);
      current_card.clone_from(&output.card);
    }

    let status = if output.connected {
      "connected"
    } else {
      "disconnected"
    };
    let resolution = output
      .native_resolution
      .map(|(w, h)| format!("{w}x{h}"))
      .unwrap_or_default();

    println!("  {:<16} {:<14} {}", output.connector, status, resolution);
  }

  process::exit(0);
}

/// Apply terminal sizing based on `[[outputs]]` and `[terminal]` config.
///
/// Priority (highest first):
/// 1. `[terminal] cols` + `rows`: explicit character-cell dimensions.
/// 2. Output marked `primary = true`: use its native pixel resolution to
///    compute character-cell dimensions from the current font size.
/// 3. First enabled output in `[[outputs]]`.
///
/// If the terminal's current pixel dimensions are unknown (e.g. not running
/// on a real TTY) the resize step is skipped with a warning.
#[cfg(not(test))]
pub fn apply_output_config(
  outputs: &[OutputConfig],
  terminal: &TerminalConfig,
) {
  // Explicit terminal size has highest priority.
  if let (Some(cols), Some(rows)) = (terminal.cols, terminal.rows) {
    tracing::info!(
      "Applying explicit terminal size: {}x{} cols/rows",
      cols,
      rows
    );
    apply_winsize(rows, cols, 0, 0);
    return;
  }

  // Find the output to derive sizing from.
  let primary = outputs
    .iter()
    .find(|o| o.enabled && o.primary)
    .or_else(|| outputs.iter().find(|o| o.enabled));

  let Some(primary) = primary else {
    tracing::warn!(
      "No enabled output found in [[outputs]] config; skipping terminal resize"
    );
    return;
  };

  tracing::info!(
    "Deriving terminal size from primary output: {}",
    primary.connector
  );

  let all_outputs = enumerate_outputs();
  let drm_output = all_outputs
    .iter()
    .find(|o| o.connector == primary.connector);

  let Some(drm_output) = drm_output else {
    tracing::warn!(
      "Output '{}' not found in /sys/class/drm/; skipping terminal resize",
      primary.connector
    );
    return;
  };

  let Some((native_w, native_h)) = drm_output.native_resolution else {
    tracing::warn!(
      "No native resolution available for '{}'; skipping terminal resize",
      primary.connector
    );
    return;
  };

  resize_terminal_to_resolution(native_w, native_h, &primary.connector);
}

/// Resize the terminal so it fills `native_w x native_h` pixels.
///
/// Reads the current terminal's pixel dimensions via `TIOCGWINSZ` to derive
/// the font cell size, then computes the new character-cell dimensions and
/// applies them via `TIOCSWINSZ`.
#[cfg(not(test))]
fn resize_terminal_to_resolution(
  native_w: u16,
  native_h: u16,
  connector: &str,
) {
  use std::os::unix::io::AsRawFd;

  let fd = std::io::stdout().as_raw_fd();

  let Some((cur_rows, cur_cols, cur_xpixel, cur_ypixel)) = get_winsize(fd)
  else {
    tracing::warn!(
      "Cannot read current terminal dimensions; skipping resize for '{}'",
      connector
    );
    return;
  };

  if cur_cols == 0 || cur_rows == 0 || cur_xpixel == 0 || cur_ypixel == 0 {
    tracing::warn!(
      "Terminal reports zero dimensions ({} cols x {} rows, {}x{} px); \
       skipping resize for '{}'",
      cur_cols,
      cur_rows,
      cur_xpixel,
      cur_ypixel,
      connector
    );
    return;
  }

  let cell_w = cur_xpixel / cur_cols;
  let cell_h = cur_ypixel / cur_rows;

  if cell_w == 0 || cell_h == 0 {
    tracing::warn!(
      "Font cell size is zero ({}x{} px); skipping resize for '{}'",
      cell_w,
      cell_h,
      connector
    );
    return;
  }

  let new_cols = native_w / cell_w;
  let new_rows = native_h / cell_h;

  if new_cols == 0 || new_rows == 0 {
    tracing::warn!(
      "Computed terminal dimensions are zero ({} cols x {} rows); skipping \
       resize for '{}'",
      new_cols,
      new_rows,
      connector
    );
    return;
  }

  tracing::info!(
    "Resizing terminal: {} cols x {} rows (font {}x{} px, output '{}' {}x{} \
     px)",
    new_cols,
    new_rows,
    cell_w,
    cell_h,
    connector,
    native_w,
    native_h
  );

  apply_winsize(new_rows, new_cols, native_w, native_h);
}

/// Write `rows x cols` (and optional pixel dimensions) to the terminal via
/// `TIOCSWINSZ`.
#[cfg(not(test))]
fn apply_winsize(rows: u16, cols: u16, xpixel: u16, ypixel: u16) {
  use std::os::unix::io::AsRawFd;

  use ffi::{TIOCSWINSZ, WinSize, ioctl};

  let fd = std::io::stdout().as_raw_fd();
  // SAFETY: `ws` is a valid, initialized `repr(C)` buffer for the duration of
  // this call, and `fd` is borrowed from the live stdout handle.
  unsafe {
    let mut ws = WinSize {
      ws_row:    rows,
      ws_col:    cols,
      ws_xpixel: xpixel,
      ws_ypixel: ypixel,
    };
    let ret = ioctl(fd, TIOCSWINSZ, &raw mut ws);
    if ret != 0 {
      tracing::warn!("TIOCSWINSZ failed: {}", std::io::Error::last_os_error());
    }
  }
}

/// Read the current terminal window size via `TIOCGWINSZ`.
///
/// Returns `(rows, cols, xpixel, ypixel)` or `None` if the ioctl fails.
#[cfg(not(test))]
fn get_winsize(fd: i32) -> Option<(u16, u16, u16, u16)> {
  use ffi::{TIOCGWINSZ, WinSize, ioctl};

  // SAFETY: `ws` is a valid, initialized `repr(C)` output buffer for the
  // duration of this call; the caller supplies an OS file descriptor.
  unsafe {
    let mut ws = WinSize {
      ws_row:    0,
      ws_col:    0,
      ws_xpixel: 0,
      ws_ypixel: 0,
    };
    let ret = ioctl(fd, TIOCGWINSZ, &raw mut ws);
    if ret == 0 {
      Some((ws.ws_row, ws.ws_col, ws.ws_xpixel, ws.ws_ypixel))
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs;

  use tempfile::TempDir;

  use super::*;

  #[test]
  fn test_parse_resolution_standard() {
    assert_eq!(parse_resolution("1920x1080"), Some((1920, 1080)));
    assert_eq!(parse_resolution("2560x1440"), Some((2560, 1440)));
    assert_eq!(parse_resolution("3840x2160"), Some((3840, 2160)));
    assert_eq!(parse_resolution("800x600"), Some((800, 600)));
  }

  #[test]
  fn test_parse_resolution_with_whitespace() {
    assert_eq!(parse_resolution("  1920x1080  "), Some((1920, 1080)));
    assert_eq!(parse_resolution("1920x1080\n"), Some((1920, 1080)));
  }

  #[test]
  fn test_parse_resolution_invalid() {
    assert_eq!(parse_resolution(""), None);
    assert_eq!(parse_resolution("1920"), None);
    assert_eq!(parse_resolution("1920x"), None);
    assert_eq!(parse_resolution("x1080"), None);
    assert_eq!(parse_resolution("AxB"), None);
    assert_eq!(parse_resolution("1920\u{00D7}1080"), None); // Unicode multiplication sign U+00D7, not ASCII x
  }

  #[test]
  fn test_parse_resolution_u16_boundary() {
    // Maximum valid u16 value in each position
    assert_eq!(parse_resolution("65535x65535"), Some((65535, 65535)));
    assert_eq!(parse_resolution("65535x1080"), Some((65535, 1080)));
    assert_eq!(parse_resolution("1920x65535"), Some((1920, 65535)));
  }

  #[test]
  fn test_parse_resolution_overflow() {
    // u16::MAX + 1 = 65536 - must not parse
    assert_eq!(parse_resolution("65536x1080"), None);
    assert_eq!(parse_resolution("1920x65536"), None);
    assert_eq!(parse_resolution("65536x65536"), None);
    // Much larger values
    assert_eq!(parse_resolution("999999x1080"), None);
    assert_eq!(parse_resolution("1920x999999"), None);
  }

  #[test]
  fn test_parse_resolution_zero_values() {
    // Zero is a valid u16 parse; validation is the caller's concern
    assert_eq!(parse_resolution("0x0"), Some((0, 0)));
    assert_eq!(parse_resolution("0x1080"), Some((0, 1080)));
    assert_eq!(parse_resolution("1920x0"), Some((1920, 0)));
  }

  #[test]
  fn test_parse_resolution_negative() {
    // u16 cannot represent negatives
    assert_eq!(parse_resolution("-1920x1080"), None);
    assert_eq!(parse_resolution("1920x-1080"), None);
    assert_eq!(parse_resolution("-1x-1"), None);
  }

  #[test]
  fn test_parse_resolution_multiple_x() {
    // split_once('x') takes only the first 'x'; the remainder can't parse
    assert_eq!(parse_resolution("1920x1080x720"), None);
    assert_eq!(parse_resolution("1920x1080x"), None);
  }

  #[test]
  fn test_parse_resolution_whitespace_variants() {
    assert_eq!(parse_resolution("   "), None);
    assert_eq!(parse_resolution("\t"), None);
    assert_eq!(parse_resolution("\t1920x1080\t"), Some((1920, 1080)));
    assert_eq!(parse_resolution("1920x1080\r\n"), Some((1920, 1080)));
  }

  fn make_connector(
    root: &TempDir,
    dir_name: &str,
    status: &str,
    modes: Option<&str>,
  ) {
    let connector_dir = root.path().join(dir_name);
    fs::create_dir_all(&connector_dir).unwrap();
    fs::write(connector_dir.join("status"), status).unwrap();
    if let Some(m) = modes {
      fs::write(connector_dir.join("modes"), m).unwrap();
    }
  }

  #[test]
  fn test_enumerate_empty_directory() {
    let dir = tempfile::tempdir().unwrap();
    let outputs = enumerate_outputs_from(dir.path());
    assert!(outputs.is_empty());
  }

  #[test]
  fn test_enumerate_nonexistent_directory() {
    let outputs =
      enumerate_outputs_from(std::path::Path::new("/nonexistent/drm/path"));
    assert!(outputs.is_empty());
  }

  #[test]
  fn test_enumerate_connected_with_resolution() {
    let dir = tempfile::tempdir().unwrap();
    make_connector(
      &dir,
      "card0-DP-1",
      "connected\n",
      Some("1920x1080\n1280x720\n"),
    );

    let outputs = enumerate_outputs_from(dir.path());
    assert_eq!(outputs.len(), 1);

    let o = &outputs[0];
    assert_eq!(o.connector, "DP-1");
    assert_eq!(o.card, "card0");
    assert!(o.connected);
    assert_eq!(o.native_resolution, Some((1920, 1080)));
  }

  #[test]
  fn test_enumerate_disconnected_no_modes() {
    let dir = tempfile::tempdir().unwrap();
    make_connector(&dir, "card1-HDMI-A-1", "disconnected\n", None);

    let outputs = enumerate_outputs_from(dir.path());
    assert_eq!(outputs.len(), 1);

    let o = &outputs[0];
    assert_eq!(o.connector, "HDMI-A-1");
    assert_eq!(o.card, "card1");
    assert!(!o.connected);
    assert_eq!(o.native_resolution, None);
  }

  #[test]
  fn test_enumerate_skips_entries_without_status_file() {
    let dir = tempfile::tempdir().unwrap();
    // Directory named like a connector but missing `status`
    fs::create_dir(dir.path().join("card0-DP-2")).unwrap();

    let outputs = enumerate_outputs_from(dir.path());
    assert!(outputs.is_empty());
  }

  #[test]
  fn test_enumerate_skips_non_card_entries() {
    let dir = tempfile::tempdir().unwrap();
    // Entry doesn't start with "card"
    let bad_dir = dir.path().join("version-DP-1");
    fs::create_dir(&bad_dir).unwrap();
    fs::write(bad_dir.join("status"), "connected\n").unwrap();

    let outputs = enumerate_outputs_from(dir.path());
    assert!(outputs.is_empty());
  }

  #[test]
  fn test_enumerate_sorted_by_card_then_connector() {
    let dir = tempfile::tempdir().unwrap();
    make_connector(&dir, "card1-DP-2", "connected\n", Some("1920x1080\n"));
    make_connector(&dir, "card0-HDMI-A-1", "connected\n", Some("1280x720\n"));
    make_connector(&dir, "card1-DP-1", "connected\n", Some("2560x1440\n"));
    make_connector(&dir, "card0-DP-1", "disconnected\n", None);

    let outputs = enumerate_outputs_from(dir.path());
    assert_eq!(outputs.len(), 4);
    assert_eq!(outputs[0].card, "card0");
    assert_eq!(outputs[0].connector, "DP-1");
    assert_eq!(outputs[1].card, "card0");
    assert_eq!(outputs[1].connector, "HDMI-A-1");
    assert_eq!(outputs[2].card, "card1");
    assert_eq!(outputs[2].connector, "DP-1");
    assert_eq!(outputs[3].card, "card1");
    assert_eq!(outputs[3].connector, "DP-2");
  }

  #[test]
  fn test_enumerate_file_not_directory() {
    let dir = tempfile::tempdir().unwrap();
    // A plain file named like a connector must be skipped; the `status` path
    // inside it cannot exist.
    fs::write(dir.path().join("card0-DP-1"), "not a directory").unwrap();
    let outputs = enumerate_outputs_from(dir.path());
    assert!(outputs.is_empty());
  }

  #[test]
  fn test_enumerate_empty_modes_file() {
    let dir = tempfile::tempdir().unwrap();
    make_connector(&dir, "card0-DP-1", "connected\n", Some(""));
    let outputs = enumerate_outputs_from(dir.path());
    assert_eq!(outputs.len(), 1);
    // Empty modes file produces no native resolution
    assert_eq!(outputs[0].native_resolution, None);
  }

  #[test]
  fn test_enumerate_garbage_modes_line() {
    let dir = tempfile::tempdir().unwrap();
    make_connector(
      &dir,
      "card0-DP-1",
      "connected\n",
      Some("not_a_resolution\n1920x1080\n"),
    );
    let outputs = enumerate_outputs_from(dir.path());
    assert_eq!(outputs.len(), 1);
    // First line is unparseable; falls back to None
    assert_eq!(outputs[0].native_resolution, None);
  }

  #[test]
  fn test_enumerate_modes_whitespace_only_first_line() {
    let dir = tempfile::tempdir().unwrap();
    make_connector(&dir, "card0-DP-1", "connected\n", Some("   \n1920x1080\n"));
    let outputs = enumerate_outputs_from(dir.path());
    assert_eq!(outputs[0].native_resolution, None);
  }

  #[test]
  fn test_enumerate_status_case_sensitive() {
    let dir = tempfile::tempdir().unwrap();
    // "Connected" (capital C) must not be treated as connected
    make_connector(&dir, "card0-DP-1", "Connected\n", None);
    let outputs = enumerate_outputs_from(dir.path());
    assert_eq!(outputs.len(), 1);
    assert!(!outputs[0].connected);
  }

  #[test]
  fn test_enumerate_status_crlf() {
    let dir = tempfile::tempdir().unwrap();
    // CRLF line endings: trim() strips both \r and \n
    make_connector(&dir, "card0-DP-1", "connected\r\n", None);
    let outputs = enumerate_outputs_from(dir.path());
    assert_eq!(outputs.len(), 1);
    assert!(outputs[0].connected);
  }

  #[test]
  fn test_enumerate_connector_name_with_multiple_dashes() {
    let dir = tempfile::tempdir().unwrap();
    // Real DRM names like "HDMI-A-1" have multiple dashes; everything after
    // the first dash is the connector name.
    make_connector(
      &dir,
      "card0-DisplayPort-2",
      "connected\n",
      Some("2560x1440\n"),
    );
    let outputs = enumerate_outputs_from(dir.path());
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].connector, "DisplayPort-2");
    assert_eq!(outputs[0].card, "card0");
  }

  #[test]
  fn test_enumerate_entry_with_no_dash_is_skipped() {
    let dir = tempfile::tempdir().unwrap();
    // A plain "card0" entry has no dash so find('-') returns None and is
    // skipped
    let no_dash = dir.path().join("card0");
    fs::create_dir(&no_dash).unwrap();
    fs::write(no_dash.join("status"), "connected\n").unwrap();
    let outputs = enumerate_outputs_from(dir.path());
    assert!(outputs.is_empty());
  }

  #[test]
  fn test_enumerate_modes_first_line_is_native() {
    let dir = tempfile::tempdir().unwrap();
    // The first listed mode is treated as the preferred/native resolution.
    make_connector(
      &dir,
      "card0-DP-1",
      "connected\n",
      Some("2560x1440\n1920x1080\n1280x720\n"),
    );

    let outputs = enumerate_outputs_from(dir.path());
    assert_eq!(outputs[0].native_resolution, Some((2560, 1440)));
  }
}
