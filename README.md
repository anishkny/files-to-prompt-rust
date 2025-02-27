# files-to-prompt

[![CI](https://github.com/anishkny/files-to-prompt-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/anishkny/files-to-prompt-rust/actions/workflows/ci.yml)
[![Dependabot](https://img.shields.io/badge/dependabot-enabled-brightgreen.svg)](https://github.com/anishkny/files-to-prompt-rust/network/dependencies)
[![Crates.io Version](https://img.shields.io/crates/v/files-to-prompt?color=brightgreen)](https://crates.io/crates/files-to-prompt)

`files-to-prompt` is a command-line tool that recursively reads all files in a specified directory (respecting `.gitignore`) and concatenates their contents into a structured format, making it easy to use as a prompt for Large Language Models (LLMs).

This tool is particularly useful when preparing large codebases or documentation as a prompt for an LLM, allowing users to efficiently gather and format multiple files into a structured input.

## Installation

Install via `cargo`:

```sh
cargo install files-to-prompt
```

Pre-compiled binaries for direct download... coming soon!

## Usage

```
files-to-prompt [OPTIONS] [PATHS]...
```

### Arguments

- `[PATHS]...` Directories or files to process. Defaults to the current directory (`.`).

### Options

- `--include-hidden` Include hidden files in the output.
- `--ignore <IGNORE_PATTERNS>` Glob patterns of files or directories to ignore.
- `--ignore-gitignore` Ignore `.gitignore` files when scanning directories.
- `-n, --line-numbers` Include line numbers in the output.
- `-c, --cxml` Output in an XML-like format suitable for Claude's long context window.
- `-m, --markdown` Output Markdown fenced code blocks.
- `-o, --output <OUTPUT>` Output file (default: stdout) or location (for `-r`, default: current directory).
- `-j, --json` Output JSON compatible with CodeSandbox API/CLI.
- `-r, --reverse` Reverse operation. Reads files from stdin and writes to disk. Requires `-c/--cxml`.
- `-h, --help` Print help information.
- `-V, --version` Print version information.

## Examples

### Concatenate all files recursively in the current directory into a single Markdown prompt

````
files-to-prompt --markdown

./sample.cpp
```cpp
#include <iostream>
int main() {
    std::cout << "Hello, C++!" << std::endl;
    return 0;
}
```
./sample.java
```java
public class Sample {
    public static void main(String[] args) {
        System.out.println("Hello, Java!");
    }
}
```
...
````

### Output in XML-like format for Claude to file

```sh
files-to-prompt --cxml --output output.xml

cat output.xml
<documents>
<document path="./file1.txt">
Contents of file1.txt
</document>
<document path="./file2.txt">
Contents of file2.txt
</document>
<document path="./folder/file3.txt">
Contents of file3.txt
</document>
</documents>
```

### Reverse operation: read from stdin and write files to disk

```sh
echo '<documents><document path="file1.txt">Contents of file1.txt</document><document path="file2.txt">Contents of file2.txt</document><document path="folder/file3.txt">Contents of file3.txt</document></documents>' \
  | files-to-prompt --reverse --cxml
```

### Concatenate files, including hidden ones, while ignoring specific patterns

```
files-to-prompt --include-hidden --ignore "\*.log"
```

### More examples

For more examples, see [`tests/cli-tests.rs`](tests/cli-tests.rs).

## Credit

This project is a Rust port of the original [files-to-prompt](https://github.com/simonw/files-to-prompt) written in Python by [Simon Willison](https://github.com/simonw).

## License

MIT License
