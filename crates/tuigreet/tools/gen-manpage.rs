use std::fs;

use clap::CommandFactory;
use clap_mangen::Man;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let app = tuigreet_config::config::Config::command();

  // Get about text and convert to owned String
  let about = app
    .get_about()
    .map(|styled| styled.to_string())
    .unwrap_or_default();

  let cmd = app.long_about(about);

  let mut buffer = Vec::<u8>::new();
  Man::new(cmd).render(&mut buffer)?;

  // Write to contrib/man directory
  let output_path = "contrib/man/tuigreet-1";
  fs::write(output_path, &buffer)?;

  eprintln!("Generated {output_path}");

  Ok(())
}
