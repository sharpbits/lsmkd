use clap::{Parser, ValueEnum};
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "lsmkd")]
#[command(about = "List and index markdown files with table-of-contents and line numbers", long_about = None)]
#[command(version)]
struct Args {
    /// One or more files or directories to list, default: .
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Disable recursive directory traversal
    #[arg(short = 'x', long = "non-recursive")]
    non_recursive: bool,

    /// Minimum markdown heading level, default: 1
    #[arg(short = 'n', long = "min-toc-depth", default_value_t = 1)]
    min_toc_depth: usize,

    /// Maximum markdown heading level, default: 2
    #[arg(short = 'm', long = "max-toc-depth", default_value_t = 2)]
    max_toc_depth: usize,

    /// Traverse all paths, including commonly ignored such as node_modules/
    #[arg(short = 'a', long = "all")]
    all: bool,

    /// Maximum directory depth for traversal, unlimited by default
    #[arg(short = 'd', long = "depth")]
    depth: Option<usize>,

    /// Output format
    #[arg(short = 'o', long = "output", default_value = "text")]
    output: OutputFormat,

    /// Verbose output with statistics
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Include token estimates in output (using 3.5 chars per token)
    #[arg(short = 't', long = "tokens")]
    tokens: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
    Yaml,
}

#[derive(Debug, Clone, Serialize)]
struct Heading {
    level: usize,
    text: String,
    line_number: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    tokens: Option<usize>,
}

#[derive(Debug, Serialize)]
struct FileInfo {
    path: String,
    size: u64,
    lines: usize,
    headings: Vec<Heading>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tokens: Option<usize>,
}

#[derive(Debug, Default)]
struct ScanStatistics {
    files_scanned: usize,
    total_lines: usize,
    total_bytes: u64,
    total_tokens: usize,
}

type FileDataItem = (PathBuf, Vec<Heading>, usize, Option<usize>);

fn main() {
    let args = Args::parse();

    match run(args) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    let paths = if args.files.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.files
    };

    let mut all_files: Vec<FileInfo> = Vec::new();
    let mut stats = ScanStatistics::default();

    for path in &paths {
        let mut markdown_files = Vec::new();
        collect_markdown_files(
            path,
            &mut markdown_files,
            args.non_recursive,
            args.all,
            args.depth,
            0,
        )?;

        markdown_files.sort();

        for md_file in markdown_files {
            let headings = extract_headings(&md_file, args.min_toc_depth, args.max_toc_depth, args.tokens)?;
            let line_count = count_lines(&md_file)?;
            let size = fs::metadata(&md_file)?.len();

            let file_tokens = if args.tokens {
                let content = fs::read_to_string(&md_file)?;
                let tokens = estimate_tokens(&content);
                stats.total_tokens += tokens;
                Some(tokens)
            } else {
                None
            };

            stats.files_scanned += 1;
            stats.total_lines += line_count;
            stats.total_bytes += size;

            all_files.push(FileInfo {
                path: md_file.display().to_string(),
                size,
                lines: line_count,
                headings,
                tokens: file_tokens,
            });
        }
    }

    match args.output {
        OutputFormat::Json => println!("{}", serde_json::to_string(&all_files)?),
        OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&all_files)?),
        OutputFormat::Text => {
            let mut first = true;
            for path in &paths {
                if !first {
                    println!();
                }
                first = false;

                let path_str = path.display().to_string();
                let file_data: Vec<_> = all_files
                    .iter()
                    .filter(|f| f.path.starts_with(&path_str))
                    .map(|f| (PathBuf::from(&f.path), f.headings.clone(), f.lines, f.tokens))
                    .collect();

                print_tree(path, &file_data, args.tokens);
            }
        }
    }

    let elapsed = start_time.elapsed();

    if args.verbose {
        println!();
        println!("Statistics:");
        println!("  Files scanned: {}", stats.files_scanned);
        println!("  Total lines: {}", stats.total_lines);
        println!("  Total bytes: {}", format_bytes(stats.total_bytes));
        if args.tokens {
            println!("  Total tokens: {}", stats.total_tokens);
        }
        println!("  Time taken: {:.3}s", elapsed.as_secs_f64());
    }

    Ok(())
}

fn collect_markdown_files(
    path: &Path,
    files: &mut Vec<PathBuf>,
    non_recursive: bool,
    all: bool,
    max_depth: Option<usize>,
    current_depth: usize,
) -> io::Result<()> {
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Path does not exist: {}", path.display()),
        ));
    }

    if path.is_file() {
        if is_markdown_file(path) {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }

    if path.is_dir() {
        let entries = fs::read_dir(path)?;
        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();

            // Skip commonly ignored directories unless --all is specified
            if !all && should_ignore(&entry_path) {
                continue;
            }

            if entry_path.is_file() && is_markdown_file(&entry_path) {
                files.push(entry_path);
            } else if entry_path.is_dir() && !non_recursive {
                // Check if we've exceeded the maximum depth before recursing
                if let Some(max) = max_depth
                    && current_depth >= max {
                        continue;
                    }
                collect_markdown_files(&entry_path, files, non_recursive, all, max_depth, current_depth + 1)?;
            }
        }
    }

    Ok(())
}

