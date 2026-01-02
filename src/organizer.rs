use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::categories::FileCategory;
use crate::scanner::{FileInfo, ScanResult};

#[derive(Debug, Clone)]
pub struct MoveOperation {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub file_name: String,
}

#[derive(Debug, Default)]
pub struct MoveResult {
    pub moved: Vec<MoveOperation>,
    pub skipped: Vec<(MoveOperation, String)>,
    pub failed: Vec<(MoveOperation, String)>,
}

impl MoveResult {
    pub fn print_summary(&self, dry_run: bool) {
        println!("\n{}", "=".repeat(50));
        if dry_run {
            println!("DRY RUN SUMMARY");
        } else {
            println!("ORGANIZATION COMPLETE");
        }
        println!("{}", "=".repeat(50));

        if dry_run {
            println!("Files that would be moved: {}", self.moved.len());
        } else {
            println!("Files successfully moved:  {}", self.moved.len());
        }

        if !self.skipped.is_empty() {
            println!("Files skipped:             {}", self.skipped.len());
        }

        if !self.failed.is_empty() {
            println!("Files failed:              {}", self.failed.len());
        }

        let total = self.moved.len() + self.skipped.len() + self.failed.len();
        println!("Total files processed:     {}", total);

        // Show skipped files details
        if !self.skipped.is_empty() {
            println!("\nSkipped files:");
            for (op, reason) in &self.skipped {
                println!("  {} - {}", op.file_name, reason);
            }
        }

        // Show failed files details
        if !self.failed.is_empty() {
            println!("\nFailed files:");
            for (op, reason) in &self.failed {
                println!("  {} - {}", op.file_name, reason);
            }
        }

        println!("{}", "=".repeat(50));
    }
}

pub struct Organizer {
    output_dir: PathBuf,
    dry_run: bool,
    verbose: bool,
}

impl Organizer {
    pub fn new(output_dir: PathBuf, dry_run: bool, verbose: bool) -> Self {
        Self {
            output_dir,
            dry_run,
            verbose,
        }
    }

    pub fn create_category_directories(&self, scan_result: &ScanResult) -> io::Result<Vec<PathBuf>> {
        let categories_needed: HashSet<&FileCategory> = scan_result
            .categorized
            .keys()
            .collect();

        let mut created_dirs = Vec::new();

        for category in categories_needed {
            let category_path = self.output_dir.join(category.folder_name());

            if category_path.exists() {
                if self.verbose {
                    println!("  Directory already exists: {}", category_path.display());
                }
            } else if self.dry_run {
                println!("  [DRY RUN] Would create: {}", category_path.display());
                created_dirs.push(category_path);
            } else {
                fs::create_dir_all(&category_path)?;
                if self.verbose {
                    println!("  Created directory: {}", category_path.display());
                }
                created_dirs.push(category_path);
            }
        }

        Ok(created_dirs)
    }

    pub fn get_target_path(&self, category: &FileCategory, file_name: &str) -> PathBuf {
        self.output_dir.join(category.folder_name()).join(file_name)
    }

    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    pub fn move_files(&self, scan_result: &ScanResult) -> MoveResult {
        let mut result = MoveResult::default();
        let total_files = scan_result.files.len();

        for (index, file) in scan_result.files.iter().enumerate() {
            let progress = index + 1;
            let operation = self.create_move_operation(file);

            if let Err(reason) = self.should_move(&operation) {
                if self.verbose {
                    println!("  [{}/{}] Skipping {}: {}", progress, total_files, file.name, reason);
                }
                result.skipped.push((operation, reason));
                continue;
            }

            if self.dry_run {
                if self.verbose {
                    println!(
                        "  [{}/{}] Would move: {} -> {}",
                        progress,
                        total_files,
                        file.name,
                        operation.destination.parent().unwrap().file_name().unwrap().to_string_lossy()
                    );
                }
                result.moved.push(operation);
            } else {
                match self.execute_move(&operation) {
                    Ok(()) => {
                        if self.verbose {
                            println!(
                                "  [{}/{}] Moved: {} -> {}",
                                progress,
                                total_files,
                                operation.file_name,
                                operation.destination.parent().unwrap().file_name().unwrap().to_string_lossy()
                            );
                        }
                        result.moved.push(operation);
                    }
                    Err(e) => {
                        let reason = e.to_string();
                        if self.verbose {
                            println!("  [{}/{}] Failed to move {}: {}", progress, total_files, file.name, reason);
                        }
                        result.failed.push((operation, reason));
                    }
                }
            }

            // Show progress every 10 files in non-verbose mode, or on last file
            if !self.verbose && (progress % 10 == 0 || progress == total_files) {
                println!("  Processed {}/{} files...", progress, total_files);
            }
        }

        result
    }

