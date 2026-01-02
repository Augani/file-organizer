use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::categories::{CategoryMapper, FileCategory};

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub category: FileCategory,
}

#[derive(Debug)]
pub struct ScanResult {
    pub files: Vec<FileInfo>,
    pub categorized: HashMap<FileCategory, Vec<FileInfo>>,
    pub total_count: usize,
}

impl ScanResult {
    pub fn category_count(&self, category: &FileCategory) -> usize {
        self.categorized.get(category).map_or(0, |v| v.len())
    }
}

pub struct DirectoryScanner {
    mapper: CategoryMapper,
}

impl DirectoryScanner {
    pub fn new(mapper: CategoryMapper) -> Self {
        Self { mapper }
    }

    pub fn scan(&self, source_dir: &Path) -> io::Result<ScanResult> {
        let mut files = Vec::new();
        let mut categorized: HashMap<FileCategory, Vec<FileInfo>> = HashMap::new();

        self.scan_directory(source_dir, &mut files)?;

        for file in &files {
            categorized
                .entry(file.category.clone())
                .or_default()
                .push(file.clone());
        }

        let total_count = files.len();

        Ok(ScanResult {
            files,
            categorized,
            total_count,
        })
    }

    fn scan_directory(&self, dir: &Path, files: &mut Vec<FileInfo>) -> io::Result<()> {
        if !dir.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                format!("{} is not a directory", dir.display()),
            ));
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(file_info) = self.process_file(&path) {
                    files.push(file_info);
                }
            }
        }

        Ok(())
    }

    fn process_file(&self, path: &Path) -> Option<FileInfo> {
        let name = path.file_name()?.to_string_lossy().to_string();

        // Skip hidden files (starting with .)
        if name.starts_with('.') {
            return None;
        }

        let extension = path
            .extension()
            .map(|ext| ext.to_string_lossy().to_string());

        let category = match &extension {
            Some(ext) => self.mapper.categorize(ext),
            None => FileCategory::Other,
        };

        Some(FileInfo {
            path: path.to_path_buf(),
            name,
            extension,
            category,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_scan_empty_directory() {
        let dir = tempdir().unwrap();
        let scanner = DirectoryScanner::new(CategoryMapper::new());
        let result = scanner.scan(dir.path()).unwrap();

        assert_eq!(result.total_count, 0);
        assert!(result.files.is_empty());
    }

    #[test]
    fn test_scan_with_files() {
        let dir = tempdir().unwrap();

        // Create test files
        File::create(dir.path().join("photo.jpg")).unwrap();
        File::create(dir.path().join("document.pdf")).unwrap();
        File::create(dir.path().join("music.mp3")).unwrap();

        let scanner = DirectoryScanner::new(CategoryMapper::new());
        let result = scanner.scan(dir.path()).unwrap();

        assert_eq!(result.total_count, 3);
        assert_eq!(result.category_count(&FileCategory::Images), 1);
        assert_eq!(result.category_count(&FileCategory::Documents), 1);
        assert_eq!(result.category_count(&FileCategory::Audio), 1);
    }

    #[test]
    fn test_skip_hidden_files() {
        let dir = tempdir().unwrap();

        File::create(dir.path().join(".hidden")).unwrap();
        File::create(dir.path().join("visible.txt")).unwrap();

        let scanner = DirectoryScanner::new(CategoryMapper::new());
        let result = scanner.scan(dir.path()).unwrap();

        assert_eq!(result.total_count, 1);
    }

    #[test]
    fn test_file_without_extension() {
        let dir = tempdir().unwrap();

        File::create(dir.path().join("noextension")).unwrap();

        let scanner = DirectoryScanner::new(CategoryMapper::new());
        let result = scanner.scan(dir.path()).unwrap();

        assert_eq!(result.total_count, 1);
        assert_eq!(result.category_count(&FileCategory::Other), 1);
    }
}