fn is_markdown_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        ext_str == "md" || ext_str == "markdown"
    } else {
        false
    }
}

fn should_ignore(path: &Path) -> bool {
    let ignored_dirs = [
        // Version control
        ".git",
        ".svn",
        ".hg",
        // Compiled/Build outputs
        "target",
        "build",
        "dist",
        // Caches
        ".cache",
        "__pycache__",
        // Python environments
        ".venv",
        "venv",
        // Dependencies
        "node_modules",
        "vendor",
        "Packages",
        "Pods",
        "bower_components",
        // Generated/Built documentation
        "_site",
        // Test fixtures/data
        "fixtures",
        "__fixtures__",
        "test-data",
    ];

    if let Some(name) = path.file_name() {
        let name_str = name.to_string_lossy();
        for ignored in &ignored_dirs {
            if name_str == *ignored {
                return true;
            }
        }
        // Ignore hidden directories/files starting with .
        if name_str.starts_with('.') {
            return true;
        }
    }

    false
}

fn count_lines(file_path: &PathBuf) -> io::Result<usize> {
    let file = fs::File::open(file_path)?;
    let reader = io::BufReader::new(file);
    Ok(reader.lines().count())
}

fn estimate_tokens(text: &str) -> usize {
    let char_count = text.chars().count();
    ((char_count as f64) / 3.5).ceil() as usize
}

fn extract_headings(
    md_file: &PathBuf,
    min_depth: usize,
    max_depth: usize,
    calculate_tokens: bool,
) -> io::Result<Vec<Heading>> {
    let file = fs::File::open(md_file)?;
    let reader = io::BufReader::new(file);

    let heading_re = Regex::new(r"^(#{1,6})\s+(.+)$").unwrap();
    let mut headings = Vec::new();
    let mut all_lines: Vec<String> = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        let line_number = line_num + 1; // Line numbers start at 1

        if calculate_tokens {
            all_lines.push(line.clone());
        }

        if let Some(captures) = heading_re.captures(&line) {
            let level = captures.get(1).unwrap().as_str().len();
            let text = captures.get(2).unwrap().as_str().trim().to_string();

            if level >= min_depth && level <= max_depth {
                headings.push(Heading {
                    level,
                    text,
                    line_number,
                    tokens: None,
                });
            }
        }
    }

    // Calculate tokens for each section
    if calculate_tokens && !headings.is_empty() {
        for i in 0..headings.len() {
            let start_line = headings[i].line_number - 1; // Convert to 0-indexed
            let end_line = if i + 1 < headings.len() {
                // Find the next heading at the same or higher level (lower number)
                let mut next_idx = i + 1;
                while next_idx < headings.len() && headings[next_idx].level > headings[i].level {
                    next_idx += 1;
                }
                if next_idx < headings.len() {
                    headings[next_idx].line_number - 1
                } else {
                    all_lines.len()
                }
            } else {
                all_lines.len()
            };

            let section_text: String = all_lines[start_line..end_line].join("\n");
            headings[i].tokens = Some(estimate_tokens(&section_text));
        }
    }

    Ok(headings)
}

