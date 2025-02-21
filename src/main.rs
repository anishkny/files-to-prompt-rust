use ignore::WalkBuilder;
use std::env;
use std::fs;
use std::path::Path;
use std::str;

fn main() {
  let args: Vec<String> = env::args().skip(1).collect();
  if args.is_empty() {
    eprintln!("Usage: files-to-prompt <path1> [path2] ...");
    std::process::exit(1);
  }

  for path in &args {
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
