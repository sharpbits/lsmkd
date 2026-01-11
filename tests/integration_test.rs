use assert_cmd::cargo;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test that lsmkd correctly processes the architecture.md sample
#[test]
fn test_architecture_doc_sample() {
    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("tests/docs/architecture.md");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("tests/docs/architecture.md"))
        .stdout(predicate::str::contains("Zephyr Platform Architecture {line: 1}"))
        .stdout(predicate::str::contains("Overview {line: 3}"))
        .stdout(predicate::str::contains("Core Components {line: 7}"))
        .stdout(predicate::str::contains("Data Flow {line: 30}"))
        .stdout(predicate::str::contains("Deployment Model {line: 40}"))
        .stdout(predicate::str::contains("Security Architecture {line: 44}"));
}

/// Test that lsmkd correctly processes the prd.md sample
#[test]
fn test_prd_doc_sample() {
    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("tests/docs/prd.md");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("tests/docs/prd.md"))
        .stdout(predicate::str::contains("Zephyr Platform - Product Requirements Document {line: 1}"))
        .stdout(predicate::str::contains("Executive Summary {line: 7}"))
        .stdout(predicate::str::contains("Business Objectives {line: 11}"))
        .stdout(predicate::str::contains("Target Users {line: 18}"))
        .stdout(predicate::str::contains("Core Features {line: 24}"))
        .stdout(predicate::str::contains("Non-Functional Requirements {line: 66}"))
        .stdout(predicate::str::contains("Deployment Constraints {line: 77}"))
        .stdout(predicate::str::contains("Success Metrics {line: 83}"))
        .stdout(predicate::str::contains("Timeline {line: 90}"))
        .stdout(predicate::str::contains("Assumptions {line: 97}"))
        .stdout(predicate::str::contains("Open Questions {line: 104}"));
}

/// Test that lsmkd processes entire docs directory recursively
#[test]
fn test_recursive_directory_processing() {
    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("tests/docs/");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("architecture.md"))
        .stdout(predicate::str::contains("prd.md"))
        .stdout(predicate::str::contains("Zephyr Platform Architecture"))
        .stdout(predicate::str::contains("Product Requirements Document"));
}

/// Test non-recursive mode with -x flag
#[test]
fn test_non_recursive_flag() {
    let temp_dir = TempDir::new().unwrap();
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();

    fs::write(temp_dir.path().join("top.md"), "# Top Level\n").unwrap();
    fs::write(subdir.join("nested.md"), "# Nested File\n").unwrap();

    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("-x").arg(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("top.md"))
        .stdout(predicate::str::contains("Top Level"))
        .stdout(predicate::str::contains("nested.md").not());
}

/// Test min-toc-depth flag filters out low-level headings
#[test]
fn test_min_toc_depth() {
    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("-n").arg("2").arg("tests/docs/prd.md");

    // Should only show h2 and deeper headings (not h1)
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Executive Summary"))
        .stdout(predicate::str::contains("Product Requirements Document").not());
}

/// Test max-toc-depth flag filters out deeper headings
#[test]
fn test_max_toc_depth() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("depth.md");

    fs::write(
        &test_file,
        "# Level 1\n## Level 2\n### Level 3\n#### Level 4\n",
    )
    .unwrap();

    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("-m").arg("2").arg(&test_file);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Level 1"))
        .stdout(predicate::str::contains("Level 2"))
        .stdout(predicate::str::contains("Level 3").not())
        .stdout(predicate::str::contains("Level 4").not());
}

/// Test combined min and max depth flags
#[test]
fn test_min_max_toc_depth() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("depth.md");

    fs::write(
        &test_file,
        "# Level 1\n## Level 2\n### Level 3\n#### Level 4\n",
    )
    .unwrap();

    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("-n").arg("2").arg("-m").arg("3").arg(&test_file);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Level 1").not())
        .stdout(predicate::str::contains("Level 2"))
        .stdout(predicate::str::contains("Level 3"))
        .stdout(predicate::str::contains("Level 4").not());
}

/// Test that ignored directories are skipped by default
#[test]
fn test_ignored_directories() {
    let temp_dir = TempDir::new().unwrap();
    let node_modules = temp_dir.path().join("node_modules");
    let target = temp_dir.path().join("target");

    fs::create_dir(&node_modules).unwrap();
    fs::create_dir(&target).unwrap();

    fs::write(temp_dir.path().join("normal.md"), "# Normal\n").unwrap();
    fs::write(node_modules.join("ignored.md"), "# Ignored\n").unwrap();
    fs::write(target.join("also_ignored.md"), "# Also Ignored\n").unwrap();

    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("normal.md"))
        .stdout(predicate::str::contains("ignored.md").not())
        .stdout(predicate::str::contains("also_ignored.md").not());
}

