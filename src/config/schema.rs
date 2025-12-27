use serde::{Deserialize, Serialize};

/// Root configuration structure
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
  #[serde(default)]
  pub general: GeneralConfig,

  #[serde(default)]
  pub session: SessionConfig,

  #[serde(default)]
  pub display: DisplayConfig,

  #[serde(default)]
  pub remember: RememberConfig,

  #[serde(default)]
  pub user_menu: UserMenuConfig,

  #[serde(default)]
  pub secret: SecretConfig,

  #[serde(default)]
  pub layout: LayoutConfig,

  #[serde(default)]
  pub power: PowerConfig,

  #[serde(default)]
  pub keybindings: KeybindingsConfig,

  #[serde(default)]
  pub theme: ThemeConfig,
}

/// General configuration options
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeneralConfig {
  /// Enable debug logging
  #[serde(default)]
  pub debug: bool,

  /// Log file path
  #[serde(default = "default_log_file")]
  pub log_file: String,
}

impl Default for GeneralConfig {
  fn default() -> Self {
    Self {
      debug:    false,
      log_file: default_log_file(),
    }
  }
}

/// Session management configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionConfig {
  /// Override session with a specific command
  #[serde(default)]
  pub command: Option<String>,

  /// Directories containing Wayland session files
  #[serde(default = "default_sessions_dirs")]
  pub sessions_dirs: Vec<String>,

  /// Directories containing X11 session files
  #[serde(default = "default_xsessions_dirs")]
  pub xsessions_dirs: Vec<String>,

  /// Wrapper command for non-X11 sessions
  #[serde(default)]
  pub session_wrapper: Option<String>,

  /// Wrapper command for X11 sessions
  #[serde(default = "default_xsession_wrapper")]
  pub xsession_wrapper: Option<String>,

  /// Environment variables for default session
  #[serde(default)]
  pub environments: Vec<String>,
}

impl Default for SessionConfig {
  fn default() -> Self {
    Self {
      command:          None,
      sessions_dirs:    default_sessions_dirs(),
      xsessions_dirs:   default_xsessions_dirs(),
      session_wrapper:  None,
      xsession_wrapper: default_xsession_wrapper(),
      environments:     Vec::new(),
    }
  }
}

/// Display and visual configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DisplayConfig {
  /// Show current time
  #[serde(default)]
  pub show_time: bool,

  /// Custom time format (strftime)
  #[serde(default)]
  pub time_format: Option<String>,

  /// Custom greeting message
  #[serde(default)]
  pub greeting: Option<String>,

  /// Show /etc/issue file
  #[serde(default)]
  pub issue: bool,

  /// Greeting text alignment
  #[serde(default)]
  pub align_greeting: AlignGreeting,
}

/// Remember/cache configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RememberConfig {
  /// Remember last logged-in username
  #[serde(default)]
  pub username: bool,

  /// Remember last selected session (global)
  #[serde(default)]
  pub session: bool,

  /// Remember last selected session per user
  #[serde(default)]
  pub user_session: bool,
}

/// User menu configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserMenuConfig {
  /// Enable user selection menu
  #[serde(default)]
  pub enabled: bool,

  /// Minimum UID to display in user menu
  #[serde(default = "default_min_uid")]
  pub min_uid: u32,

  /// Maximum UID to display in user menu
  #[serde(default = "default_max_uid")]
  pub max_uid: u32,
}

impl Default for UserMenuConfig {
  fn default() -> Self {
    Self {
      enabled: false,
      min_uid: default_min_uid(),
      max_uid: default_max_uid(),
    }
  }
}

/// Secret display configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecretConfig {
  /// How to display secrets
  #[serde(default)]
  pub mode: SecretMode,

  /// Characters to use when mode is Characters
  #[serde(default = "default_secret_characters")]
  pub characters: String,
}

impl Default for SecretConfig {
  fn default() -> Self {
    Self {
      mode:       SecretMode::Hidden,
      characters: default_secret_characters(),
    }
  }
}

