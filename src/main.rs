use clap::Parser;
use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::{self, create_dir_all, File};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::str;

#[derive(Parser, Debug)]
#[command(
  version,
  about = "Concatenates a directory full of files into a single prompt for use with LLMs"
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
    help = "Output in XML-ish format suitable for Claude's long context window",
    conflicts_with_all = &["markdown", "json"]
  )]
  cxml: bool,

  #[arg(
    short = 'm',
    long = "markdown",
    default_value_t = false,
    help = "Output Markdown fenced code blocks",
    conflicts_with_all = &["cxml", "json"]
  )]
  markdown: bool,

  #[arg(
    short = 'o',
    long = "output",
    help = "Output file (default: stdout) or location (for -r, default: current directory)"
  )]
  output: Option<String>,

  #[arg(
    short = 'j',
    long = "json",
    default_value_t = false,
    help = "Output JSON compatible with CodeSandbox API/CLI",
    conflicts_with_all = &["cxml", "markdown"]
  )]
  json: bool,

  #[arg(
    short = 'r',
    long = "reverse",
    default_value_t = false,
    help = "Reverse operation. Read files from stdin and write to disk. For now, requires -c/--cxml.",
    requires = "cxml"
  )]
  reverse: bool,

  #[arg(
    short = 'e',
    long = "extension",
    help = "Only include files with the given extension(s)",
    action = clap::ArgAction::Append
  )]
  extensions: Vec<String>,
}

struct ProcessPathOptions {
  include_hidden: bool,
  ignore_patterns: Vec<String>,
  ignore_gitignore: bool,
  extensions: Vec<String>,
}

struct PrintFileOptions {
  line_numbers: bool,
  cxml: bool,
  markdown: bool,
  json: bool,
}

fn main() {
  let args = Args::parse();
  let process_path_options = ProcessPathOptions {
    include_hidden: args.include_hidden,
    ignore_patterns: args.ignore_patterns.clone(),
    ignore_gitignore: args.ignore_gitignore,
    extensions: args.extensions.clone(),
  };

  let print_file_options = PrintFileOptions {
    line_numbers: args.line_numbers,
    cxml: args.cxml,
    markdown: args.markdown,
    json: args.json,
  };

  if args.reverse {
    write_files_from_stdin(args.output);
    return;
  }

  // Collect all the files in the given paths
  let mut file_paths = BTreeSet::new();
  for path in &args.paths {
    process_path(Path::new(path), &process_path_options, &mut file_paths);
  }

  // Print the files to stdout (or a file if specified)
  let output: Box<dyn Write> = match &args.output {
    Some(output_file) => {
      Box::new(fs::File::create(output_file).expect("Could not create output file"))
    }
    None => Box::new(io::stdout()),
  };
  print_files(file_paths, &print_file_options, output);
}

