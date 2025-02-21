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

  #[arg(
    long = "include-hidden",
    default_value_t = false,
    help = "Include hidden files in the output"
  )]
  include_hidden: bool,
}

fn main() {
  let args = Args::parse();
  for path in &args.paths {
    process_path(Path::new(path), args.include_hidden);
  }
}

fn process_path(path: &Path, include_hidden: bool) {
  let walker = WalkBuilder::new(path)
    .hidden(!include_hidden)
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