    fn create_move_operation(&self, file: &FileInfo) -> MoveOperation {
        let destination = self.get_target_path(&file.category, &file.name);
        MoveOperation {
            source: file.path.clone(),
            destination,
            file_name: file.name.clone(),
        }
    }

    fn should_move(&self, operation: &MoveOperation) -> Result<(), String> {
        // Skip if source and destination are the same
        if operation.source == operation.destination {
            return Err("source and destination are the same".to_string());
        }

        // Skip if destination file already exists
        if operation.destination.exists() {
            return Err("destination file already exists".to_string());
        }

        // Skip detailed permission checks in dry-run mode since directories may not exist yet
        if self.dry_run {
            return Ok(());
        }

        // Check if source file is readable
        if let Err(e) = self.check_source_readable(&operation.source) {
            return Err(e);
        }

        // Check if destination directory is writable
        if let Err(e) = self.check_destination_writable(&operation.destination) {
            return Err(e);
        }

        Ok(())
    }

    fn check_source_readable(&self, source: &Path) -> Result<(), String> {
        // Try to open the file for reading to verify access
        match fs::File::open(source) {
            Ok(_) => Ok(()),
            Err(e) => Err(self.format_io_error("cannot read source file", &e)),
        }
    }

    fn check_destination_writable(&self, destination: &Path) -> Result<(), String> {
        if let Some(parent) = destination.parent() {
            if parent.exists() {
                // Check if we can write to the directory by checking metadata
                match fs::metadata(parent) {
                    Ok(metadata) => {
                        if metadata.permissions().readonly() {
                            return Err("destination directory is read-only".to_string());
                        }
                        Ok(())
                    }
                    Err(e) => Err(self.format_io_error("cannot access destination directory", &e)),
                }
            } else {
                Err("destination directory does not exist".to_string())
            }
        } else {
            Err("invalid destination path".to_string())
        }
    }

    fn execute_move(&self, operation: &MoveOperation) -> io::Result<()> {
        // Try rename first (fastest, works on same filesystem)
        match fs::rename(&operation.source, &operation.destination) {
            Ok(()) => Ok(()),
            Err(e) => {
                // Handle cross-device link error by copying and deleting
                if e.raw_os_error() == Some(libc::EXDEV) {
                    self.copy_and_delete(operation)
                } else {
                    Err(self.enhance_io_error(e, operation))
                }
            }
        }
    }

    fn copy_and_delete(&self, operation: &MoveOperation) -> io::Result<()> {
        // Copy the file first
        fs::copy(&operation.source, &operation.destination)?;

        // Then delete the source
        fs::remove_file(&operation.source).map_err(|e| {
            // If we can't delete the source, try to clean up the destination
            let _ = fs::remove_file(&operation.destination);
            io::Error::new(
                e.kind(),
                format!("copied file but failed to remove source: {}", e),
            )
        })
    }

    fn enhance_io_error(&self, error: io::Error, operation: &MoveOperation) -> io::Error {
        let message = match error.kind() {
            io::ErrorKind::PermissionDenied => {
                format!(
                    "permission denied moving '{}' to '{}'",
                    operation.source.display(),
                    operation.destination.display()
                )
            }
            io::ErrorKind::NotFound => {
                format!("source file '{}' not found", operation.source.display())
            }
            io::ErrorKind::AlreadyExists => {
                format!(
                    "destination '{}' already exists",
                    operation.destination.display()
                )
            }
            _ => {
                format!(
                    "failed to move '{}': {}",
                    operation.file_name,
                    error
                )
            }
        };
        io::Error::new(error.kind(), message)
    }

