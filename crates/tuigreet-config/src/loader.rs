use clap::{
  ArgMatches,
  CommandFactory,
  FromArgMatches,
  Parser,
  parser::ValueSource,
};
use figment::{
  Figment,
  providers::{Env, Format, Serialized, Toml},
};
use tuigreet_types::expand_tilde;

use crate::{config::Config, error::ConfigError};

/// Clears any argument whose value came from clap's `default_value` (not
/// genuinely supplied by the user via CLI or env), leaving a sparse
/// `ArgMatches` containing only explicitly-provided values.
///
/// This is the key trick that lets CLI-provided values act as the highest
/// priority layer without clap's own defaults unconditionally clobbering
/// TOML/env layers underneath them.
fn strip_defaults(matches: &ArgMatches) -> ArgMatches {
  let mut stripped = matches.clone();
  for id in matches.ids() {
    let key = id.as_str();
    if matches.value_source(key) == Some(ValueSource::DefaultValue) {
      let _ = stripped.try_clear_id(key);
    }
  }
  stripped
}

/// Loads config layering TOML -> env -> CLI (CLI wins, but only for
/// explicitly-supplied values; clap's own defaults act as the floor), then
/// validates the fully-resolved result.
///
/// Takes an explicit argv (including program name at index 0) so this can be
/// exercised deterministically in tests instead of reading `std::env::args()`.
pub fn load_config_from<I, T>(
  args: I,
) -> Result<
  (Config, Vec<ConfigError>),
  (Config, (Vec<ConfigError>, Vec<ConfigError>)),
>
where
  I: IntoIterator<Item = T>,
  T: Into<std::ffi::OsString> + Clone,
{
  let args_vec: Vec<std::ffi::OsString> =
    args.into_iter().map(Into::into).collect();
  let matches = Config::command().get_matches_from(args_vec.clone());
  let matches_no_defaults = strip_defaults(&matches);

  // clap-derived defaults become the lowest priority figment layer
  let cli_defaults = Config::from_arg_matches(&matches)
    .expect("matches were produced by Config::command()");

  let cli = Config::parse_from(args_vec);

  // Access directly as struct fields
  let no_config = cli.general.no_config;
  let system_config_path = expand_tilde(&cli.general.system_config_path);
  let user_config_path = expand_tilde(&cli.general.user_config_path);

  let mut figment = Figment::new().merge(Serialized::defaults(cli_defaults)); // baseline = clap defaults
  // set config in case next steps fail, still have options from cli
  let mut config: Config = match figment.extract() {
    Ok(c) => c,
    Err(e) => {
      return Err((
        Config::defaults(),
        (vec![ConfigError::Figment(e)], vec![]),
      ));
    },
  };

  if !no_config {
    // Only merge system config if it exists
    if system_config_path.exists() {
      figment = figment.merge(Toml::file(&system_config_path));
      config = match figment.extract() {
        Ok(c) => c,
        Err(e) => {
          return Err((config, (vec![ConfigError::Figment(e)], vec![])));
        },
      };
    } else {
      tracing::debug!(
        "System config not found, skipping: {}",
        &cli.general.system_config_path
      );
    }

    // Only merge user config if it exists
    if user_config_path.exists() {
      figment = figment.merge(Toml::file(&user_config_path));
      config = match figment.extract() {
        Ok(c) => c,
        Err(e) => {
          return Err((config, (vec![ConfigError::Figment(e)], vec![])));
        },
      };
    } else {
      tracing::debug!(
        "User config not found, skipping: {}",
        &cli.general.user_config_path
      );
    }
  }
  // NOTE: use a double underscore as the section/field separator, not a
  // single underscore. `log_level` (and any other snake_case field
  // name) contains an underscore itself, so splitting on a single "_"
  // would incorrectly turn `TUIGREET_LOG_LEVEL` into the nested
  // path `general.log.level` instead of `general.log_level`.
  figment = figment.merge(Env::prefixed("TUIGREET_").split("__")); // overrides toml

  config = match figment.extract() {
    Ok(c) => c,
    Err(e) => {
      return Err((config, (vec![ConfigError::Figment(e)], vec![])));
    },
  };

  // apply only args the user actually supplied on the CLI (final override)
  config
    .update_from_arg_matches(&matches_no_defaults)
    .expect("matches were derived from same Config type");

  match config.validate(false) {
    Ok(warnings) => Ok((config, warnings)),
    Err((errors, warnings)) => Err((config, (errors, warnings))),
  }
}

/// Loads config using the real process argv.
pub fn load_config() -> Result<
  (Config, Vec<ConfigError>),
  (Config, (Vec<ConfigError>, Vec<ConfigError>)),
> {
  load_config_from(std::env::args())
}
