use goldenfile::Mint;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use std::{fs::read_to_string, process::Command};
use test_case::test_case;

mod helpers;
use helpers::{get_temp_dir, get_temp_output_file_path, rename_gitignore_files};

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
#[test_case(&["tests/inputs/01_basic", "tests/inputs/01_basic/file1.txt"], "09_dedupe.golden.txt"; "09_dedupe")]
#[test_case(&["tests/inputs/01_basic", "--cxml"], "10_cxml.golden.txt"; "10_cxml")]
#[test_case(&["tests/inputs/11_markdown", "--markdown"], "11_markdown.golden.txt"; "11_markdown")]
#[test_case(&["tests/inputs/01_basic", "--output", get_temp_output_file_path()], "12_output.golden.txt"; "12_output")]
#[test_case(&["tests/inputs/11_markdown", "--json"], "13_json.golden.txt"; "13_json")]
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

  // Handle the case where the output is written to a temporary file
  if golden_filename == "12_output.golden.txt" {
    golden_file
      .write_all(
        read_to_string(get_temp_output_file_path())
          .unwrap()
          .as_bytes(),
      )
      .expect("Failed to write to golden file");
  } else {
    golden_file
      .write_all(&output.stdout)
      .expect("Failed to write to golden file");
  }
}

#[test]
fn test_files_to_prompt_reverse() {
  let temp_dir = get_temp_dir();
  println!("Temp dir: {:?}", temp_dir);

  // Read stdin data from the file "tests/inputs/reverse.txt"
  let stdin_data = fs::read_to_string("tests/inputs/reverse.txt").expect("Failed to read file");

  // Define the arguments as an array of strings
  let args = vec![
    "--reverse".to_string(),
    "--cxml".to_string(),
    "--output".to_string(),
    temp_dir.to_string(),
  ];

  // Run the command with the specified arguments
  let mut command: Command = Command::new(TARGET);
  command.args(args);

  // Set up the stdin pipe to send data to the command's stdin
  let mut child = command
    .stdin(Stdio::piped()) // Set up a piped stdin
    .spawn()
    .expect("Failed to start the command");

  // Write the file content to stdin
  if let Some(mut stdin) = child.stdin.take() {
    stdin
      .write_all(stdin_data.as_bytes())
      .expect("Failed to write to stdin");
  }

  // Wait for the command to finish and collect the output
  let output = child.wait_with_output().expect("Failed to wait on command");
  assert!(output.status.success());

  // Verify that each file exists and has the correct contents
  let expected_files = vec![
    ("file1.txt", "Contents of file1.txt\n"),
    ("file2.txt", "Contents of file2.txt\n"),
    ("folder/file3.txt", "Contents of file3.txt\n"),
  ];

  for (file_name, expected_content) in expected_files {
    let file_path = Path::new(&temp_dir).join(file_name);
    let file_contents = fs::read_to_string(file_path.clone()).unwrap();
    println!("File {:?} contents: {:?}", file_path, file_contents);
    assert_eq!(file_contents, expected_content);
  }
}
