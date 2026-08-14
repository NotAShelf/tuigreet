use clap::{ArgAction, Args, Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use tuigreet_types::{
  DEFAULT_LOG_FILE,
  DEFAULT_SYSTEM_TOML_PATH,
  DEFAULT_USER_TOML_PATH,
  DEFAULT_XSESSION_WRAPPER,
};

/// Root configuration structure
#[derive(Parser, Debug, Clone, Deserialize, Serialize, PartialEq)]
#[clap(
  author,
  version,
  about = "Stylish graphical console greeter for greetd",
  name = "tuigreet"
)]
pub struct Config {
  #[command(flatten)]
  pub general: GeneralConfig,

  #[command(flatten)]
  pub session: SessionConfig,

  #[command(flatten)]
  pub display: DisplayConfig,

  #[command(flatten)]
  pub remember: RememberConfig,

  #[command(flatten)]
  pub user_menu: UserMenuConfig,

  #[command(flatten)]
  pub secret: SecretConfig,

  #[command(flatten)]
  pub layout: LayoutConfig,

  #[command(flatten)]
  pub power: PowerConfig,

  #[command(flatten)]
  pub keybindings: KeybindingsConfig,

  #[command(flatten)]
  pub theme: ThemeConfig,

  #[command(flatten)]
  pub background: BackgroundConfig,

  /// Per-output (display/monitor) configuration.
  /// Use `[[outputs]]` array of tables in TOML.
  #[serde(default)]
  #[clap(skip)]
  pub outputs: Vec<OutputConfig>,

  /// Explicit terminal size override. When both `cols` and `rows` are set
  /// they take precedence over output-derived sizing.
  #[serde(default)]
  #[clap(skip)]
  pub terminal: TerminalConfig,
}

impl Config {
  // clap defaults config
  #[must_use]
  pub fn defaults() -> Self {
    Self::parse_from(std::iter::empty::<&str>())
  }
}

/// Configuration for a single DRM output (monitor/display).
///
/// Example:
/// ```toml
/// [[outputs]]
/// connector = "DP-1"
/// primary = true
///
/// [[outputs]]
/// connector = "HDMI-A-1"
/// enabled = false
/// ```
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OutputConfig {
  /// DRM connector name as it appears in `/sys/class/drm/` (e.g. `"DP-1"`,
  /// `"HDMI-A-1"`).
  #[serde(default)]
  pub connector: String,

  /// Whether tuigreet should use this output. Defaults to `true`.
  #[serde(default = "default_true")]
  pub enabled: bool,

  /// If `true`, size the terminal to match this output's native resolution.
  /// At most one output should be marked primary. If none is marked primary
  /// the first enabled output is used for sizing.
  #[serde(default)]
  pub primary: bool,
}

/// Explicit terminal character-cell size override.
///
/// When both `cols` and `rows` are provided they take highest priority over
/// output-derived sizing. Providing only one of the two fields is an error.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct TerminalConfig {
  /// Number of character columns.
  #[serde(default)]
  pub cols: Option<u16>,
  /// Number of character rows.
  #[serde(default)]
  pub rows: Option<u16>,
}

/// General configuration options
#[derive(Args, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GeneralConfig {
  /// path to system configuration toml file
  #[arg(long = "system-config", default_value_t = DEFAULT_SYSTEM_TOML_PATH.to_string())]
  pub system_config_path: String,

  /// path to user configuration toml file
  #[arg(long = "config", default_value_t = DEFAULT_USER_TOML_PATH.to_string())]
  pub user_config_path: String,

  /// disable loading toml configuration files
  #[arg(long, action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = false)]
  pub no_config: bool,

  /// Enable debug logging
  #[arg(short, long, action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = false)]
  pub debug: bool,

  /// Log file path
  #[arg(short, long, default_value_t = DEFAULT_LOG_FILE.to_string())]
  pub log_file: String,

  /// visual mock-up mode: skip the greetd socket and fake the auth flow
  /// locally
  #[arg(short, long, action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = false)]
  pub mock: bool,

  /// Enable numlock on startup
  #[arg(short, long, action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = false)]
  pub numlock: bool,
}

