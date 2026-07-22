# tuigreet Changelog

## 0.12.0

### Added

- Suspend and hibernate actions are now available from the power menu. They use
  `loginctl suspend` and `loginctl hibernate` by default.
- `[power] suspend` and `[power] hibernate` configuration keys, plus the
  `--power-suspend` and `--power-hibernate` command-line options, allow
  overriding those actions.

### Changed

- `display.asterisks` is deprecated. Use `[secret] mode = "characters"` or
  `mode = "hidden"` instead. Existing configurations remain supported and emit a
  validation warning; `secret.mode` takes precedence when both are configured.
  The deprecated option **will be removed in a future release**.
- Configuration reload now uses the same configuration sources and command line
  overrides as initial startup.

### Fixed

- Reloading configuration now replaces removed session commands, session
  directories, wrappers, and user-menu state instead of retaining stale values.
- Explicit `secret.mode = "hidden"` and the legacy `display.asterisks = false`
  setting now correctly override lower-priority configuration layers.
- The top infobar now respects `layout.window_padding`.
- Background animations no longer bleed into the login form.
