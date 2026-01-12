# lsmkd - ls for markdown docs

This command utility recursively finds all markdown files in the given directories and lists them along with each file's table of contents and respective line-number ranges. 

This tool is primarily intended for use by AI coding agents to rapidly identify relevant documentation and understand which lines are important to read. By guiding coding agents to only read portions of large markdown files, we save valuable context for the work that matters. If you're struggling with context windows while using BMAD method, you've found the right tool to help.

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
  -t, --tokens              Include token estimates (using 3.5 chars per token)
  -v, --verbose             Output statistics (files, lines, bytes scanned and time taken)
  -h, --help                Show help and usage
  --version                 Show version information

Output:
$ lsmkd docs/
docs/
├── architecture.md {size: 15k, lines: 50}
│   └── Platform Architecture {line: 1}
│       ├── Overview {line: 3}
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

With tokens:
$ lsmkd docs/ -t
docs/
├── architecture.md {size: 15k, lines: 50, tokens: 4285}
│   └── Platform Architecture {line: 1, tokens: 4285}
│       ├── Overview {line: 3, tokens: 857}
|       └── Core Components {line: 7, tokens: 3428}
├── prd.md {size: 23k, lines: 150, tokens: 6571}
│   └── Product Requirements Document {line: 1, tokens: 6571}
│       ├── Executive Summary {line: 3, tokens: 1314}
|       └── Business Objectives {line: 15, tokens: 5257}
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

## AGENTS.md, CLAUDE.md, copilot-instructions.md

Add the following to your coding agent instructions:

```
CRITICAL: LLM context is limited! To conserve context when searching local files for documentation, use the `lsmkd <path(s)>` command instead of `ls` or `find` to list and index all markdown files with heading line numbers then READ ONLY RELEVANT sections of relevant files for the requested task.
```

## Benchmarks

On an M2 Pro Macbook this utiltity indexes the complete github.com docs repo in 1.46s:

```
$ git clone https://github.com/github/docs.git --depth=1
$ lsmkd docs -v
...
Statistics:
  Files scanned: 6889
  Total lines: 307589
  Total bytes: 18.53 MB
  Time taken: 1.460s
```

With token estimation:

```
$ lsmkd docs -t -v
...
Statistics:
  Files scanned: 6889
  Total lines: 307589
  Total bytes: 18.53 MB
  Total tokens: 5294540
  Time taken: 1.520s
```
