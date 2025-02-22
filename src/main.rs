use clap::Parser;
use globset::{Glob, GlobSetBuilder};
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

  #[arg(
    long = "ignore",
    help = "Glob patterns of files or directories to ignore",
    action = clap::ArgAction::Append
  )]
  ignore_patterns: Vec<String>,
}

struct ProcessPathOptions {
  include_hidden: bool,
  ignore_patterns: Vec<String>,
}

fn main() {
  let args = Args::parse();
  let process_path_options = ProcessPathOptions {
    include_hidden: args.include_hidden,
    ignore_patterns: args.ignore_patterns.clone(),
  };
  for path in &args.paths {
    process_path(Path::new(path), &process_path_options);
  }
}

fn process_path(path: &Path, options: &ProcessPathOptions) {
  let ProcessPathOptions {
    include_hidden,
    ignore_patterns,
  } = options;
  let mut walker = WalkBuilder::new(path);
  walker.hidden(!include_hidden);

  let mut glob_builder = GlobSetBuilder::new();
  for pattern in ignore_patterns {
    if let Ok(glob) = Glob::new(pattern) {
      glob_builder.add(glob);
    }
  }
  let glob_set = glob_builder.build().unwrap();

  let walker = walker
    .filter_entry(move |entry| !glob_set.is_match(entry.path()))
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