/// Test --all flag includes ignored directories
#[test]
fn test_all_flag_includes_ignored() {
    let temp_dir = TempDir::new().unwrap();
    let node_modules = temp_dir.path().join("node_modules");

    fs::create_dir(&node_modules).unwrap();
    fs::write(node_modules.join("included.md"), "# Included\n").unwrap();

    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("-a").arg(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("included.md"))
        .stdout(predicate::str::contains("Included"));
}

/// Test processing multiple file arguments
#[test]
fn test_multiple_file_arguments() {
    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("tests/docs/architecture.md")
        .arg("tests/docs/prd.md");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("architecture.md"))
        .stdout(predicate::str::contains("prd.md"));
}

/// Test default directory is current directory when no args provided
#[test]
fn test_default_current_directory() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("test.md"), "# Test\n").unwrap();

    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.current_dir(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test.md"));
}

/// Test that files without headings show appropriate message
#[test]
fn test_file_without_headings() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("empty.md");

    fs::write(&test_file, "Just some text without any headings.\n").unwrap();

    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg(&test_file);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("empty.md"))
        .stdout(predicate::str::contains("(no headings found)"));
}

/// Test that non-existent path returns error
#[test]
fn test_nonexistent_path_error() {
    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("nonexistent/path/to/file.md");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error"))
        .stderr(predicate::str::contains("does not exist"));
}

/// Test that both .md and .markdown extensions are recognized
#[test]
fn test_markdown_extensions() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(temp_dir.path().join("test.md"), "# MD Extension\n").unwrap();
    fs::write(temp_dir.path().join("test.markdown"), "# Markdown Extension\n").unwrap();
    fs::write(temp_dir.path().join("test.txt"), "# Not Markdown\n").unwrap();

    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test.md"))
        .stdout(predicate::str::contains("test.markdown"))
        .stdout(predicate::str::contains("test.txt").not());
}

/// Test help flag displays usage information
#[test]
fn test_help_flag() {
    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("-h");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List and index markdown files"))
        .stdout(predicate::str::contains("Usage:"));
}

/// Test version flag displays version
#[test]
fn test_version_flag() {
    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

/// Test hidden directories are ignored by default
#[test]
fn test_hidden_directories_ignored() {
    let temp_dir = TempDir::new().unwrap();
    let hidden_dir = temp_dir.path().join(".hidden");

    fs::create_dir(&hidden_dir).unwrap();
    fs::write(temp_dir.path().join("visible.md"), "# Visible\n").unwrap();
    fs::write(hidden_dir.join("hidden.md"), "# Hidden\n").unwrap();

    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("visible.md"))
        .stdout(predicate::str::contains("hidden.md").not());
}

/// Test indentation is correct for nested headings
#[test]
fn test_heading_indentation() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("indent.md");

    fs::write(&test_file, "# H1\n## H2\n").unwrap();

    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg(&test_file);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // H1 shows as "└── H1 {line: 1}" at root level
    // H2 shows as "    └── H2 {line: 2}" with indentation for level 2
    assert!(stdout.contains("H1 {line: 1}"));
    assert!(stdout.contains("H2 {line: 2}"));
    assert!(stdout.contains("└──") || stdout.contains("├──"));
}

/// Test depth limit with --depth flag (depth=0)
#[test]
fn test_depth_zero() {
    let temp_dir = TempDir::new().unwrap();
    let level1 = temp_dir.path().join("level1");

    fs::create_dir(&level1).unwrap();
    fs::write(temp_dir.path().join("root.md"), "# Root Level\n").unwrap();
    fs::write(level1.join("nested.md"), "# Nested File\n").unwrap();

    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("-d").arg("0").arg(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("root.md"))
        .stdout(predicate::str::contains("Root Level"))
        .stdout(predicate::str::contains("nested.md").not());
}

/// Test depth limit with --depth flag (depth=1)
#[test]
fn test_depth_one() {
    let temp_dir = TempDir::new().unwrap();
    let level1 = temp_dir.path().join("level1");
    let level2 = level1.join("level2");

    fs::create_dir_all(&level2).unwrap();
    fs::write(temp_dir.path().join("root.md"), "# Root\n").unwrap();
    fs::write(level1.join("level1.md"), "# Level 1\n").unwrap();
    fs::write(level2.join("level2.md"), "# Level 2\n").unwrap();

    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("-d").arg("1").arg(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("root.md"))
        .stdout(predicate::str::contains("level1.md"))
        .stdout(predicate::str::contains("level2.md").not());
}

