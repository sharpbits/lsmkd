# lsmkd - ls for markdown docs

This command utility recursively finds all markdown files in the given directories and lists them along with each file's table of contents and respective line-number ranges. This tool is primarily intended for use by AI coding agents to rapidly identify relevant documentation and understand which lines are important to read. By guiding coding agents to only read portions of large markdown files, we save valuable context for the work that matters.

This utility is explicitly only read-only, it does not modify markdown files to insert table-of-contents. If that's what you're looking for, check out [mktoc](https://github.com/KevinGimbel/mktoc).

## Usage

```sh
$ lsmkd -h
List and index markdown files with table-of-contents and line numbers

Usage: lsmkd [OPTION]... [FILE]...

Arguments:
 [FILE]  [One or more files or directories to list, default: .]

Options:
  -x, --non-recursive       Disable recursive directory traversal
  -n, --min-toc-depth       Minimum markdown heading level, default: 1
  -m, --max-toc-depth       Maximum markdown heading level, default: 2
  -a, --all                 Traverse all paths, including commonly ignored such as node_modules/
  -d, --depth               Maximum directory depth for traversal, unlimited by default
  -o, --output <FORMAT>     Output format: text, json, yaml (default: text)
  -h, --help                Show help and usage
  --version                 Show version information

Output:
$ lsmkd docs/
docs/
├── architecture.md {size: 15k, lines: 50}
│   └── Platform Architecture {line: 1}      # Section starts on line 1
│       ├── Overview {line: 3}               # Section starts on line 3
|       └── Core Components {line: 7}
├── prd.md {size: 23k, lines: 150}
│   └── Product Requirements Document {line: 1}
│       ├── Executive Summary {line: 3}
|       └── Business Objectives {line: 15}
├── epics/
│   ├── epic1.md {size: 8k, lines: 30}
|   |   └── Epic 1: Project Scaffold {line: 1}
|   └── epic2.md {size: 9k, lines: 34}
...
```

## Installation

`lsmkd` can be installed using [Cargo](https://rust-lang.org/tools/install/), the Rust package manager.

```sh
$ cargo install lsmkd
```

**Update**
```sh
$ cargo install --force lsmkd
```

### Binary

Binaries can be downloaded from the [release page](https://github.com/sharpbits/lsmkd/releases/latest).