    fn format_io_error(&self, context: &str, error: &io::Error) -> String {
        match error.kind() {
            io::ErrorKind::PermissionDenied => format!("{}: permission denied", context),
            io::ErrorKind::NotFound => format!("{}: file not found", context),
            _ => format!("{}: {}", context, error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::categories::CategoryMapper;
    use crate::scanner::DirectoryScanner;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_create_directories_for_categories() {
        let source_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        // Create test files
        File::create(source_dir.path().join("photo.jpg")).unwrap();
        File::create(source_dir.path().join("document.pdf")).unwrap();

        let scanner = DirectoryScanner::new(CategoryMapper::new());
        let scan_result = scanner.scan(source_dir.path()).unwrap();

        let organizer = Organizer::new(output_dir.path().to_path_buf(), false, false);
        let created = organizer.create_category_directories(&scan_result).unwrap();

        assert_eq!(created.len(), 2);
        assert!(output_dir.path().join("Images").exists());
        assert!(output_dir.path().join("Documents").exists());
    }

    #[test]
    fn test_dry_run_does_not_create_directories() {
        let source_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        File::create(source_dir.path().join("photo.jpg")).unwrap();

        let scanner = DirectoryScanner::new(CategoryMapper::new());
        let scan_result = scanner.scan(source_dir.path()).unwrap();

        let organizer = Organizer::new(output_dir.path().to_path_buf(), true, false);
        let created = organizer.create_category_directories(&scan_result).unwrap();

        assert_eq!(created.len(), 1);
        assert!(!output_dir.path().join("Images").exists());
    }

    #[test]
    fn test_existing_directory_not_recreated() {
        let source_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        // Create the Images directory ahead of time
        fs::create_dir(output_dir.path().join("Images")).unwrap();

        File::create(source_dir.path().join("photo.jpg")).unwrap();

        let scanner = DirectoryScanner::new(CategoryMapper::new());
        let scan_result = scanner.scan(source_dir.path()).unwrap();

        let organizer = Organizer::new(output_dir.path().to_path_buf(), false, false);
        let created = organizer.create_category_directories(&scan_result).unwrap();

        // Should not be in created list since it already existed
        assert_eq!(created.len(), 0);
        assert!(output_dir.path().join("Images").exists());
    }

    #[test]
    fn test_get_target_path() {
        let output_dir = tempdir().unwrap();
        let organizer = Organizer::new(output_dir.path().to_path_buf(), false, false);

        let target = organizer.get_target_path(&FileCategory::Images, "photo.jpg");
        assert_eq!(
            target,
            output_dir.path().join("Images").join("photo.jpg")
        );
    }

    #[test]
    fn test_empty_scan_creates_no_directories() {
        let source_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        let scanner = DirectoryScanner::new(CategoryMapper::new());
        let scan_result = scanner.scan(source_dir.path()).unwrap();

        let organizer = Organizer::new(output_dir.path().to_path_buf(), false, false);
        let created = organizer.create_category_directories(&scan_result).unwrap();

        assert!(created.is_empty());
    }

    #[test]
    fn test_move_files() {
        let source_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        // Create test files with content
        std::fs::write(source_dir.path().join("photo.jpg"), "image data").unwrap();
        std::fs::write(source_dir.path().join("document.pdf"), "pdf data").unwrap();

        let scanner = DirectoryScanner::new(CategoryMapper::new());
        let scan_result = scanner.scan(source_dir.path()).unwrap();

        let organizer = Organizer::new(output_dir.path().to_path_buf(), false, false);
        organizer.create_category_directories(&scan_result).unwrap();
        let move_result = organizer.move_files(&scan_result);

        assert_eq!(move_result.moved.len(), 2);
        assert!(move_result.failed.is_empty());
        assert!(move_result.skipped.is_empty());

        // Verify files were moved
        assert!(output_dir.path().join("Images/photo.jpg").exists());
        assert!(output_dir.path().join("Documents/document.pdf").exists());

        // Verify source files no longer exist
        assert!(!source_dir.path().join("photo.jpg").exists());
        assert!(!source_dir.path().join("document.pdf").exists());
    }

    #[test]
    fn test_dry_run_does_not_move_files() {
        let source_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        std::fs::write(source_dir.path().join("photo.jpg"), "image data").unwrap();

        let scanner = DirectoryScanner::new(CategoryMapper::new());
        let scan_result = scanner.scan(source_dir.path()).unwrap();

        let organizer = Organizer::new(output_dir.path().to_path_buf(), true, false);
        organizer.create_category_directories(&scan_result).unwrap();
        let move_result = organizer.move_files(&scan_result);

        assert_eq!(move_result.moved.len(), 1);

        // File should still exist in source (dry run)
        assert!(source_dir.path().join("photo.jpg").exists());
    }

    #[test]
    fn test_skip_existing_destination() {
        let source_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        std::fs::write(source_dir.path().join("photo.jpg"), "new image").unwrap();

        // Create destination with existing file
        fs::create_dir_all(output_dir.path().join("Images")).unwrap();
        std::fs::write(output_dir.path().join("Images/photo.jpg"), "existing image").unwrap();

        let scanner = DirectoryScanner::new(CategoryMapper::new());
        let scan_result = scanner.scan(source_dir.path()).unwrap();

        let organizer = Organizer::new(output_dir.path().to_path_buf(), false, false);
        let move_result = organizer.move_files(&scan_result);

        assert!(move_result.moved.is_empty());
        assert_eq!(move_result.skipped.len(), 1);

        // Source file should still exist
        assert!(source_dir.path().join("photo.jpg").exists());

        // Destination should have original content
        let content = std::fs::read_to_string(output_dir.path().join("Images/photo.jpg")).unwrap();
        assert_eq!(content, "existing image");
    }

    #[test]
    fn test_move_to_same_directory_skipped() {
        let dir = tempdir().unwrap();

        // Create the category directory
        fs::create_dir_all(dir.path().join("Images")).unwrap();

        // Put a file directly in the source
        std::fs::write(dir.path().join("photo.jpg"), "image data").unwrap();

        let scanner = DirectoryScanner::new(CategoryMapper::new());
        let scan_result = scanner.scan(dir.path()).unwrap();

        // Use same directory as output
        let organizer = Organizer::new(dir.path().to_path_buf(), false, false);
        let move_result = organizer.move_files(&scan_result);

        // File should be moved to Images subfolder
        assert_eq!(move_result.moved.len(), 1);
        assert!(dir.path().join("Images/photo.jpg").exists());
    }

    #[test]
    fn test_check_source_readable() {
        let source_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        // Create a readable file
        std::fs::write(source_dir.path().join("photo.jpg"), "image data").unwrap();

        let organizer = Organizer::new(output_dir.path().to_path_buf(), false, false);
        let result = organizer.check_source_readable(&source_dir.path().join("photo.jpg"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_source_not_found() {
        let source_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        let organizer = Organizer::new(output_dir.path().to_path_buf(), false, false);
        let result = organizer.check_source_readable(&source_dir.path().join("nonexistent.jpg"));

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("file not found"));
    }

    #[test]
    fn test_check_destination_writable() {
        let output_dir = tempdir().unwrap();

        // Create the Images directory
        fs::create_dir_all(output_dir.path().join("Images")).unwrap();

        let organizer = Organizer::new(output_dir.path().to_path_buf(), false, false);
        let result = organizer.check_destination_writable(&output_dir.path().join("Images/photo.jpg"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_destination_directory_missing() {
        let output_dir = tempdir().unwrap();

        let organizer = Organizer::new(output_dir.path().to_path_buf(), false, false);
        let result = organizer.check_destination_writable(&output_dir.path().join("NonExistent/photo.jpg"));

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_format_io_error_permission_denied() {
        let output_dir = tempdir().unwrap();
        let organizer = Organizer::new(output_dir.path().to_path_buf(), false, false);

        let error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test error");
        let formatted = organizer.format_io_error("test context", &error);

        assert!(formatted.contains("permission denied"));
        assert!(formatted.contains("test context"));
    }

    #[test]
    fn test_format_io_error_not_found() {
        let output_dir = tempdir().unwrap();
        let organizer = Organizer::new(output_dir.path().to_path_buf(), false, false);

        let error = std::io::Error::new(std::io::ErrorKind::NotFound, "test error");
        let formatted = organizer.format_io_error("test context", &error);

        assert!(formatted.contains("file not found"));
        assert!(formatted.contains("test context"));
    }

    #[test]
    fn test_enhance_io_error_permission_denied() {
        let output_dir = tempdir().unwrap();
        let organizer = Organizer::new(output_dir.path().to_path_buf(), false, false);

        let operation = MoveOperation {
            source: PathBuf::from("/source/file.jpg"),
            destination: PathBuf::from("/dest/file.jpg"),
            file_name: "file.jpg".to_string(),
        };

        let error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test");
        let enhanced = organizer.enhance_io_error(error, &operation);

        assert!(enhanced.to_string().contains("permission denied"));
        assert!(enhanced.to_string().contains("/source/file.jpg"));
    }

    #[test]
    fn test_move_result_has_failures() {
        let mut result = MoveResult::default();
        assert!(result.failed.is_empty());

        result.failed.push((
            MoveOperation {
                source: PathBuf::from("/source/file.jpg"),
                destination: PathBuf::from("/dest/file.jpg"),
                file_name: "file.jpg".to_string(),
            },
            "permission denied".to_string(),
        ));

        assert_eq!(result.failed.len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn test_unreadable_source_file() {
        use std::os::unix::fs::PermissionsExt;

        let source_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        // Create a file and make it unreadable
        let file_path = source_dir.path().join("unreadable.jpg");
        std::fs::write(&file_path, "data").unwrap();

        // Remove all permissions
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&file_path, perms).unwrap();

        let organizer = Organizer::new(output_dir.path().to_path_buf(), false, false);
        let result = organizer.check_source_readable(&file_path);

        // Restore permissions for cleanup
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&file_path, perms).unwrap();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("permission denied"));
    }
}
