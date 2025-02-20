use std::fs;
use std::path::Path;

pub struct GitignoreRenamer {
  original: String,
  renamed: String,
}

impl Drop for GitignoreRenamer {
  fn drop(&mut self) {
    let original_path = Path::new(&self.renamed);
    let renamed_path = Path::new(&self.original);

    if original_path.exists() {
      if let Err(err) = fs::rename(original_path, renamed_path) {
        eprintln!(
          "Failed to rename {:?} back to {:?}: {}",
          original_path, renamed_path, err
        );
      } else {
        println!("Restored {:?} to {:?}", original_path, renamed_path);
      }
    }
  }
}

pub fn rename_gitignore_files(input: &[&str]) -> Vec<GitignoreRenamer> {
  let mut renamers = Vec::new();

  for folder in input {
    let gitignore_path = Path::new(folder).join("gitignore");
    let dot_gitignore_path = Path::new(folder).join(".gitignore");

    if gitignore_path.exists() {
      fs::rename(&gitignore_path, &dot_gitignore_path)
        .expect("Failed to rename gitignore to .gitignore");
      println!("Renamed {:?} to {:?}", gitignore_path, dot_gitignore_path);

      // Store in the vector for automatic restoration
      renamers.push(GitignoreRenamer {
        original: gitignore_path.to_string_lossy().into_owned(),
        renamed: dot_gitignore_path.to_string_lossy().into_owned(),
      });
    }
  }

  renamers
}
