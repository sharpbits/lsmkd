use clap::Parser;
use regex::Regex;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

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
}

#[derive(Debug, Clone)]
struct Heading {
    level: usize,
    text: String,
    line_number: usize,
}

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
    let paths = if args.files.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.files
    };

    let mut first = true;
    for path in paths {
        // Add blank line between different top-level paths
        if !first {
            println!();
        }
        first = false;

        let mut markdown_files = Vec::new();
        collect_markdown_files(
            &path,
            &mut markdown_files,
            args.non_recursive,
            args.all,
        )?;

        // Sort files to ensure consistent ordering
        markdown_files.sort();

        // Extract headings for all files
        let mut file_data = Vec::new();
        for md_file in markdown_files {
            let headings = extract_headings(&md_file, args.min_toc_depth, args.max_toc_depth)?;
            let line_count = count_lines(&md_file)?;
            file_data.push((md_file, headings, line_count));
        }

        // Print the tree structure
        print_tree(&path, &file_data);
    }

    Ok(())
}

fn collect_markdown_files(
    path: &Path,
    files: &mut Vec<PathBuf>,
    non_recursive: bool,
    all: bool,
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
                collect_markdown_files(&entry_path, files, non_recursive, all)?;
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
        "node_modules",
        ".git",
        ".svn",
        ".hg",
        "target",
        "build",
        "dist",
        ".cache",
        "__pycache__",
        ".venv",
        "venv",
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

fn extract_headings(
    md_file: &PathBuf,
    min_depth: usize,
    max_depth: usize,
) -> io::Result<Vec<Heading>> {
    let file = fs::File::open(md_file)?;
    let reader = io::BufReader::new(file);

    let heading_re = Regex::new(r"^(#{1,6})\s+(.+)$").unwrap();
    let mut headings = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        let line_number = line_num + 1; // Line numbers start at 1

        if let Some(captures) = heading_re.captures(&line) {
            let level = captures.get(1).unwrap().as_str().len();
            let text = captures.get(2).unwrap().as_str().trim().to_string();

            if level >= min_depth && level <= max_depth {
                headings.push(Heading {
                    level,
                    text,
                    line_number,
                });
            }
        }
    }

    Ok(headings)
}

fn print_tree(root_path: &Path, file_data: &[(PathBuf, Vec<Heading>, usize)]) {
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
        let (path, headings, line_count) = &file_data[0];
        println!("├── {}:{}", path.display(), line_count);

        if headings.is_empty() {
            println!("    └── (no headings found)");
        } else {
            print_headings(headings, "    ");
        }
        return;
    }

    // Group files by their directory relative to base
    let mut dir_map: BTreeMap<PathBuf, Vec<&(PathBuf, Vec<Heading>, usize)>> = BTreeMap::new();

    for item in file_data {
        let file_dir = item.0.parent().unwrap_or_else(|| Path::new("."));
        let rel_dir = if file_dir == base_dir {
            PathBuf::from(".")
        } else {
            file_dir.strip_prefix(base_dir).unwrap_or(file_dir).to_path_buf()
        };
        dir_map.entry(rel_dir).or_insert_with(Vec::new).push(item);
    }

    // Separate current dir files from subdirectories
    let current_files = dir_map.remove(Path::new(".")).unwrap_or_default();
    let mut subdirs: Vec<_> = dir_map.into_iter().collect();
    subdirs.sort_by(|a, b| a.0.cmp(&b.0));

    // Count total items (files + subdirs) to determine last item
    let total_items = current_files.len() + subdirs.len();
    let mut item_idx = 0;

    // Print files in current directory first
    for (_file_idx, (path, headings, line_count)) in current_files.iter().enumerate() {
        item_idx += 1;
        let file_name = path.file_name().unwrap().to_string_lossy();

        println!("├── {}:{}", file_name, line_count);

        // Always use │ continuation for files with headings in the root directory
        print_headings(headings, "│   ");
    }

    // Print subdirectories and their files
    for (subdir, files) in subdirs.iter() {
        item_idx += 1;
        let is_last_subdir = item_idx == total_items;

        println!("├── {}/", subdir.display());

        // Print files in this subdirectory
        for (file_idx, (path, headings, line_count)) in files.iter().enumerate() {
            let is_last_file = file_idx == files.len() - 1;
            let file_name = path.file_name().unwrap().to_string_lossy();

            let file_prefix = if is_last_subdir && is_last_file {
                "    └── "
            } else if is_last_file {
                "│   └── "
            } else if is_last_subdir {
                "    ├── "
            } else {
                "│   ├── "
            };

            println!("{}{}:{}", file_prefix, file_name, line_count);

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

            print_headings(headings, heading_base);
        }
    }
}

fn print_headings(headings: &[Heading], base_prefix: &str) {
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
        let tree_char = if heading.level == 1 && level_1_count == 1 {
            "└── "
        } else if is_last_at_this_level {
            "└── "
        } else {
            "├── "
        };

        println!("{}{}{} [L{}]", heading_prefix, tree_char, heading.text, heading.line_number);
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