/// Session management configuration
#[derive(Args, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SessionConfig {
  /// Override session with a specific command
  #[arg(short = 'c', long = "cmd")]
  pub command: Option<String>,

  /// Directories containing Wayland session files
  #[arg(short, long = "sessions")]
  pub sessions_dirs: Vec<String>,

  /// Directories containing X11 session files
  #[arg(short, long = "xsessions", default_values_t = vec!["/usr/share/xsessions".to_string()])]
  pub xsessions_dirs: Vec<String>,

  /// Wrapper command for non-X11 sessions
  #[arg(long)]
  pub session_wrapper: Option<String>,

  /// Wrapper command for X11 sessions (set to empty string "" for no wrapper)
  #[arg(long, default_value_t = DEFAULT_XSESSION_WRAPPER.to_string())]
  pub xsession_wrapper: String,

  /// Environment variables for default session
  #[arg(long = "env")]
  pub environments: Vec<String>,
}

/// Display and visual configuration
#[derive(
  Args, Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq,
)]
pub struct DisplayConfig {
  /// Show current time
  #[arg(short = 't', long = "time", action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = false)]
  pub show_time: bool,

  /// Custom time format (strftime)
  #[arg(long)]
  pub time_format: Option<String>,

  /// Custom greeting message
  #[arg(short, long)]
  pub greeting: Option<String>,

  /// Show login form title
  #[arg(long = "title", action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = true)]
  pub show_title: bool,

  /// Custom login form title
  #[arg(long)]
  pub custom_title: Option<String>,

  /// Show /etc/issue file
  #[arg(short, long, action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = false)]
  #[arg(short, long, default_value_t = false)]
  pub issue: bool,

  /// Show battery percentage
  #[arg(short, long, action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = false)]
  pub battery: bool,

  /// Greeting text alignment
  #[arg(long = "greet-align", value_enum, default_value_t = AlignGreeting::default())]
  pub align_greeting: AlignGreeting,
}

/// Remember/cache configuration
#[derive(Args, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RememberConfig {
  /// Default user to pre-fill
  #[arg(short = 'u', long = "user")]
  pub default_user: Option<String>,

  /// Remember last logged-in username
  #[arg(short = 'r', long = "remember", action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = false)]
  pub username: bool,

  /// Remember last selected session (global)
  #[arg(long = "remember-session", action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = false)]
  pub session: bool,

  /// Remember last selected session per user
  #[arg(long = "remember-user-session", action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = false)]
  pub user_session: bool,
}

/// User menu configuration
#[derive(Args, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct UserMenuConfig {
  /// Enable user selection menu
  #[arg(long = "remember-user-session", action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = false)]
  #[arg(long = "user-menu", default_value_t = false)]
  pub enabled: bool,

  /// Minimum UID to display in user menu
  #[arg(long = "user-menu-min-uid", default_value_t = 1000)]
  pub min_uid: u32,

  /// Maximum UID to display in user menu
  #[arg(long = "user-menu-max-uid", default_value_t = 60000)]
  pub max_uid: u32,
}

/// Secret display configuration
#[derive(Args, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SecretConfig {
  /// How to display secrets
  #[arg(long = "secret-mode", value_enum, default_value_t = SecretMode::default())]
  pub mode: SecretMode,

  /// Characters to use when mode is Characters
  #[arg(long = "secret-characters", default_value_t = "*".to_string())]
  pub characters: String,
}

/// Layout and sizing configuration
#[derive(Args, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LayoutConfig {
  /// Width of the main prompt container
  #[arg(short, long, default_value_t = 80)]
  pub width: u16,

  /// Padding around the terminal window
  #[arg(long, default_value_t = 0)]
  pub window_padding: u16,

  /// Padding inside the main container
  #[arg(long, default_value_t = 2)]
  pub container_padding: u16,

  /// Padding between prompt rows
  #[arg(long, default_value_t = 1)]
  pub prompt_padding: u16,

  /// Widget positioning options
  #[command(flatten)]
  pub widgets: WidgetConfig,
}

