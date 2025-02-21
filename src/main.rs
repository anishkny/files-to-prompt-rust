use clap::Parser;
use ignore::WalkBuilder;
use std::fs;
use std::path::Path;
use std::str;

#[derive(Parser, Debug)]
#[command(
  version,
  about = "Concatenates a directory full of files into a single prompt for use with LLMs."
)]
struct Args {
  #[arg(default_value = ".")]
  paths: Vec<String>,
}

fn main() {
  let args = Args::parse();
  for path in &args.paths {
    process_path(Path::new(path));
  }
}

fn process_path(path: &Path) {
  let walker = WalkBuilder::new(path)
    .sort_by_file_path(|a, b| a.cmp(b))
    .build();
  for result in walker {
    match result {
      Ok(entry) => {
        let entry_path = entry.path();
        if entry_path.is_file() {
          print_file(entry_path);
        }
      }
      Err(err) => eprintln!("Error: {}", err),
    }
  }
}

fn print_file(path: &Path) {
  match fs::read(path) {
    Ok(bytes) => match str::from_utf8(&bytes) {
      Ok(contents) => println!("{}\n----\n{}\n----\n", path.display(), contents),
      Err(_) => eprintln!("Warning: Skipping non-UTF-8 file: {}", path.display()),
    },
    Err(err) => eprintln!("Could not read {}: {}", path.display(), err),
  }
}
