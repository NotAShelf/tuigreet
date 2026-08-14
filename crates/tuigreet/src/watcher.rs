use std::{path::PathBuf, sync::Arc, time::Duration};

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
use tuigreet_config::{config::Config, loader::load_config};

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
  /// `ConfigWatcher` that monitors the config file for changes
  ///
  /// # Errors
  /// Returns error if file watcher cannot be initialized
  pub fn new(
    config_paths: Vec<PathBuf>,
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
    let mut watched_paths = Vec::new();

    // watch all provided config paths that exist
    for path in config_paths {
      let Some(parent) = path.parent() else {
        continue;
      };

      // watch the parent directory (notify works on dirs, not files)
      let watch_root = parent.ancestors().find(|p| p.exists());

      if let Some(root) = watch_root {
        watcher.watch(root, RecursiveMode::NonRecursive)?;
        watched_paths.push(path.clone());
        info!("Watching config: {}", path.display());
      }
    }

    // Only watch if the config file exists
    if watched_paths.is_empty() {
      info!("No config files exist, hot reloading disabled");
    }

    // Spawn background task to handle file events
    let watched_paths_clone = watched_paths.clone();
    tokio::spawn(async move {
      while let Some(result) = rx.recv().await {
        match result {
          Ok(event) => {
            // Check if this is a modification to any of our config files
            if Self::is_config_event(&event, &watched_paths_clone) {
              debug!("Config file change detected: {:?}", event);

              // Add a small delay to avoid partial writes
              tokio::time::sleep(Duration::from_millis(100)).await;

              match Self::reload_config() {
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

    Ok(Self { watcher })
  }

  fn is_config_event(event: &Event, config_paths: &[PathBuf]) -> bool {
    match event.kind {
      EventKind::Modify(_) | EventKind::Create(_) => {
        event.paths.iter().any(|path| config_paths.contains(path))
      },
      _ => false,
    }
  }

  fn reload_config() -> Result<Config, Box<dyn std::error::Error + Send + Sync>>
  {
    debug!("Reloading configuration");

    match load_config() {
      Ok((config, warnings)) => {
        for warning in &warnings {
          tracing::warn!("{}", warning);
        }
        Ok(config)
      },
      Err((config, (errors, warnings))) => {
        // Log all errors and warnings from the failed load
        for error in &errors {
          tracing::error!("{}", error);
        }
        for warning in &warnings {
          tracing::warn!("{}", warning);
        }

        // Decide: do you want to:
        // A) Still return the partial config?
        // B) Fail and return an error?

        if errors.is_empty() {
          // Only warnings, so it's effectively OK
          Ok(config)
        } else {
          // Return a meaningful error with collected errors
          let err_msg = errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", ");
          Err(err_msg.into())
        }
      },
    }
  }

  async fn apply_config_to_greeter(
    greeter: &Arc<RwLock<Greeter>>,
    config: Config,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut greeter_guard = greeter.write().await;

    // A tracing subscriber is installed once at startup and cannot be safely
    // replaced here. Keep its settings stable until the next restart.
    let logger_settings = (
      greeter_guard.general.debug,
      greeter_guard.general.log_file.clone(),
    );
    if greeter_guard.general.debug != config.general.debug
      || greeter_guard.general.log_file != config.general.log_file
    {
      warn!("general.debug and general.log_file changes require a restart");
    }

    // Applying an already validated configuration replaces all
    // configuration-owned runtime state while holding this write lock.
    greeter_guard.apply_config(config);
    (greeter_guard.general.debug, greeter_guard.general.log_file) =
      logger_settings;
    greeter_guard.reload_sessions();

    info!("Config hot reload completed successfully");
    Ok(())
  }
}
