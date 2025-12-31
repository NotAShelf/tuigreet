#[macro_use]
mod macros;

mod event;
mod greeter;
mod info;
mod ipc;
mod keyboard;
mod power;
mod ui;
mod watcher;

#[cfg(test)] mod integration;

use std::{error::Error, fs::OpenOptions, io, process, sync::Arc};

#[cfg(not(test))]
use crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};
use crossterm::{
  execute,
  terminal::{LeaveAlternateScreen, disable_raw_mode},
};
use event::Event;
use greetd_ipc::Request;
use power::PowerPostAction;
use tokio::sync::RwLock;
use tracing_appender::non_blocking::WorkerGuard;
use tui::{Terminal, backend::CrosstermBackend};
use tuigreet::AuthStatus;

pub use self::greeter::*;
use self::{event::Events, ipc::Ipc};

#[tokio::main]
async fn main() {
  let backend = CrosstermBackend::new(io::stdout());
  let events = Events::new().await;
  let greeter = Greeter::new(events.sender()).await;

  if let Err(error) = run(backend, greeter, events).await {
    if let Some(AuthStatus::Success) = error.downcast_ref::<AuthStatus>() {
      return;
    }

    process::exit(1);
  }
}

async fn run<B>(
  backend: B,
  mut greeter: Greeter,
  mut events: Events,
) -> Result<(), Box<dyn Error>>
where
  B: tui::backend::Backend,
{
  tracing::info!("tuigreet started");

  register_panic_handler();

  #[cfg(not(test))]
  {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
  }

  let mut terminal = Terminal::new(backend)?;

  #[cfg(not(test))]
  terminal.clear()?;

  let ipc = Ipc::new();

  if greeter.remember && !greeter.username.value.is_empty() {
    greeter.working = true;

    tracing::info!(
      "creating remembered session for user {}",
      greeter.username.value
    );

    ipc
      .send(Request::CreateSession {
        username: greeter.username.value.clone(),
      })
      .await;
  }

  let greeter = Arc::new(RwLock::new(greeter));

  // Initialize config watcher for hot reloading
  #[cfg(not(test))]
  let _config_watcher = {
    let config_path = {
      let greeter_guard = greeter.read().await;
      greeter_guard
        .config()
        .opt_str("config")
        .map(std::path::PathBuf::from)
    };

    match crate::watcher::ConfigWatcher::new(
      config_path,
      greeter.clone(),
      events.sender(),
    ) {
      Ok(watcher) => Some(watcher),
      Err(e) => {
        tracing::warn!("Failed to initialize config watcher: {}", e);
        None
      },
    }
  };

  tokio::task::spawn({
    let greeter = greeter.clone();
    let mut ipc = ipc.clone();

    async move {
      loop {
        let _ = ipc.handle(greeter.clone()).await;
      }
    }
  });

  loop {
    if let Some(status) = greeter.read().await.exit {
      tracing::info!("exiting main loop");

      return Err(status.into());
    }

    match events.next().await {
      Some(Event::Render) => ui::draw(greeter.clone(), &mut terminal).await?,
      Some(Event::Key(key)) => {
        keyboard::handle(greeter.clone(), key, ipc.clone()).await?
      },

      Some(Event::Exit(status)) => {
        crate::exit(&mut *greeter.write().await, status).await;
      },

      Some(Event::PowerCommand(command)) => {
        if let PowerPostAction::ClearScreen =
          power::run(&greeter, command).await
        {
          execute!(io::stdout(), LeaveAlternateScreen)?;
          terminal.set_cursor_position((1, 1))?;
          terminal.clear()?;
          disable_raw_mode()?;

          break;
        }
      },

      Some(Event::Refresh) => {
        // Config was hot reloaded, force a render
        ui::draw(greeter.clone(), &mut terminal).await?
      },

      _ => {},
    }
  }

  Ok(())
}

async fn exit(greeter: &mut Greeter, status: AuthStatus) {
  tracing::info!("preparing exit with status {}", status);

  match status {
    AuthStatus::Success => {},
    AuthStatus::Cancel | AuthStatus::Failure => Ipc::cancel(greeter).await,
  }

  #[cfg(not(test))]
  clear_screen();

  let _ = execute!(io::stdout(), LeaveAlternateScreen);
  let _ = disable_raw_mode();

  greeter.exit = Some(status);
}

fn register_panic_handler() {
  let hook = std::panic::take_hook();

  std::panic::set_hook(Box::new(move |info| {
    #[cfg(not(test))]
    clear_screen();

    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    let _ = disable_raw_mode();

    hook(info);
  }));
}

#[cfg(not(test))]
pub fn clear_screen() {
  let backend = CrosstermBackend::new(io::stdout());

  if let Ok(mut terminal) = Terminal::new(backend) {
    let _ = terminal.hide_cursor();
    let _ = terminal.clear();
  }
}

fn init_logger(greeter: &Greeter) -> Option<WorkerGuard> {
  use tracing_subscriber::{
    filter::{LevelFilter, Targets},
    prelude::*,
  };

  let logfile = OpenOptions::new()
    .write(true)
    .create(true)
    .append(true)
    .clone();

  match (greeter.debug, logfile.open(&greeter.logfile)) {
    (true, Ok(file)) => {
      let (appender, guard) = tracing_appender::non_blocking(file);
      let target = Targets::new().with_target("tuigreet", LevelFilter::DEBUG);

      tracing_subscriber::registry()
        .with(
          tracing_subscriber::fmt::layer()
            .with_writer(appender)
            .with_line_number(true),
        )
        .with(target)
        .init();

      Some(guard)
    },

    _ => None,
  }
}