/// Battery widget placement
#[derive(
  Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq, ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum BatteryPosition {
  #[default]
  Left,
  Right,
}

/// Widget positioning configuration
#[derive(Args, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct WidgetConfig {
  /// Position of time widget
  #[arg(long, value_enum, default_value_t = WidgetPosition::default())]
  pub time_position: WidgetPosition,

  /// Position of status bar widget
  #[arg(long, value_enum, default_value_t = WidgetPosition::default())]
  pub status_position: WidgetPosition,

  /// Position of battery widget
  #[arg(long, value_enum, default_value_t = BatteryPosition::default())]
  pub battery_position: BatteryPosition,

  /// Status bar item visibility
  #[command(flatten)]
  pub status_bar: StatusBarConfig,
}

/// Status bar item visibility configuration
#[derive(Args, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct StatusBarConfig {
  /// Show the ESC/Reset button
  #[arg(long, action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = true)]
  pub show_reset: bool,

  /// Show the command button
  #[arg(long, action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = true)]
  pub show_command: bool,

  /// Show the session button
  #[arg(long, action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = true)]
  pub show_session: bool,

  /// Show the power button
  #[arg(long, action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = true)]
  pub show_power: bool,

  /// Show the background button
  #[arg(long, action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = true)]
  pub show_background: bool,

  /// Show the current session/command indicator
  #[arg(long, action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = true)]
  pub show_session_status: bool,

  /// Show the caps lock indicator
  #[arg(long, action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = true)]
  pub show_caps_lock: bool,
}

/// Power management configuration
#[derive(Args, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PowerConfig {
  /// Custom shutdown command
  #[arg(long = "power-shutdown")]
  pub shutdown: Option<String>,

  /// Custom reboot command
  #[arg(long = "power-reboot")]
  pub reboot: Option<String>,

  /// Custom suspend command
  #[arg(long = "power-suspend")]
  pub suspend: Option<String>,

  /// Custom hibernate command
  #[arg(long = "power-hibernate")]
  pub hibernate: Option<String>,

  /// Use setsid to detach power commands
  #[arg(long = "power-use-setsid", action = ArgAction::Set,
      num_args = 0..=1, require_equals = true,
      default_missing_value = "true",
      default_value_t = false)]
  pub use_setsid: bool,
}

/// Keybindings configuration
#[derive(Args, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct KeybindingsConfig {
  /// F-key for command menu (1-12)
  #[arg(long = "kb-command", id = "kb_command", default_value_t = 2)]
  pub command: u8,

  /// F-key for sessions menu (1-12)
  #[arg(long = "kb-sessions", default_value_t = 3)]
  pub sessions: u8,

  /// F-key for power menu (1-12)
  #[arg(long = "kb-power", default_value_t = 12)]
  pub power: u8,

  /// F-key for the on-the-fly background switcher menu (1-12)
  #[arg(long = "kb-background", default_value_t = 4)]
  pub background: u8,
}

/// Theme/color configuration
#[derive(Args, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ThemeConfig {
  /// Border color
  #[arg(long = "theme-border")]
  pub border:    Option<String>,
  /// Base text color
  #[arg(long = "theme-text")]
  pub text:      Option<String>,
  /// Time display color
  #[arg(long = "theme-time")]
  pub time:      Option<String>,
  /// Container background color
  #[arg(long = "theme-container")]
  pub container: Option<String>,
  /// Container title color
  #[arg(long = "theme-title")]
  pub title:     Option<String>,
  /// Greeting text color
  #[arg(long = "theme-greet")]
  pub greet:     Option<String>,
  /// Prompt text color
  #[arg(long = "theme-prompt")]
  pub prompt:    Option<String>,
  /// User input color
  #[arg(long = "theme-input")]
  pub input:     Option<String>,
  /// Action text color
  #[arg(long = "theme-action")]
  pub action:    Option<String>,
  /// Action button color
  #[arg(long = "theme-button")]
  pub button:    Option<String>,
}