fn print_tree(root_path: &Path, file_data: &[FileDataItem], show_tokens: bool) {
    use std::collections::BTreeMap;

    if file_data.is_empty() {
        return;
    }

    // Determine if we're processing a single file or a directory
    let is_single_file = root_path.is_file();

    // Determine the base directory
    let base_dir = if root_path.is_dir() {
        root_path
    } else if root_path.is_file() {
        root_path.parent().unwrap_or_else(|| Path::new("."))
    } else {
        Path::new(".")
    };

    // Print root directory name if it's a directory
    if root_path.is_dir() {
        println!("{}/", root_path.display());
    }

    // If it's a single file, just print it with full path
    if is_single_file && file_data.len() == 1 {
        let (path, headings, line_count, file_tokens) = &file_data[0];
        let file_size = get_file_size(path);
        if show_tokens {
            if let Some(tokens) = file_tokens {
                println!("├── {} {{size: {}, lines: {}, tokens: {}}}", path.display(), file_size, line_count, tokens);
            } else {
                println!("├── {} {{size: {}, lines: {}}}", path.display(), file_size, line_count);
            }
        } else {
            println!("├── {} {{size: {}, lines: {}}}", path.display(), file_size, line_count);
        }

        if headings.is_empty() {
            println!("    └── (no headings found)");
        } else {
            print_headings(headings, "    ", show_tokens);
        }
        return;
    }

    // Group files by their directory relative to base
    let mut dir_map: BTreeMap<PathBuf, Vec<&FileDataItem>> = BTreeMap::new();

    for item in file_data {
        let file_dir = item.0.parent().unwrap_or_else(|| Path::new("."));
        let rel_dir = if file_dir == base_dir {
            PathBuf::from(".")
        } else {
            file_dir.strip_prefix(base_dir).unwrap_or(file_dir).to_path_buf()
        };
        dir_map.entry(rel_dir).or_default().push(item);
    }

    // Separate current dir files from subdirectories
    let current_files = dir_map.remove(Path::new(".")).unwrap_or_default();
    let mut subdirs: Vec<_> = dir_map.into_iter().collect();
    subdirs.sort_by(|a, b| a.0.cmp(&b.0));

    // Count total items (files + subdirs) to determine last item
    let total_items = current_files.len() + subdirs.len();
    let mut item_idx = 0;

    // Print files in current directory first
    for (path, headings, line_count, file_tokens) in current_files.iter() {
        item_idx += 1;
        let file_name = path.file_name().unwrap().to_string_lossy();
        let file_size = get_file_size(path);

        if show_tokens {
            if let Some(tokens) = file_tokens {
                println!("├── {} {{size: {}, lines: {}, tokens: {}}}", file_name, file_size, line_count, tokens);
            } else {
                println!("├── {} {{size: {}, lines: {}}}", file_name, file_size, line_count);
            }
        } else {
            println!("├── {} {{size: {}, lines: {}}}", file_name, file_size, line_count);
        }

        // Always use │ continuation for files with headings in the root directory
        print_headings(headings, "│   ", show_tokens);
    }

    // Print subdirectories and their files
    for (subdir, files) in subdirs.iter() {
        item_idx += 1;
        let is_last_subdir = item_idx == total_items;

        println!("├── {}/", subdir.display());

        // Print files in this subdirectory
        for (file_idx, (path, headings, line_count, file_tokens)) in files.iter().enumerate() {
            let is_last_file = file_idx == files.len() - 1;
            let file_name = path.file_name().unwrap().to_string_lossy();
            let file_size = get_file_size(path);

            let file_prefix = if is_last_subdir && is_last_file {
                "    └── "
            } else if is_last_file {
                "│   └── "
            } else if is_last_subdir {
                "    ├── "
            } else {
                "│   ├── "
            };

            if show_tokens {
                if let Some(tokens) = file_tokens {
                    println!("{}{} {{size: {}, lines: {}, tokens: {}}}", file_prefix, file_name, file_size, line_count, tokens);
                } else {
                    println!("{}{} {{size: {}, lines: {}}}", file_prefix, file_name, file_size, line_count);
                }
            } else {
                println!("{}{} {{size: {}, lines: {}}}", file_prefix, file_name, file_size, line_count);
            }

            // Determine heading prefix based on context
            let heading_base = if is_last_subdir && is_last_file {
                "        "
            } else if is_last_file {
                "│       "
            } else if is_last_subdir {
                "    │   "
            } else {
                "│   │   "
            };

            print_headings(headings, heading_base, show_tokens);
        }
    }
}

fn get_file_size(path: &PathBuf) -> String {
    if let Ok(metadata) = fs::metadata(path) {
        let size = metadata.len();
        if size < 1024 {
            format!("{}B", size)
        } else if size < 1024 * 1024 {
            format!("{}k", size / 1024)
        } else {
            format!("{}M", size / (1024 * 1024))
        }
    } else {
        "?".to_string()
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} bytes", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn print_headings(headings: &[Heading], base_prefix: &str, show_tokens: bool) {
    if headings.is_empty() {
        return;
    }

    // Check if there's only one level-1 heading or multiple
    let level_1_count = headings.iter().filter(|h| h.level == 1).count();

    for (h_idx, heading) in headings.iter().enumerate() {
        let is_last_at_this_level = !headings.iter().skip(h_idx + 1).any(|h| h.level <= heading.level);

        // Build indentation for heading level
        let mut heading_prefix = String::from(base_prefix);

        if heading.level > 1 {
            for level in 1..heading.level {
                let has_continuation = headings.iter().skip(h_idx + 1).any(|h| h.level <= level);
                heading_prefix.push_str(if has_continuation { "│   " } else { "    " });
            }
        }

        // For level-1 headings, use └── if it's the only one or the last one
        let tree_char = if (heading.level == 1 && level_1_count == 1) || is_last_at_this_level {
            "└── "
        } else {
            "├── "
        };

        if show_tokens {
            if let Some(tokens) = heading.tokens {
                println!("{}{}{} {{line: {}, tokens: {}}}",
                         heading_prefix, tree_char, heading.text, heading.line_number, tokens);
            } else {
                println!("{}{}{} {{line: {}}}",
                         heading_prefix, tree_char, heading.text, heading.line_number);
            }
        } else {
            println!("{}{}{} {{line: {}}}",
                     heading_prefix, tree_char, heading.text, heading.line_number);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_markdown_file() {
        assert!(is_markdown_file(Path::new("test.md")));
        assert!(is_markdown_file(Path::new("test.markdown")));
        assert!(is_markdown_file(Path::new("TEST.MD")));
        assert!(!is_markdown_file(Path::new("test.txt")));
        assert!(!is_markdown_file(Path::new("test")));
    }

    #[test]
    fn test_should_ignore() {
        assert!(should_ignore(Path::new("node_modules")));
        assert!(should_ignore(Path::new(".git")));
        assert!(should_ignore(Path::new("target")));
        assert!(should_ignore(Path::new(".hidden")));
        assert!(!should_ignore(Path::new("src")));
        assert!(!should_ignore(Path::new("docs")));
    }
}
