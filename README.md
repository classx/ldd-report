# ldd-report

A utility for generating dependency reports of ELF libraries and binaries.

## Overview

`ldd-report` is a command-line tool that analyzes ELF files in a directory and reports their dynamic library dependencies. It provides functionality similar to the `ldd` command but with more structured output and additional features.

## Features

- Recursively scan directories for ELF libraries and binaries
- Identify dynamic library dependencies for each ELF file
- Map dependencies to their actual paths on the system
- Show missing dependencies that could cause runtime issues

## Installation

### From Source

```bash
git clone https://github.com/username/ldd-report.git
cd ldd-report
cargo build --release
```

The compiled binary will be available at `target/release/ldd-report`.

## Usage

```
ldd-report [OPTIONS] --dir <DIR>
```

### Options

```
  -d, --dir <DIR>      Libraries path (directory to scan for ELF files)
  -s, --silent         Return exit code only (no output)
  -h, --help           Print help
  -V, --version        Print version
```

### Example

```bash
# Analyze all libraries in a directory
ldd-report --dir /path/to/libraries

# Analyze but only return exit code
ldd-report --dir /path/to/libraries --silent
```

## Output Format

For each ELF file found, the tool outputs:
- The full path of the file
- A list of required libraries and their resolved system paths
- Missing dependencies are marked as "NOT FOUND"

Example output:
```
/path/to/libraries/libexample.so
  libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6
  libm.so.6 => /lib/x86_64-linux-gnu/libm.so.6
  libcustom.so => NOT FOUND
```

## Requirements

- Linux operating system
- `ldconfig` available in the system path
- Rust 1.56 or later (for building from source)

## Dependencies

- `clap`: Command-line argument parsing
- `goblin`: ELF file parsing

## License

[MIT License](LICENSE)