/// Background animation configuration. Each animation kind owns its own
/// namespaced sub-section so new animations can be added without collisions.
/// Distinct from upstream's `[animations]` section, which configures the
/// foreground tachyonfx post-processor.
#[derive(Args, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct BackgroundConfig {
  /// Which animation to run. `None` or `"none"` disables animations.
  /// Currently supported: `"doom"`, `"matrix"`.
  #[arg(long = "background")]
  pub kind: Option<String>,

  /// Render frame rate when an animation is active. Defaults to 30 FPS when
  /// `kind` is set; the base UI tick (2 FPS) otherwise.
  #[arg(long = "background-fps", default_value_t = 30)]
  pub fps: u32,

  /// Parameters for the DOOM-style fire effect.
  #[command(flatten)]
  pub doom: DoomConfig,

  /// Parameters for the cmatrix-style digital rain effect.
  #[command(flatten)]
  pub matrix: MatrixConfig,
}

/// Parameters for the DOOM-style fire animation. Field names mirror Ly's
/// `doom_fire_*` config keys.
#[derive(Args, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DoomConfig {
  /// Decay control (1..=9). Higher = taller flames.
  #[arg(long = "doom-height", default_value_t = 6)]
  pub height: u8,

  /// Horizontal jitter range (0..=4).
  #[arg(long = "doom-spread", default_value_t = 2)]
  pub spread: u8,

  /// Color of the coolest flame tips. Accepts `#RRGGBB`, `0xRRGGBB`, or any
  /// color name accepted by ratatui (e.g. `red`, `magenta`).
  #[arg(long = "doom-top-color", default_value_t = "#9F2707".to_string())]
  pub top_color: String,

  /// Color of the mid-band flames.
  #[arg(long = "doom-middle-color", default_value_t = "#C78F17".to_string())]
  pub middle_color: String,

  /// Color of the hottest flames at the base.
  #[arg(long = "doom-bottom-color", default_value_t = "#FFFFFF".to_string())]
  pub bottom_color: String,
}

/// Parameters for the cmatrix-style digital rain animation.
#[derive(Args, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MatrixConfig {
  /// Color of the leading glyph in each falling stream. Accepts `#RRGGBB`,
  /// `0xRRGGBB`, or any color name accepted by ratatui.
  #[arg(long = "matrix-head-color", default_value_t = "#CCFFCC".to_string())]
  pub head_color: String,

  /// Color of the brightest part of the trail (just behind the head).
  #[arg(long = "matrix-bright-color", default_value_t = "#33FF66".to_string())]
  pub bright_color: String,

  /// Color of the dim tail before each glyph fades out.
  #[arg(long = "matrix-dim-color", default_value_t = "#006622".to_string())]
  pub dim_color: String,

  /// Inclusive minimum trail length in rows.
  #[arg(long = "matrix-min-length", default_value_t = 6)]
  pub min_length: u16,

  /// Inclusive maximum trail length in rows.
  #[arg(long = "matrix-max-length", default_value_t = 18)]
  pub max_length: u16,

  /// Inclusive minimum stream speed, in rows-per-frame. Lower = slower.
  #[arg(long = "matrix-min-speed", default_value_t = 0.30)]
  pub min_speed: f32,

  /// Inclusive maximum stream speed, in rows-per-frame.
  #[arg(long = "matrix-max-speed", default_value_t = 1.10)]
  pub max_speed: f32,

  /// Per-cell, per-frame probability of a glyph mutating (the faint trail
  /// shimmer). `0.0` disables.
  #[arg(long = "matrix-mutate-chance", default_value_t = 0.02)]
  pub mutate_chance: f32,
}

/// Greeting alignment options
#[derive(
  Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq, ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum AlignGreeting {
  Left,
  #[default]
  Center,
  Right,
}

/// Secret display modes
#[derive(
  Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq, ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum SecretMode {
  #[default]
  Hidden,
  Characters,
}

/// Widget position options
#[derive(
  Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq, ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum WidgetPosition {
  #[default]
  Default,
  Top,
  Bottom,
  Hidden,
}

fn default_true() -> bool {
  true
}
