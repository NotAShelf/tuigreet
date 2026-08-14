use std::{process::Stdio, sync::Arc};

use tokio::{process::Command, sync::RwLock};
use tuigreet_types::Mode;

use crate::{Greeter, event::Event, ui::power::Power};

/// Power management options
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PowerOption {
  #[default]
  Shutdown,
  Reboot,
  Suspend,
  Hibernate,
}

/// Execute a power command (shutdown or reboot).
pub async fn power(greeter: &mut Greeter, option: PowerOption) {
  if greeter.general.mock {
    if let Some(ref sender) = greeter.events {
      let _ = sender
        .send(Event::Exit(tuigreet_types::AuthStatus::Cancel))
        .await;
    }
    return;
  }

  let command = match greeter
    .powers
    .options
    .iter()
    .find(|opt| opt.action == option)
  {
    None => None,

    Some(Power {
      command: Some(args),
      ..
    }) => {
      let command = if greeter.power.use_setsid {
        let mut command = Command::new("setsid");
        command.args(args.split(' '));
        command
      } else {
        let mut args = args.split(' ');

        let mut command = Command::new(args.next().unwrap_or_default());
        command.args(args);
        command
      };

      Some(command)
    },

    Some(_) => {
      let command = match option {
        PowerOption::Shutdown => {
          let mut command = Command::new("shutdown");
          command.args(["-h", "now"]);
          command
        },
        PowerOption::Reboot => {
          let mut command = Command::new("shutdown");
          command.args(["-r", "now"]);
          command
        },
        // `loginctl` is provided by both systemd and elogind, so these
        // defaults work across systemd and non-systemd distributions.
        PowerOption::Suspend => {
          let mut command = Command::new("loginctl");
          command.arg("suspend");
          command
        },
        PowerOption::Hibernate => {
          let mut command = Command::new("loginctl");
          command.arg("hibernate");
          command
        },
      };

      Some(command)
    },
  };

  if let Some(mut command) = command {
    command.stdin(Stdio::null());
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());

    if let Some(ref sender) = greeter.events {
      let _ = sender.send(Event::PowerCommand(command)).await;
    }
  }
}

pub enum PowerPostAction {
  Noop,
  ClearScreen,
}

pub async fn run(
  greeter: &Arc<RwLock<Greeter>>,
  mut command: Command,
) -> PowerPostAction {
  tracing::info!("executing power command: {:?}", command);

  greeter.write().await.mode = Mode::Processing;

  let message = match command.output().await {
    Ok(result) => {
      match (result.status, result.stderr) {
        (status, _) if status.success() => None,
        (status, output) => {
          let status = format!("{} {status}", fl!("command_exited"));
          let output = String::from_utf8(output).unwrap_or_default();

          Some(format!("{status}\n{output}"))
        },
      }
    },

    Err(err) => Some(format!("{}: {err}", fl!("command_failed"))),
  };

  tracing::info!("power command exited with: {:?}", message);

  let mode = greeter.read().await.previous_mode;

  let mut greeter = greeter.write().await;

  if message.is_none() {
    PowerPostAction::ClearScreen
  } else {
    greeter.mode = mode;
    greeter.message = message;

    PowerPostAction::Noop
  }
}
