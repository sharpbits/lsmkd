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

    let mut markdown_files = Vec::new();

    for path in paths {
        collect_markdown_files(
            &path,
            &mut markdown_files,
            args.non_recursive,
            args.all,
        )?;
    }

    for md_file in markdown_files {
        let headings = extract_headings(&md_file, args.min_toc_depth, args.max_toc_depth)?;
        print_markdown_file(&md_file, &headings);
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

fn print_markdown_file(path: &PathBuf, headings: &[Heading]) {
    println!("{}", path.display());

    if headings.is_empty() {
        println!("  (no headings found)");
    } else {
        for heading in headings {
            let indent = "  ".repeat(heading.level - 1);
            println!("{}{}:{} {}", indent, path.display(), heading.line_number, heading.text);
        }
    }

    println!();
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