fn process_path(path: &Path, options: &ProcessPathOptions, file_paths: &mut BTreeSet<String>) {
  let ProcessPathOptions {
    include_hidden,
    ignore_patterns,
    ignore_gitignore,
    extensions,
  } = options;
  let mut walker = WalkBuilder::new(path);
  walker.hidden(!include_hidden);
  walker.git_ignore(!ignore_gitignore);
  walker.require_git(false);

  let mut glob_builder = GlobSetBuilder::new();
  for pattern in ignore_patterns {
    if let Ok(glob) = Glob::new(pattern) {
      glob_builder.add(glob);
    }
  }
  let glob_set = glob_builder.build().unwrap();
  let extensions_set: HashSet<String> = extensions.iter().cloned().collect();

  let walker = walker
    .filter_entry(move |entry| {
      let path = entry.path();

      // Skip ignored files
      if glob_set.is_match(path) {
        return false;
      }

      // If extensions are specified, only include files with those extensions
      if !extensions_set.is_empty() {
        if let Some(ext) = path.extension() {
          if !extensions_set.contains(ext.to_str().unwrap()) {
            return false;
          }
        }
      }

      // Include all other files
      true
    })
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

fn print_files(
  file_paths: BTreeSet<String>,
  options: &PrintFileOptions,
  mut output: Box<dyn Write>,
) {
  if file_paths.is_empty() {
    writeln!(output, "No files found.").unwrap();
    return;
  }
  if options.cxml {
    writeln!(output, "<documents>").unwrap();
  } else if options.json {
    writeln!(output, "{{\n  \"files\": {{").unwrap();
  }
  let last_file_path = file_paths.iter().last().unwrap();
  for file_path in &file_paths {
    print_file(
      &Path::new(&file_path),
      options,
      &mut output,
      file_path == last_file_path,
    );
  }
  if options.cxml {
    writeln!(output, "</documents>").unwrap();
  } else if options.json {
    writeln!(output, "  }}\n}}").unwrap();
  }
}

fn print_file(path: &Path, options: &PrintFileOptions, output: &mut dyn Write, is_last: bool) {
  match fs::read(path) {
    Ok(bytes) => match str::from_utf8(&bytes) {
      Ok(contents) => {
        // Header
        if options.cxml {
          writeln!(output, "<document path=\"{}\">", path.display()).unwrap();
        } else if options.markdown {
          writeln!(
            output,
            "{}\n```{}",
            path.display(),
            path_to_markdown_language(path)
          )
          .unwrap();
        } else if options.json {
          writeln!(output, "    \"{}\": {{", path.display()).unwrap();
        } else {
          writeln!(output, "{}\n----", path.display()).unwrap();
        }

        // Contents
        if options.line_numbers {
          for (i, line) in contents.lines().enumerate() {
            writeln!(output, "{:>4}  {}", i + 1, line).unwrap();
          }
        } else if options.json {
          writeln!(output, "      \"contents\": {:?}", contents).unwrap();
        } else {
          write!(output, "{}", contents).unwrap();
        }

        // Footer
        if options.cxml {
          writeln!(output, "</document>").unwrap();
        } else if options.markdown {
          writeln!(output, "```\n").unwrap();
        } else if options.json {
          if is_last {
            writeln!(output, "    }}").unwrap();
          } else {
            writeln!(output, "    }},").unwrap();
          }
        } else {
          writeln!(output, "\n----\n").unwrap();
        }
      }
      Err(_) => eprintln!("Warning: Skipping non-UTF-8 file: {}", path.display()),
    },
    Err(err) => eprintln!("Could not read {}: {}", path.display(), err),
  }
}

// Map file extension to markdown fence language
fn path_to_markdown_language(path: &Path) -> String {
  let ext_to_lang: HashMap<&str, &str> = [
    ("c", "c"),
    ("cpp", "cpp"),
    ("css", "css"),
    ("html", "html"),
    ("java", "java"),
    ("js", "javascript"),
    ("json", "json"),
    ("py", "python"),
    ("rb", "ruby"),
    ("sh", "bash"),
    ("ts", "typescript"),
    ("xml", "xml"),
    ("yaml", "yaml"),
    ("yml", "yaml"),
  ]
  .iter()
  .cloned()
  .collect();

  if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
    return ext_to_lang.get(extension).unwrap_or(&extension).to_string();
  }

  "plaintext".to_string()
}

// Read file contents from stdin and write to disk
fn write_files_from_stdin(output: Option<String>) {
  let stdin = io::stdin();
  let base_path = output.unwrap_or_else(|| "./".to_string());

  let mut current_path: Option<String> = None;
  let mut content = Vec::new();

  for line in stdin.lock().lines() {
    let line = line.unwrap();
    if line.starts_with("<document path=") {
      if let Some(path) = current_path.take() {
        write_to_file(&base_path, &path, &content);
      }
      current_path = Some(line.split('"').nth(1).unwrap_or_default().to_string());
      content.clear();
    } else if line == "</document>" {
      if let Some(path) = current_path.take() {
        write_to_file(&base_path, &path, &content);
      }
    } else {
      content.push(line);
    }
  }
}

fn write_to_file(base_path: &str, relative_path: &str, content: &[String]) {
  let full_path = Path::new(base_path).join(relative_path);
  if let Some(parent) = full_path.parent() {
    create_dir_all(parent).expect("Failed to create directory");
  }
  let mut file = File::create(full_path).expect("Failed to create file");
  for line in content {
    writeln!(file, "{}", line).expect("Failed to write to file");
  }
}
