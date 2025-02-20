use goldenfile::Mint;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use test_case::test_case;

mod helpers;
use helpers::rename_gitignore_files;

#[test_case(&["tests/inputs/01_basic"], "01_basic.golden.txt"; "01_basic")]
#[test_case(
  &["tests/inputs/02_multiple_folders/f1", "tests/inputs/02_multiple_folders/f2"],
  "02_multiple_folders.golden.txt"; "02_multiple_folders"
)]
#[test_case(&["tests/inputs/03_gitignore"], "03_gitignore.golden.txt"; "03_gitignore")]
fn test_files_to_prompt(input: &[&str], golden_filename: &str) {
  // Rename gitignore files to .gitignore if they exist
  let _gitignore_renamers = rename_gitignore_files(input);

  // Run the CLI tool
  println!("Running CLI tool with input: {:?}", input);
  let output = Command::new("cargo")
    .args(["run", "--"].iter().chain(input.iter()))
    .output()
    .expect("Failed to execute command");

  // Ensure the command was successful
  assert!(output.status.success());

  // Print the output
  println!("stdout: {}", String::from_utf8_lossy(&output.stdout));

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
