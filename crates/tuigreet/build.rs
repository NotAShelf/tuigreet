use std::{env, process::Command};

fn main() {
  println!(
    "cargo::error=This repository is archived; its changes have been merged \
     back into tuigreet."
  );
  println!(
    "cargo::error=The canonical repository is now https://github.com/tuigreet/tuigreet, where I'm joining as a co-maintainer."
  );
  println!(
    "cargo::error=Please ask your distro packagers to update the repository \
     location and package the new release."
  );

  let version =
    git_version().unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
  println!("cargo:rustc-env=VERSION={version}");
  println!("cargo:rustc-env=TARGET={}", env::var("TARGET").unwrap());
}

fn git_version() -> Option<String> {
  let out = Command::new("git")
    .args(["describe", "--long"])
    .output()
    .ok()?;
  if !out.status.success() {
    return None;
  }
  let s = String::from_utf8(out.stdout).ok()?;
  Some(s.trim().replacen('-', ".r", 1).replacen('-', ".", 1))
}
