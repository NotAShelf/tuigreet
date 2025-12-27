{
  mkShell,
  rustc,
  cargo,
  rust-analyzer-unwrapped,
  rustfmt,
  clippy,
  taplo,
  rustPlatform,
  cargo-nextest,
}:
mkShell {
  name = "rust";

  strictDeps = true;
  packages = [
    rustc
    cargo

    # Tools
    rust-analyzer-unwrapped # LSP
    (rustfmt.override {asNightly = true;}) # formatter
    clippy # linter
    taplo # TOML formatter

    # Additional Cargo Tooling
    cargo-nextest
  ];

  RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
}