/// Layout and sizing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LayoutConfig {
  /// Width of the main prompt container
  #[serde(default = "default_width")]
  pub width: u16,

  /// Padding around the terminal window
  #[serde(default)]
  pub window_padding: Option<u16>,

  /// Padding inside the main container
  #[serde(default)]
  pub container_padding: Option<u16>,

  /// Padding between prompt rows
  #[serde(default)]
  pub prompt_padding: Option<u16>,

  /// Widget positioning options
  #[serde(default)]
  pub widgets: WidgetConfig,
}

impl Default for LayoutConfig {
  fn default() -> Self {
    Self {
      width:             default_width(),
      window_padding:    None,
      container_padding: None,
      prompt_padding:    None,
      widgets:           WidgetConfig::default(),
    }
  }
}

/// Widget positioning configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct WidgetConfig {
  /// Position of time widget
  #[serde(default)]
  pub time_position: WidgetPosition,

  /// Position of status bar widget
  #[serde(default)]
  pub status_position: WidgetPosition,
}

/// Power management configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PowerConfig {
  /// Custom shutdown command
  #[serde(default)]
  pub shutdown: Option<String>,

  /// Custom reboot command
  #[serde(default)]
  pub reboot: Option<String>,

  /// Use setsid to detach power commands
  #[serde(default = "default_use_setsid")]
  pub use_setsid: bool,
}

impl Default for PowerConfig {
  fn default() -> Self {
    Self {
      shutdown:   None,
      reboot:     None,
      use_setsid: default_use_setsid(),
    }
  }
}

/// Keybindings configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeybindingsConfig {
  /// F-key for command menu (1-12)
  #[serde(default = "default_kb_command")]
  pub command: u8,

  /// F-key for sessions menu (1-12)
  #[serde(default = "default_kb_sessions")]
  pub sessions: u8,

  /// F-key for power menu (1-12)
  #[serde(default = "default_kb_power")]
  pub power: u8,
}

impl Default for KeybindingsConfig {
  fn default() -> Self {
    Self {
      command:  default_kb_command(),
      sessions: default_kb_sessions(),
      power:    default_kb_power(),
    }
  }
}

/// Theme/color configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ThemeConfig {
  /// Border color
  #[serde(default)]
  pub border:    Option<String>,
  /// Base text color
  #[serde(default)]
  pub text:      Option<String>,
  /// Time display color
  #[serde(default)]
  pub time:      Option<String>,
  /// Container background color
  #[serde(default)]
  pub container: Option<String>,
  /// Container title color
  #[serde(default)]
  pub title:     Option<String>,
  /// Greeting text color
  #[serde(default)]
  pub greet:     Option<String>,
  /// Prompt text color
  #[serde(default)]
  pub prompt:    Option<String>,
  /// User input color
  #[serde(default)]
  pub input:     Option<String>,
  /// Action text color
  #[serde(default)]
  pub action:    Option<String>,
  /// Action button color
  #[serde(default)]
  pub button:    Option<String>,
}

/// Greeting alignment options
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AlignGreeting {
  Left,
  #[default]
  Center,
  Right,
}

/// Secret display modes
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SecretMode {
  #[default]
  Hidden,
  Characters,
}

/// Widget position options
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WidgetPosition {
  #[default]
  Default,
  Top,
  Bottom,
  Hidden,
}

// Default value functions
fn default_log_file() -> String {
  "/tmp/tuigreet.log".to_string()
}

fn default_sessions_dirs() -> Vec<String> {
  vec!["/usr/share/wayland-sessions".to_string()]
}

fn default_xsessions_dirs() -> Vec<String> {
  vec!["/usr/share/xsessions".to_string()]
}

fn default_xsession_wrapper() -> Option<String> {
  Some("startx".to_string())
}

fn default_min_uid() -> u32 {
  1000
}

fn default_max_uid() -> u32 {
  60000
}

fn default_secret_characters() -> String {
  "*".to_string()
}

fn default_width() -> u16 {
  80
}

fn default_use_setsid() -> bool {
  true
}

fn default_kb_command() -> u8 {
  2
}

fn default_kb_sessions() -> u8 {
  3
}

fn default_kb_power() -> u8 {
  12
}
