use goldenfile::Mint;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use test_case::test_case;

mod helpers;
use helpers::rename_gitignore_files;

const TARGET: &str = if cfg!(debug_assertions) {
  "./target/debug/files-to-prompt"
} else {
  "./target/release/files-to-prompt"
};

#[test_case(&["tests/inputs/01_basic"], "01_basic.golden.txt"; "01_basic")]
#[test_case(
  &["tests/inputs/02_multiple_folders/f1", "tests/inputs/02_multiple_folders/f2"],
  "02_multiple_folders.golden.txt"; "02_multiple_folders"
)]
#[test_case(&["tests/inputs/03_gitignore"], "03_gitignore.golden.txt"; "03_gitignore")]
#[test_case(&["tests/inputs/04_invalid_utf8"], "04_invalid_utf8.golden.txt"; "04_invalid_utf8")]
#[test_case(&["tests/inputs/05_hidden"], "05_hidden.golden.txt"; "05_hidden")]
#[test_case(&["tests/inputs/05_hidden", "--include-hidden"], "05a_include_hidden.golden.txt"; "05a_include_hidden")]
#[test_case(&["tests/inputs/06_ignore", "--ignore", "*.md", "--ignore", "*.csv"], "06_ignore.golden.txt"; "06_ignore")]
#[test_case(&["tests/inputs/07_ignore_gitignore", "--ignore-gitignore"], "07_ignore_gitignore.golden.txt"; "07_ignore_gitignore")]
#[test_case(&["tests/inputs/01_basic", "--line-numbers"], "08_line_numbers.golden.txt"; "08_line_numbers")]
fn test_files_to_prompt(input: &[&str], golden_filename: &str) {
  // Rename gitignore files to .gitignore if they exist
  let _gitignore_renamers = rename_gitignore_files(input);

  // Run the CLI tool
  println!("Running TARGET [{:}] with input: {:?}", TARGET, input);
  let output = Command::new(TARGET)
    .args(input.iter())
    .output()
    .expect("Failed to execute command");

  // Ensure the command was successful
  assert!(output.status.success());

  // Write the output to a golden file
  let mut mint = Mint::new("tests/golden");
  let golden_path = Path::new(golden_filename);
  let mut golden_file = mint
    .new_goldenfile(golden_path)
    .expect("Failed to create golden file");

  golden_file
    .write_all(&output.stdout)
    .expect("Failed to write to golden file");
}
