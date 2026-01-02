use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileCategory {
    Images,
    Documents,
    Videos,
    Audio,
    Archives,
    Code,
    Data,
    Executables,
    Fonts,
    Other,
}

impl FileCategory {
    pub fn folder_name(&self) -> &'static str {
        match self {
            FileCategory::Images => "Images",
            FileCategory::Documents => "Documents",
            FileCategory::Videos => "Videos",
            FileCategory::Audio => "Audio",
            FileCategory::Archives => "Archives",
            FileCategory::Code => "Code",
            FileCategory::Data => "Data",
            FileCategory::Executables => "Executables",
            FileCategory::Fonts => "Fonts",
            FileCategory::Other => "Other",
        }
    }
}

pub struct CategoryMapper {
    extension_map: HashMap<String, FileCategory>,
}

impl CategoryMapper {
    pub fn new() -> Self {
        let mut extension_map = HashMap::new();

        // Images
        for ext in ["jpg", "jpeg", "png", "gif", "bmp", "svg", "webp", "ico", "tiff", "tif", "raw", "heic", "heif", "psd", "ai", "eps"] {
            extension_map.insert(ext.to_string(), FileCategory::Images);
        }

        // Documents
        for ext in ["pdf", "doc", "docx", "txt", "rtf", "odt", "xls", "xlsx", "ods", "ppt", "pptx", "odp", "md", "tex", "pages", "numbers", "keynote", "epub", "mobi"] {
            extension_map.insert(ext.to_string(), FileCategory::Documents);
        }

        // Videos
        for ext in ["mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "mpeg", "mpg", "3gp", "ogv"] {
            extension_map.insert(ext.to_string(), FileCategory::Videos);
        }

        // Audio
        for ext in ["mp3", "wav", "flac", "aac", "ogg", "wma", "m4a", "aiff", "alac", "opus", "mid", "midi"] {
            extension_map.insert(ext.to_string(), FileCategory::Audio);
        }

        // Archives
        for ext in ["zip", "rar", "7z", "tar", "gz", "bz2", "xz", "tgz", "tbz2", "cab", "iso", "dmg"] {
            extension_map.insert(ext.to_string(), FileCategory::Archives);
        }

        // Code
        for ext in ["rs", "py", "js", "ts", "jsx", "tsx", "java", "c", "cpp", "h", "hpp", "cs", "go", "rb", "php", "swift", "kt", "scala", "r", "pl", "sh", "bash", "zsh", "fish", "ps1", "bat", "cmd", "html", "htm", "css", "scss", "sass", "less", "vue", "svelte", "sql", "graphql", "yaml", "yml", "toml", "ini", "cfg", "conf"] {
            extension_map.insert(ext.to_string(), FileCategory::Code);
        }

        // Data
        for ext in ["json", "xml", "csv", "tsv", "parquet", "avro", "db", "sqlite", "sqlite3", "mdb", "accdb", "ndjson", "jsonl"] {
            extension_map.insert(ext.to_string(), FileCategory::Data);
        }

        // Executables
        for ext in ["exe", "msi", "app", "deb", "rpm", "apk", "jar", "dll", "so", "dylib", "bin", "run"] {
            extension_map.insert(ext.to_string(), FileCategory::Executables);
        }

        // Fonts
        for ext in ["ttf", "otf", "woff", "woff2", "eot", "fon", "fnt"] {
            extension_map.insert(ext.to_string(), FileCategory::Fonts);
        }

        Self { extension_map }
    }

    pub fn categorize(&self, extension: &str) -> FileCategory {
        let ext_lower = extension.to_lowercase();
        self.extension_map
            .get(&ext_lower)
            .cloned()
            .unwrap_or(FileCategory::Other)
    }

    pub fn all_categories(&self) -> Vec<FileCategory> {
        vec![
            FileCategory::Images,
            FileCategory::Documents,
            FileCategory::Videos,
            FileCategory::Audio,
            FileCategory::Archives,
            FileCategory::Code,
            FileCategory::Data,
            FileCategory::Executables,
            FileCategory::Fonts,
            FileCategory::Other,
        ]
    }
}

impl Default for CategoryMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_extensions() {
        let mapper = CategoryMapper::new();
        assert_eq!(mapper.categorize("jpg"), FileCategory::Images);
        assert_eq!(mapper.categorize("PNG"), FileCategory::Images);
        assert_eq!(mapper.categorize("gif"), FileCategory::Images);
    }

    #[test]
    fn test_document_extensions() {
        let mapper = CategoryMapper::new();
        assert_eq!(mapper.categorize("pdf"), FileCategory::Documents);
        assert_eq!(mapper.categorize("DOCX"), FileCategory::Documents);
        assert_eq!(mapper.categorize("txt"), FileCategory::Documents);
    }

    #[test]
    fn test_unknown_extension() {
        let mapper = CategoryMapper::new();
        assert_eq!(mapper.categorize("xyz123"), FileCategory::Other);
    }

    #[test]
    fn test_folder_names() {
        assert_eq!(FileCategory::Images.folder_name(), "Images");
        assert_eq!(FileCategory::Documents.folder_name(), "Documents");
        assert_eq!(FileCategory::Other.folder_name(), "Other");
    }
}
