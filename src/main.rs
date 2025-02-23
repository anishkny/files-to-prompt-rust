use clap::Parser;
use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;
use std::collections::BTreeSet;
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

  #[arg(
    long = "ignore-gitignore",
    default_value_t = false,
    help = "Ignore .gitignore files when scanning directories"
  )]
  ignore_gitignore: bool,

  #[arg(
    short = 'n',
    long = "line-numbers",
    default_value_t = false,
    help = "Include line numbers in the output"
  )]
  line_numbers: bool,

  #[arg(
    short = 'c',
    long = "cxml",
    default_value_t = false,
    help = "Output in XML-ish format suitable for Claude's long context window."
  )]
  cxml: bool,
}

struct ProcessPathOptions {
  include_hidden: bool,
  ignore_patterns: Vec<String>,
  ignore_gitignore: bool,
}

struct PrintFileOptions {
  line_numbers: bool,
  cxml: bool,
}

fn main() {
  let args = Args::parse();
  let process_path_options = ProcessPathOptions {
    include_hidden: args.include_hidden,
    ignore_patterns: args.ignore_patterns.clone(),
    ignore_gitignore: args.ignore_gitignore,
  };

  let print_file_options = PrintFileOptions {
    line_numbers: args.line_numbers,
    cxml: args.cxml,
  };

  // Collect all the files in the given paths
  let mut file_paths = BTreeSet::new();
  for path in &args.paths {
    process_path(Path::new(path), &process_path_options, &mut file_paths);
  }

  // Print the files
  print_files(file_paths, &print_file_options);
}

fn process_path(path: &Path, options: &ProcessPathOptions, file_paths: &mut BTreeSet<String>) {
  let ProcessPathOptions {
    include_hidden,
    ignore_patterns,
    ignore_gitignore,
  } = options;
  let mut walker = WalkBuilder::new(path);
  walker.hidden(!include_hidden);
  walker.git_ignore(!ignore_gitignore);

  let mut glob_builder = GlobSetBuilder::new();
  for pattern in ignore_patterns {
    if let Ok(glob) = Glob::new(pattern) {
      glob_builder.add(glob);
    }
  }
  let glob_set = glob_builder.build().unwrap();

  let walker = walker
    .filter_entry(move |entry| !glob_set.is_match(entry.path()))
    .build();

  for result in walker {
    match result {
      Ok(entry) => {
        let entry_path = entry.path();
        if entry_path.is_file() {
          if let Some(file_path) = entry_path.to_str() {
            file_paths.insert(file_path.to_string());
          }
        }
      }
      Err(err) => eprintln!("Error: {}", err),
    }
  }
}

fn print_files(file_paths: BTreeSet<String>, options: &PrintFileOptions) {
  if options.cxml {
    println!("<documents>");
  }
  for file_path in file_paths {
    print_file(&Path::new(&file_path), options);
  }
  if options.cxml {
    println!("</documents>");
  }
}

fn print_file(path: &Path, options: &PrintFileOptions) {
  match fs::read(path) {
    Ok(bytes) => match str::from_utf8(&bytes) {
      Ok(contents) => {
        // Header
        if options.cxml {
          println!("<document path=\"{}\">", path.display());
        } else {
          println!("{}\n----", path.display());
        }

        // Contents
        if options.line_numbers {
          for (i, line) in contents.lines().enumerate() {
            println!("{:>4}  {}", i + 1, line);
          }
        } else {
          print!("{}", contents);
        }

        // Footer
        if options.cxml {
          println!("</document>");
        } else {
          println!("\n----\n");
        }
      }
      Err(_) => eprintln!("Warning: Skipping non-UTF-8 file: {}", path.display()),
    },
    Err(err) => eprintln!("Could not read {}: {}", path.display(), err),
  }
}
