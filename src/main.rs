use ignore::WalkBuilder;
use std::env;
use std::fs;
use std::path::Path;

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
  let walker = WalkBuilder::new(path).build();
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
  match fs::read_to_string(path) {
    Ok(contents) => {
      println!("{}\n----\n{}\n----\n", path.display(), contents);
    }
    Err(err) => eprintln!("Could not read {}: {}", path.display(), err),
  }
}
