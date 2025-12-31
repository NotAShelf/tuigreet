use std::{
  path::{Path, PathBuf},
  sync::Arc,
  time::Duration,
};

use notify::{
  Config as NotifyConfig,
  Event,
  EventKind,
  RecommendedWatcher,
  RecursiveMode,
  Watcher,
};
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, error, info, warn};
use tuigreet::config::{Config, parser::load_config};

use crate::{Greeter, event::Event as GreeterEvent};

/// File watcher for hot-reloading configuration changes.
///
/// Keeps the watcher alive for the lifetime of the struct.
/// Dropping this struct will stop file watching.
#[allow(dead_code)]
pub struct ConfigWatcher {
  #[allow(dead_code)]
  watcher: RecommendedWatcher,
}

#[allow(dead_code)]
impl ConfigWatcher {
  /// Create a new config file watcher.
  ///
  /// # Arguments
  /// * `config_path` - Optional explicit config path, otherwise uses XDG/system
  ///   paths
  /// * `greeter` - Shared greeter state to update on config changes
  /// * `event_sender` - Channel to send UI refresh events
  ///
  /// # Returns
  /// ConfigWatcher that monitors the config file for changes
  ///
  /// # Errors
  /// Returns error if file watcher cannot be initialized
  pub fn new(
    config_path: Option<PathBuf>,
    greeter: Arc<RwLock<Greeter>>,
    event_sender: mpsc::Sender<GreeterEvent>,
  ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
    let (tx, mut rx) = mpsc::channel::<Result<Event, notify::Error>>(100);

    // Create the file watcher
    let mut watcher = RecommendedWatcher::new(
      move |res| {
        if let Err(e) = tx.blocking_send(res) {
          error!("Failed to send file watch event: {}", e);
        }
      },
      NotifyConfig::default(),
    )?;

    // Determine which config file to watch
    let watch_path = if let Some(ref path) = config_path {
      path.clone()
    } else {
      // Use XDG config path
      if let Some(config_dir) = dirs::config_dir() {
        let user_config = config_dir.join("tuigreet").join("config.toml");
        if user_config.exists() {
          user_config
        } else {
          // Fall back to system config
          PathBuf::from("/etc/tuigreet/config.toml")
        }
      } else {
        PathBuf::from("/etc/tuigreet/config.toml")
      }
    };

    // Only watch if the config file exists
    if watch_path.exists() {
      info!("Starting config file watcher for: {}", watch_path.display());

      // Watch the parent directory to catch file recreations
      if let Some(parent) = watch_path.parent() {
        watcher.watch(parent, RecursiveMode::NonRecursive)?;
      }
    } else {
      info!(
        "Config file does not exist, hot reloading disabled: {}",
        watch_path.display()
      );
    }

    // Spawn background task to handle file events
    tokio::spawn(async move {
      while let Some(result) = rx.recv().await {
        match result {
          Ok(event) => {
            // Check if this is a modification to our config file
            if Self::is_config_event(&event, &watch_path) {
              debug!("Config file change detected: {:?}", event);

              // Add a small delay to avoid partial writes
              tokio::time::sleep(Duration::from_millis(100)).await;

              match Self::reload_config(&watch_path).await {
                Ok(new_config) => {
                  if let Err(e) =
                    Self::apply_config_to_greeter(&greeter, new_config).await
                  {
                    error!("Failed to apply reloaded config: {}", e);
                  } else {
                    info!("Config successfully reloaded");
                    // Optionally trigger a UI refresh
                    if let Err(e) =
                      event_sender.send(GreeterEvent::Refresh).await
                    {
                      warn!("Failed to send refresh event: {}", e);
                    }
                  }
                },
                Err(e) => {
                  error!("Failed to reload config: {}", e);
                },
              }
            }
          },
          Err(e) => {
            error!("File watcher error: {}", e);
          },
        }
      }
    });

    Ok(ConfigWatcher { watcher })
  }

  fn is_config_event(event: &Event, config_path: &Path) -> bool {
    match event.kind {
      EventKind::Modify(_) | EventKind::Create(_) => {
        event.paths.iter().any(|path| path == config_path)
      },
      _ => false,
    }
  }

  async fn reload_config(
    config_path: &Path,
  ) -> Result<Config, Box<dyn std::error::Error + Send + Sync>> {
    debug!("Reloading config from: {}", config_path.display());

    let mut config = load_config(Some(config_path))?;

    tuigreet::config::env::apply_env_vars(&mut config);

    match config.validate(false) {
      Ok(warnings) => {
        for warning in warnings {
          warn!("Config warning after reload: {}", warning);
        }
      },
      Err(e) => {
        return Err(
          format!("Config validation failed after reload: {}", e).into(),
        );
      },
    }

    Ok(config)
  }

  async fn apply_config_to_greeter(
    greeter: &Arc<RwLock<Greeter>>,
    config: Config,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut greeter_guard = greeter.write().await;

    // Store the old config for rollback if needed
    let _old_config = greeter_guard.loaded_config.clone();

    // Apply the new configuration
    greeter_guard.apply_config(&config);

    // Apply theme configuration
    greeter_guard.apply_theme_config(&config.theme, None);

    // Store the new config
    greeter_guard.loaded_config = Some(config);

    // XXX: We don't rollback on theme application failure since it's
    // non-critical The config has been successfully applied at this point
    info!("Config hot reload completed successfully");
    Ok(())
  }
}
