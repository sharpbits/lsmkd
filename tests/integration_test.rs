use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test that lsmkd correctly processes the architecture.md sample
#[test]
fn test_architecture_doc_sample() {
    let mut cmd = Command::cargo_bin("lsmkd").unwrap();
    cmd.arg("tests/docs/architecture.md");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("tests/docs/architecture.md"))
        .stdout(predicate::str::contains("Zephyr Platform Architecture [L1]"))
        .stdout(predicate::str::contains("Overview [L3]"))
        .stdout(predicate::str::contains("Core Components [L7]"))
        .stdout(predicate::str::contains("Data Flow [L30]"))
        .stdout(predicate::str::contains("Deployment Model [L40]"))
        .stdout(predicate::str::contains("Security Architecture [L44]"));
}

/// Test that lsmkd correctly processes the prd.md sample
#[test]
fn test_prd_doc_sample() {
    let mut cmd = Command::cargo_bin("lsmkd").unwrap();
    cmd.arg("tests/docs/prd.md");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("tests/docs/prd.md"))
        .stdout(predicate::str::contains("Zephyr Platform - Product Requirements Document [L1]"))
        .stdout(predicate::str::contains("Executive Summary [L7]"))
        .stdout(predicate::str::contains("Business Objectives [L11]"))
        .stdout(predicate::str::contains("Target Users [L18]"))
        .stdout(predicate::str::contains("Core Features [L24]"))
        .stdout(predicate::str::contains("Non-Functional Requirements [L66]"))
        .stdout(predicate::str::contains("Deployment Constraints [L77]"))
        .stdout(predicate::str::contains("Success Metrics [L83]"))
        .stdout(predicate::str::contains("Timeline [L90]"))
        .stdout(predicate::str::contains("Assumptions [L97]"))
        .stdout(predicate::str::contains("Open Questions [L104]"));
}

/// Test that lsmkd processes entire docs directory recursively
#[test]
fn test_recursive_directory_processing() {
    let mut cmd = Command::cargo_bin("lsmkd").unwrap();
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

    let mut cmd = Command::cargo_bin("lsmkd").unwrap();
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
    let mut cmd = Command::cargo_bin("lsmkd").unwrap();
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

    let mut cmd = Command::cargo_bin("lsmkd").unwrap();
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

    let mut cmd = Command::cargo_bin("lsmkd").unwrap();
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

    let mut cmd = Command::cargo_bin("lsmkd").unwrap();
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

    let mut cmd = Command::cargo_bin("lsmkd").unwrap();
    cmd.arg("-a").arg(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("included.md"))
        .stdout(predicate::str::contains("Included"));
}

/// Test processing multiple file arguments
#[test]
fn test_multiple_file_arguments() {
    let mut cmd = Command::cargo_bin("lsmkd").unwrap();
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

    let mut cmd = Command::cargo_bin("lsmkd").unwrap();
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

    let mut cmd = Command::cargo_bin("lsmkd").unwrap();
    cmd.arg(&test_file);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("empty.md"))
        .stdout(predicate::str::contains("(no headings found)"));
}

/// Test that non-existent path returns error
#[test]
fn test_nonexistent_path_error() {
    let mut cmd = Command::cargo_bin("lsmkd").unwrap();
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

    let mut cmd = Command::cargo_bin("lsmkd").unwrap();
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
    let mut cmd = Command::cargo_bin("lsmkd").unwrap();
    cmd.arg("-h");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List and index markdown files"))
        .stdout(predicate::str::contains("Usage:"));
}

/// Test version flag displays version
#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("lsmkd").unwrap();
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

    let mut cmd = Command::cargo_bin("lsmkd").unwrap();
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

    let mut cmd = Command::cargo_bin("lsmkd").unwrap();
    cmd.arg(&test_file);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // H1 shows as "└── H1 [L1]" at root level
    // H2 shows as "    └── H2 [L2]" with indentation for level 2
    assert!(stdout.contains("H1 [L1]"));
    assert!(stdout.contains("H2 [L2]"));
    assert!(stdout.contains("└──") || stdout.contains("├──"));
}