/// Test depth limit with --depth flag (depth=2)
#[test]
fn test_depth_two() {
    let temp_dir = TempDir::new().unwrap();
    let level1 = temp_dir.path().join("level1");
    let level2 = level1.join("level2");
    let level3 = level2.join("level3");

    fs::create_dir_all(&level3).unwrap();
    fs::write(temp_dir.path().join("root.md"), "# Root\n").unwrap();
    fs::write(level1.join("level1.md"), "# Level 1\n").unwrap();
    fs::write(level2.join("level2.md"), "# Level 2\n").unwrap();
    fs::write(level3.join("level3.md"), "# Level 3\n").unwrap();

    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("--depth").arg("2").arg(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("root.md"))
        .stdout(predicate::str::contains("level1.md"))
        .stdout(predicate::str::contains("level2.md"))
        .stdout(predicate::str::contains("level3.md").not());
}

/// Test depth limit works with --all flag
#[test]
fn test_depth_with_all_flag() {
    let temp_dir = TempDir::new().unwrap();
    let level1 = temp_dir.path().join("level1");
    let node_modules = level1.join("node_modules");

    fs::create_dir_all(&node_modules).unwrap();
    fs::write(temp_dir.path().join("root.md"), "# Root\n").unwrap();
    fs::write(level1.join("level1.md"), "# Level 1\n").unwrap();
    fs::write(node_modules.join("deep.md"), "# Deep\n").unwrap();

    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("-a").arg("-d").arg("1").arg(temp_dir.path());

    // With depth=1, we can only go 1 level deep, so node_modules/deep.md is at depth 2
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("root.md"))
        .stdout(predicate::str::contains("level1.md"))
        .stdout(predicate::str::contains("deep.md").not());
}

/// Test that unlimited depth (default) traverses all levels
#[test]
fn test_unlimited_depth_default() {
    let temp_dir = TempDir::new().unwrap();
    let level1 = temp_dir.path().join("l1");
    let level2 = level1.join("l2");
    let level3 = level2.join("l3");

    fs::create_dir_all(&level3).unwrap();
    fs::write(temp_dir.path().join("root.md"), "# Root\n").unwrap();
    fs::write(level1.join("l1.md"), "# L1\n").unwrap();
    fs::write(level2.join("l2.md"), "# L2\n").unwrap();
    fs::write(level3.join("l3.md"), "# L3\n").unwrap();

    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg(temp_dir.path());

    // Without depth limit, should find all files
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("root.md"))
        .stdout(predicate::str::contains("l1.md"))
        .stdout(predicate::str::contains("l2.md"))
        .stdout(predicate::str::contains("l3.md"));
}

/// Test JSON output format
#[test]
fn test_json_output() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("test.md"), "# Heading\n## Subheading\n").unwrap();

    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("-o").arg("json").arg(temp_dir.path());

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(serde_json::from_str::<Vec<serde_json::Value>>(&stdout).is_ok());
    assert!(stdout.contains("\"path\""));
    assert!(stdout.contains("\"size\""));
    assert!(stdout.contains("\"lines\""));
    assert!(stdout.contains("\"headings\""));
    assert!(stdout.contains("\"level\""));
    assert!(stdout.contains("\"text\""));
    assert!(stdout.contains("\"line_number\""));
}

/// Test YAML output format
#[test]
fn test_yaml_output() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("test.md"), "# Heading\n## Subheading\n").unwrap();

    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("-o").arg("yaml").arg(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("path:"))
        .stdout(predicate::str::contains("size:"))
        .stdout(predicate::str::contains("lines:"))
        .stdout(predicate::str::contains("headings:"))
        .stdout(predicate::str::contains("level:"))
        .stdout(predicate::str::contains("text:"))
        .stdout(predicate::str::contains("line_number:"));
}

/// Test text output format (default)
#[test]
fn test_text_output_explicit() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("test.md"), "# Heading\n").unwrap();

    let mut cmd = Command::new(cargo::cargo_bin!("lsmkd"));
    cmd.arg("-o").arg("text").arg(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test.md"))
        .stdout(predicate::str::contains("Heading {line: 1}"));
}
