use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn get_binary_path() -> String {
    // Build the binary first to ensure it's up to date
    let output = Command::new("cargo")
        .args(["build", "--release"])
        .output()
        .expect("Failed to build");

    assert!(output.status.success(), "Failed to build binary");

    // Return path to the release binary
    format!(
        "{}/target/release/file-organizer",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[test]
fn test_cli_help() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("organize files by their extensions"));
    assert!(stdout.contains("--source"));
    assert!(stdout.contains("--output"));
    assert!(stdout.contains("--dry-run"));
    assert!(stdout.contains("--verbose"));
}

#[test]
fn test_cli_version() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("file-organizer"));
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn test_organize_files_end_to_end() {
    let binary = get_binary_path();
    let source_dir = tempdir().unwrap();

    // Create test files of various types
    fs::write(source_dir.path().join("photo1.jpg"), "image data 1").unwrap();
    fs::write(source_dir.path().join("photo2.png"), "image data 2").unwrap();
    fs::write(source_dir.path().join("document.pdf"), "pdf data").unwrap();
    fs::write(source_dir.path().join("music.mp3"), "audio data").unwrap();
    fs::write(source_dir.path().join("video.mp4"), "video data").unwrap();
    fs::write(source_dir.path().join("archive.zip"), "archive data").unwrap();
    fs::write(source_dir.path().join("code.rs"), "fn main() {}").unwrap();
    fs::write(source_dir.path().join("data.json"), "{}").unwrap();

    // Run the organizer
    let output = Command::new(&binary)
        .args(["-s", source_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);

    // Verify files were moved to correct categories
    assert!(source_dir.path().join("Images/photo1.jpg").exists());
    assert!(source_dir.path().join("Images/photo2.png").exists());
    assert!(source_dir.path().join("Documents/document.pdf").exists());
    assert!(source_dir.path().join("Audio/music.mp3").exists());
    assert!(source_dir.path().join("Videos/video.mp4").exists());
    assert!(source_dir.path().join("Archives/archive.zip").exists());
    assert!(source_dir.path().join("Code/code.rs").exists());
    assert!(source_dir.path().join("Data/data.json").exists());

    // Verify source files no longer exist in root
    assert!(!source_dir.path().join("photo1.jpg").exists());
    assert!(!source_dir.path().join("document.pdf").exists());
}

#[test]
fn test_dry_run_does_not_move_files() {
    let binary = get_binary_path();
    let source_dir = tempdir().unwrap();

    // Create test files
    fs::write(source_dir.path().join("photo.jpg"), "image data").unwrap();
    fs::write(source_dir.path().join("document.pdf"), "pdf data").unwrap();

    // Run with --dry-run
    let output = Command::new(&binary)
        .args(["-s", source_dir.path().to_str().unwrap(), "--dry-run"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DRY RUN"));
    assert!(stdout.contains("Would"));

    // Files should still be in their original location
    assert!(source_dir.path().join("photo.jpg").exists());
    assert!(source_dir.path().join("document.pdf").exists());

    // Category directories should NOT be created
    assert!(!source_dir.path().join("Images").exists());
    assert!(!source_dir.path().join("Documents").exists());
}

#[test]
fn test_verbose_output() {
    let binary = get_binary_path();
    let source_dir = tempdir().unwrap();

    fs::write(source_dir.path().join("photo.jpg"), "image data").unwrap();

    let output = Command::new(&binary)
        .args([
            "-s",
            source_dir.path().to_str().unwrap(),
            "--verbose",
            "--dry-run",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Verbose mode: enabled"));
    assert!(stdout.contains("Supported categories:"));
    assert!(stdout.contains("Images"));
    assert!(stdout.contains("Documents"));
}

#[test]
fn test_separate_output_directory() {
    let binary = get_binary_path();
    let source_dir = tempdir().unwrap();
    let output_dir = tempdir().unwrap();

    // Create test files
    fs::write(source_dir.path().join("photo.jpg"), "image data").unwrap();
    fs::write(source_dir.path().join("document.pdf"), "pdf data").unwrap();

    // Run with separate output directory
    let output = Command::new(&binary)
        .args([
            "-s",
            source_dir.path().to_str().unwrap(),
            "-o",
            output_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Files should be in output directory
    assert!(output_dir.path().join("Images/photo.jpg").exists());
    assert!(output_dir.path().join("Documents/document.pdf").exists());

    // Source should no longer have files
    assert!(!source_dir.path().join("photo.jpg").exists());
    assert!(!source_dir.path().join("document.pdf").exists());
}

#[test]
fn test_nonexistent_source_directory() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(["-s", "/nonexistent/directory/path"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error") || stderr.contains("not a directory"));
}

#[test]
fn test_empty_directory() {
    let binary = get_binary_path();
    let source_dir = tempdir().unwrap();

    let output = Command::new(&binary)
        .args(["-s", source_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Found 0 files"));
    assert!(stdout.contains("No files to organize"));
}

#[test]
fn test_skip_hidden_files() {
    let binary = get_binary_path();
    let source_dir = tempdir().unwrap();

    // Create visible and hidden files
    fs::write(source_dir.path().join("photo.jpg"), "image data").unwrap();
    fs::write(source_dir.path().join(".hidden.jpg"), "hidden image").unwrap();

    let output = Command::new(&binary)
        .args(["-s", source_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Found 1 files"));

    // Hidden file should remain untouched
    assert!(source_dir.path().join(".hidden.jpg").exists());
}

#[test]
fn test_skip_existing_destination_file() {
    let binary = get_binary_path();
    let source_dir = tempdir().unwrap();

    // Create source file
    fs::write(source_dir.path().join("photo.jpg"), "new image").unwrap();

    // Create destination directory with existing file
    fs::create_dir_all(source_dir.path().join("Images")).unwrap();
    fs::write(source_dir.path().join("Images/photo.jpg"), "existing image").unwrap();

    let output = Command::new(&binary)
        .args(["-s", source_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("skipped"));

    // Source file should still exist
    assert!(source_dir.path().join("photo.jpg").exists());

    // Existing file should not be overwritten
    let content = fs::read_to_string(source_dir.path().join("Images/photo.jpg")).unwrap();
    assert_eq!(content, "existing image");
}

#[test]
fn test_summary_report() {
    let binary = get_binary_path();
    let source_dir = tempdir().unwrap();

    fs::write(source_dir.path().join("photo1.jpg"), "image 1").unwrap();
    fs::write(source_dir.path().join("photo2.jpg"), "image 2").unwrap();
    fs::write(source_dir.path().join("document.pdf"), "pdf").unwrap();

    let output = Command::new(&binary)
        .args(["-s", source_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ORGANIZATION COMPLETE"));
    assert!(stdout.contains("Files successfully moved"));
    assert!(stdout.contains("Total files processed"));
}

#[test]
fn test_multiple_files_same_category() {
    let binary = get_binary_path();
    let source_dir = tempdir().unwrap();

    // Create multiple images
    for i in 1..=5 {
        fs::write(
            source_dir.path().join(format!("photo{}.jpg", i)),
            format!("image data {}", i),
        )
        .unwrap();
    }

    let output = Command::new(&binary)
        .args(["-s", source_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // All images should be in Images folder
    for i in 1..=5 {
        assert!(source_dir
            .path()
            .join(format!("Images/photo{}.jpg", i))
            .exists());
    }
}

#[test]
fn test_files_without_extension() {
    let binary = get_binary_path();
    let source_dir = tempdir().unwrap();

    // Create file without extension
    fs::write(source_dir.path().join("noextension"), "some data").unwrap();
    fs::write(source_dir.path().join("photo.jpg"), "image data").unwrap();

    let output = Command::new(&binary)
        .args(["-s", source_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // File without extension goes to Other
    assert!(source_dir.path().join("Other/noextension").exists());
    assert!(source_dir.path().join("Images/photo.jpg").exists());
}

#[test]
fn test_case_insensitive_extensions() {
    let binary = get_binary_path();
    let source_dir = tempdir().unwrap();

    // Create files with different case extensions
    fs::write(source_dir.path().join("photo.JPG"), "image 1").unwrap();
    fs::write(source_dir.path().join("photo2.Jpg"), "image 2").unwrap();
    fs::write(source_dir.path().join("photo3.jpg"), "image 3").unwrap();

    let output = Command::new(&binary)
        .args(["-s", source_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // All should be in Images regardless of case
    assert!(source_dir.path().join("Images/photo.JPG").exists());
    assert!(source_dir.path().join("Images/photo2.Jpg").exists());
    assert!(source_dir.path().join("Images/photo3.jpg").exists());
}
