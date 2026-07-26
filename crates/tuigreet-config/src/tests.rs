#[cfg(test)]
mod tests {
  use figment::Jail;
  use tuigreet_types::DEFAULT_LOG_FILE;

  use crate::loader::load_config_from;

  /// No TOML, no env, no CLI args beyond program name -> pure clap defaults.
  #[test]
  fn defaults_when_nothing_provided() {
    Jail::expect_with(|_jail| {
      let (cfg, warnings) = load_config_from(["tuigreet"]).unwrap();

      assert!(!cfg.general.debug);
      assert_eq!(cfg.general.log_file, DEFAULT_LOG_FILE.to_string());
      assert_eq!(cfg.general.numlock, false);
      assert!(warnings.is_empty(), "Expected no warnings");

      Ok(())
    });
  }
  // #[test]
  // fn test_env_vars_processed_once_with_multiple_theme_components() {
  // unsafe {
  // env::set_var("TUIGREET_THEME", "border=red;text=blue;container=green");
  // env::set_var("TUIGREET_SESSIONS_DIRS", "/test:/usr/share");
  // env::set_var("TUIGREET_ALIGN_GREETING", "center");
  // }
  //
  // let config = load_env_variables();
  //
  // Verify all theme components applied
  // assert_eq!(config.theme.border, Some("red".to_string()));
  // assert_eq!(config.theme.text, Some("blue".to_string()));
  // assert_eq!(config.theme.container, Some("green".to_string()));
  //
  // Verify other env vars applied correctly (once, not per-component)
  // assert_eq!(config.session.sessions_dirs, vec![
  // "/test".to_string(),
  // "/usr/share".to_string()
  // ]);
  // assert_eq!(config.display.align_greeting, AlignGreeting::Center);
  //
  // unsafe {
  // env::remove_var("TUIGREET_THEME");
  // env::remove_var("TUIGREET_SESSIONS_DIRS");
  // env::remove_var("TUIGREET_ALIGN_GREETING");
  // }
  // }
  // #[test]
  // fn toml_layers_preserve_explicit_default_values() {
  // let mut system: toml::Table =
  // toml::from_str("[display]\nshow_time = true\n[layout]\nwidth = 120\n")
  // .expect("system config is valid");
  // let user: toml::Table =
  // toml::from_str("[display]\nshow_time = false\n[layout]\nwidth = 80\n")
  // .expect("user config is valid");
  //
  // merge_toml_tables(&mut system, user);
  // let config: Config = toml::Value::Table(system)
  // .try_into()
  // .expect("merged config is valid");
  //
  // assert!(!config.display.show_time);
  // assert_eq!(config.layout.width, 80);
  // }
  //
  // #[test]
  // fn canonical_secret_mode_wins_over_legacy_alias() {
  // let mut config: Config = toml::from_str(
  // r#"
  // [display]
  // asterisks = true
  //
  // [secret]
  // mode = "hidden"
  // "#,
  // )
  // .expect("configuration must parse");
  // config.secret_mode_specified = true;
  //
  // normalize_legacy_asterisks(&mut config);
  //
  // assert_eq!(config.secret.mode, SecretMode::Hidden);
  // }
  //
  // #[test]
  // fn legacy_false_overrides_a_lower_characters_layer() {
  // let mut resolved = Config::default();
  // resolved.secret.mode = SecretMode::Characters;
  //
  // let mut legacy_layer = Config::default();
  // legacy_layer.display.asterisks = Some(false);
  // apply_config_layer(&mut resolved, legacy_layer);
  //
  // assert_eq!(resolved.secret.mode, SecretMode::Hidden);
  // assert!(
  // resolved
  // .validate(false)
  // .expect("legacy configuration is valid")
  // .iter()
  // .any(|warning| warning.contains("display.asterisks is deprecated"))
  // );
  // }
  //
  // #[test]
  // fn canonical_higher_layer_overrides_legacy_alias() {
  // let mut resolved = Config::default();
  // resolved.display.asterisks = Some(true);
  //
  // let mut canonical_layer = Config::default();
  // canonical_layer.secret.mode = SecretMode::Hidden;
  // canonical_layer.secret_mode_specified = true;
  // apply_config_layer(&mut resolved, canonical_layer);
  // normalize_legacy_asterisks(&mut resolved);
  //
  // assert_eq!(resolved.secret.mode, SecretMode::Hidden);
  // }
  //
  // #[test]
  // fn test_mutual_exclusive_remember_flags() {
  // let toml_content = r"
  // [remember]
  // username = true
  // session = true
  // user_session = true
  // ";
  //
  // let config: Config =
  // toml::from_str(toml_content).expect("Failed to parse TOML");
  //
  // assert!(config.remember.session);
  // assert!(config.remember.user_session);
  //
  // Both flags being set is now a warning, not a hard error, so that the
  // rest of the config is still applied. user_session takes behavioral
  // precedence when both are true.
  // let result = config.validate(false);
  // assert!(
  // result.is_ok(),
  // "Both remember.session and remember.user_session being true should \
  // produce a warning, not an error"
  // );
  //
  // let warnings = result.unwrap();
  // assert!(
  // warnings.iter().any(|w| {
  // w.contains("remember.session") && w.contains("remember.user_session")
  // }),
  // "Expected a warning about conflicting remember options, got: \
  // {warnings:?}"
  // );
  // }
  //
  // #[test]
  // fn test_keybindings_distinctness_in_config() {
  // let toml_content = r"
  // [keybindings]
  // command = 3
  // sessions = 3
  // power = 7
  // ";
  //
  // let config: Config =
  // toml::from_str(toml_content).expect("Failed to parse TOML");
  // let validation_result = config.validate(false);
  //
  // match validation_result {
  // Err(ConfigError::DuplicateKeybindings) => {},
  // _ => {
  // panic!(
  // "Expected DuplicateKeybindings error, got: {validation_result:?}"
  // );
  // },
  // }
  // }
  //
  // #[test]
  // fn test_session_config_default_consistency() {
  // let default_config = Config::default();
  //
  // let partial_toml = r#"
  // [session]
  // command = "test"
  // "#;
  // let partial_config: Config =
  // toml::from_str(partial_toml).expect("Failed to parse partial TOML");
  //
  // assert_eq!(
  // default_config.session.sessions_dirs,
  // partial_config.session.sessions_dirs,
  // "Default and partially deserialized sessions_dirs should match"
  // );
  // }
  //
  // #[test]
  // fn test_power_config_default_consistency() {
  // let default_config = Config::default();
  //
  // let partial_toml = r#"
  // [power]
  // shutdown = "poweroff"
  // "#;
  // let partial_config: Config =
  // toml::from_str(partial_toml).expect("Failed to parse partial TOML");
  //
  // assert_eq!(
  // default_config.power.use_setsid, partial_config.power.use_setsid,
  // "Default and partially deserialized use_setsid should match"
  // );
  // }
  //
  // #[test]
  // fn test_wrapper_validation_empty_string() {
  // let empty_wrapper = r#"
  // [session]
  // session_wrapper = ""
  // "#;
  //
  // let mut config: Config =
  // toml::from_str(empty_wrapper).expect("Failed to parse TOML");
  //
  // config.session.xsession_wrapper = None;
  //
  // let result = config.validate(true);
  //
  // assert!(
  // result.is_err(),
  // "Empty wrapper command should fail validation"
  // );
  // }
  //
  // [[outputs]] validation
  // #[test]
  // fn test_outputs_toml_roundtrip() {
  // let toml_content = r#"
  // [[outputs]]
  // connector = "DP-1"
  // primary = true
  //
  // [[outputs]]
  // connector = "HDMI-A-1"
  // enabled = false
  // "#;
  // let config: Config =
  // toml::from_str(toml_content).expect("Failed to parse [[outputs]] TOML");
  //
  // assert_eq!(config.outputs.len(), 2);
  // assert_eq!(config.outputs[0].connector, "DP-1");
  // assert!(config.outputs[0].primary);
  // assert!(config.outputs[0].enabled); // default = true
  // assert_eq!(config.outputs[1].connector, "HDMI-A-1");
  // assert!(!config.outputs[1].primary); // default = false
  // assert!(!config.outputs[1].enabled);
  //
  // Validation should pass
  // assert!(config.validate(false).is_ok());
  // }
  //
  // #[test]
  // fn test_outputs_multiple_primary_is_error() {
  // let toml_content = r#"
  // [[outputs]]
  // connector = "DP-1"
  // primary = true
  //
  // [[outputs]]
  // connector = "HDMI-A-1"
  // primary = true
  // "#;
  // let config: Config =
  // toml::from_str(toml_content).expect("Failed to parse TOML");
  // let result = config.validate(false);
  // assert!(
  // matches!(result, Err(ConfigError::Validation(_))),
  // "Multiple primary outputs should be a Validation error, got: {result:?}"
  // );
  // }
  //
  // #[test]
  // fn test_outputs_empty_connector_is_error() {
  // let toml_content = r#"
  // [[outputs]]
  // connector = ""
  // "#;
  // let config: Config =
  // toml::from_str(toml_content).expect("Failed to parse TOML");
  // let result = config.validate(false);
  // assert!(
  // matches!(result, Err(ConfigError::Validation(_))),
  // "Empty connector name should be a Validation error, got: {result:?}"
  // );
  // }
  //
  // #[test]
  // fn test_outputs_path_separator_in_connector_is_error() {
  // for bad in &["../DP-1", "/sys/class/drm/DP-1", "foo/bar"] {
  // let config: Config =
  // toml::from_str(&format!("[[outputs]]\nconnector = \"{bad}\"\n"))
  // .expect("Failed to parse TOML");
  // let result = config.validate(false);
  // assert!(
  // matches!(result, Err(ConfigError::Validation(_))),
  // "Connector '{bad}' with path separator should be a Validation error, \
  // got: {result:?}"
  // );
  // }
  // }
  //
  // #[test]
  // fn test_outputs_valid_connector_names() {
  // Typical DRM connector name patterns that must pass
  // for good in &[
  // "DP-1",
  // "HDMI-A-1",
  // "DisplayPort-2",
  // "eDP-1",
  // "VGA-1",
  // "DVI-D-1",
  // ] {
  // let config: Config =
  // toml::from_str(&format!("[[outputs]]\nconnector = \"{good}\"\n"))
  // .expect("Failed to parse TOML");
  // assert!(
  // config.validate(false).is_ok(),
  // "Connector '{good}' should be valid, but validation failed"
  // );
  // }
  // }
  //
  // #[test]
  // fn test_outputs_all_disabled_is_warning() {
  // let toml_content = r#"
  // [[outputs]]
  // connector = "DP-1"
  // enabled = false
  //
  // [[outputs]]
  // connector = "HDMI-A-1"
  // enabled = false
  // "#;
  // let config: Config =
  // toml::from_str(toml_content).expect("Failed to parse TOML");
  // let result = config.validate(false);
  // assert!(
  // result.is_ok(),
  // "All-disabled outputs should not be an error"
  // );
  // let warnings = result.unwrap();
  // assert!(
  // warnings.iter().any(|w| w.contains("enabled = false")),
  // "Expected a warning about all outputs being disabled, got: {warnings:?}"
  // );
  // }
  //
  // #[test]
  // fn test_outputs_single_primary_passes() {
  // let toml_content = r#"
  // [[outputs]]
  // connector = "DP-1"
  // primary = true
  //
  // [[outputs]]
  // connector = "HDMI-A-1"
  // "#;
  // let config: Config =
  // toml::from_str(toml_content).expect("Failed to parse TOML");
  // assert!(config.validate(false).is_ok());
  // }
  //
  // [terminal] validation
  // #[test]
  // fn test_terminal_both_set_passes() {
  // let toml_content = r"
  // [terminal]
  // cols = 237
  // rows = 52
  // ";
  // let config: Config =
  // toml::from_str(toml_content).expect("Failed to parse TOML");
  // assert_eq!(config.terminal.cols, Some(237));
  // assert_eq!(config.terminal.rows, Some(52));
  // assert!(config.validate(false).is_ok());
  // }
  //
  // #[test]
  // fn test_terminal_cols_without_rows_is_error() {
  // let toml_content = r"
  // [terminal]
  // cols = 237
  // ";
  // let config: Config =
  // toml::from_str(toml_content).expect("Failed to parse TOML");
  // let result = config.validate(false);
  // assert!(
  // matches!(result, Err(ConfigError::Validation(_))),
  // "cols without rows should be a Validation error, got: {result:?}"
  // );
  // }
  //
  // #[test]
  // fn test_terminal_rows_without_cols_is_error() {
  // let toml_content = r"
  // [terminal]
  // rows = 52
  // ";
  // let config: Config =
  // toml::from_str(toml_content).expect("Failed to parse TOML");
  // let result = config.validate(false);
  // assert!(
  // matches!(result, Err(ConfigError::Validation(_))),
  // "rows without cols should be a Validation error, got: {result:?}"
  // );
  // }
  //
  // #[test]
  // fn test_terminal_neither_set_passes() {
  // let config = Config::default();
  // assert!(config.validate(false).is_ok());
  // }
  //
  // #[test]
  // fn test_terminal_zero_cols_is_error() {
  // let mut config = Config::default();
  // config.terminal.cols = Some(0);
  // config.terminal.rows = Some(52);
  // let result = config.validate(false);
  // assert!(
  // matches!(result, Err(ConfigError::Validation(_))),
  // "cols = 0 should be a Validation error, got: {result:?}"
  // );
  // }
  //
  // #[test]
  // fn test_terminal_zero_rows_is_error() {
  // let mut config = Config::default();
  // config.terminal.cols = Some(237);
  // config.terminal.rows = Some(0);
  // let result = config.validate(false);
  // assert!(
  // matches!(result, Err(ConfigError::Validation(_))),
  // "rows = 0 should be a Validation error, got: {result:?}"
  // );
  // }
  //
  // #[test]
  // fn test_wrapper_validation_whitespace_only() {
  // let whitespace_wrapper = r#"
  // [session]
  // session_wrapper = "   "
  // "#;
  //
  // let mut config: Config =
  // toml::from_str(whitespace_wrapper).expect("Failed to parse TOML");
  //
  // config.session.xsession_wrapper = None;
  //
  // let result = config.validate(true);
  //
  // assert!(
  // result.is_err(),
  // "Whitespace-only wrapper command should fail validation"
  // );
  // }
  //
  // Config priority tests
  //
  // #[test]
  // fn test_cli_only_no_config() {
  // let mut config = Config::default();
  //
  // let mut cli = Config::default();
  // cli.keybindings.sessions = 5;
  //
  // apply_config_layer(&mut config, cli);
  //
  // assert_eq!(config.keybindings.sessions, 5);
  // }
  //
  // #[test]
  // fn test_config_overrides_defaults() {
  // let mut config = Config::default();
  //
  // let mut file_config = Config::default();
  // file_config.keybindings.command = 3;
  //
  // apply_config_layer(&mut config, file_config);
  //
  // assert_eq!(config.keybindings.command, 3);
  // }
  //
  // #[test]
  // fn test_user_overrides_system() {
  // let mut config = Config::default();
  // config.keybindings.command = 5;
  //
  // let mut user = Config::default();
  // user.keybindings.command = 7;
  //
  // apply_config_layer(&mut config, user);
  //
  // assert_eq!(config.keybindings.command, 7);
  // }
  //
  // #[test]
  // fn test_env_overrides_user() {
  // let mut config = Config::default();
  // config.keybindings.power = 10;
  //
  // let mut env = Config::default();
  // env.keybindings.power = 8;
  //
  // apply_config_layer(&mut config, env);
  //
  // assert_eq!(config.keybindings.power, 8);
  // }
  //
  // #[test]
  // fn test_cli_overrides_env() {
  // let mut config = Config::default();
  // config.keybindings.sessions = 3;
  //
  // let mut cli = Config::default();
  // cli.keybindings.sessions = 9;
  //
  // apply_config_layer(&mut config, cli);
  //
  // assert_eq!(config.keybindings.sessions, 9);
  // }
  //
  // #[test]
  // fn test_full_priority_chain() {
  // let mut config = Config::default();
  // assert_eq!(config.keybindings.command, 2);
  //
  // let mut system = Config::default();
  // system.keybindings.command = 5;
  // apply_config_layer(&mut config, system);
  // assert_eq!(config.keybindings.command, 5);
  //
  // let mut user = Config::default();
  // user.keybindings.command = 7;
  // apply_config_layer(&mut config, user);
  // assert_eq!(config.keybindings.command, 7);
  //
  // let mut env = Config::default();
  // env.keybindings.command = 9;
  // apply_config_layer(&mut config, env);
  // assert_eq!(config.keybindings.command, 9);
  //
  // let mut cli = Config::default();
  // cli.keybindings.command = 11;
  // apply_config_layer(&mut config, cli);
  // assert_eq!(config.keybindings.command, 11);
  // }
  //
  // #[test]
  // fn test_lower_layer_preserved_when_higher_layer_uses_defaults() {
  // System config sets a non-default value
  // let mut config = Config::default();
  // config.keybindings.power = 10;
  // config.display.show_time = true;
  // config.remember.username = true;
  //
  // User config only touches one unrelated field; all others remain at
  // their defaults and must NOT overwrite the system values above.
  // let mut user = Config::default();
  // user.display.greeting = Some("hello".to_string());
  // apply_config_layer(&mut config, user);
  //
  // assert_eq!(
  // config.keybindings.power, 10,
  // "system keybinding must survive"
  // );
  // assert!(config.display.show_time, "system show_time must survive");
  // assert!(
  // config.remember.username,
  // "system remember.username must survive"
  // );
  // assert_eq!(config.display.greeting, Some("hello".to_string()));
  // }
}
