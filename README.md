# files-to-prompt

[![CI](https://github.com/anishkny/files-to-prompt-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/anishkny/files-to-prompt-rust/actions/workflows/ci.yml)
[![Dependabot](https://img.shields.io/badge/dependabot-enabled-brightgreen.svg)](https://github.com/anishkny/files-to-prompt-rust/network/dependencies)
[![Crates.io Version](https://img.shields.io/crates/v/files-to-prompt?color=brightgreen)](https://crates.io/crates/files-to-prompt)

`files-to-prompt` is a command-line tool that recursively reads all files in a specified directory (respecting `.gitignore`) and concatenates their contents into a structured format, making it easy to use as a prompt for Large Language Models (LLMs).

## Features

- Recursively scans directories and reads all files
- Outputs file paths along with their contents
- Respects `.gitignore`
- Ignores hidden files be default
- Sorts files by path for consistency

## Installation

To use `files-to-prompt`, first ensure you have Rust installed. Then, build the project:

```sh
cargo build --release
```

Or install it directly using:

```sh
cargo install --path .
```

## Usage

Run the tool by providing one or more directory paths:

```sh
files-to-prompt <path1> [path2] ...
```

### Example

```sh
files-to-prompt ./my_project
```

This will output:

```
./my_project/file1.txt
----
<contents of file1.txt>
----

./my_project/file2.rs
----
<contents of file2.rs>
----
```

## Error Handling

- If a file cannot be read, an error message is printed.
- If no path is provided, the program exits with an error message.

## Use Case

This tool is particularly useful when preparing large codebases or documentation as a prompt for an LLM, allowing users to efficiently gather and format multiple files into a structured input.

## Credit

This project is a Rust port of the original [files-to-prompt](https://github.com/simonw/files-to-prompt) written in Python by [Simon Willison](https://github.com/simonw).

## License

MIT License
