//! Archive editor for modifying existing archives
//!
//! This module provides BitArchiveEditor for editing archive contents
//! (renaming, updating, and deleting items).

use crate::ffi::BitLibrary;
use crate::format::CompressionFormat;
use crate::error::{Bit7zError, Result};
use crate::archive_reader::BitArchiveReader;
use crate::compressor::BitCompressor;
use crate::extractor::BitExtractor;
use crate::compress_callback::InputItem;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};

/// Delete policy for archive items
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeletePolicy {
    /// Delete only the specified item (for files)
    ItemOnly,
    /// Delete item and all its contents (for directories)
    RecurseDirs,
}

/// Archive editor for modifying existing archives
///
/// BitArchiveEditor allows editing of existing archives by:
/// - Renaming items within the archive
/// - Updating item contents from files or buffers
/// - Deleting items from the archive
///
/// Changes are applied when `apply_changes()` is called.
pub struct BitArchiveEditor<'a> {
    /// Reference to 7-Zip library
    library: &'a BitLibrary,
    /// Compression format
    format: CompressionFormat,
    /// Path to source archive
    source_archive: PathBuf,
    /// Path to temporary archive (for applying changes)
    temp_archive_path: Option<PathBuf>,
    /// Items to add/update (index -> InputItem)
    edited_items: HashMap<u32, InputItem>,
    /// Indices of items to delete
    deleted_indices: HashSet<u32>,
    /// Renamed items (index -> new_path)
    renamed_items: HashMap<u32, String>,
    /// Password for encryption
    password: Option<String>,
    /// Compression level
    compression_level: crate::format::CompressionLevel,
    /// Compression method
    compression_method: Option<crate::format::CompressionMethod>,
    /// Dictionary size
    dictionary_size: Option<u32>,
    /// Word size
    word_size: Option<u32>,
    /// Solid compression mode
    solid: bool,
}

impl<'a> BitArchiveEditor<'a> {
    /// Create a new archive editor
    ///
    /// # Arguments
    /// * `library` - 7-Zip library reference
    /// * `archive_path` - Path to existing archive to edit
    /// * `format` - Compression format
    /// * `password` - Optional password for encrypted archives
    ///
    /// # Returns
    /// * `Ok(BitArchiveEditor)` - Editor instance
    /// * `Err(Bit7zError)` - Error if archive cannot be opened
    pub fn new<P: AsRef<Path>>(
        library: &'a BitLibrary,
        archive_path: P,
        format: CompressionFormat,
        password: Option<String>,
    ) -> Result<Self> {
        let source_archive = archive_path.as_ref().to_path_buf();
        
        // Verify archive exists and can be opened
        if !source_archive.exists() {
            return Err(Bit7zError::OpenFailed(
                format!("Archive does not exist: {:?}", source_archive)
            ));
        }

        Ok(BitArchiveEditor {
            library,
            format,
            source_archive,
            temp_archive_path: None,
            edited_items: HashMap::new(),
            deleted_indices: HashSet::new(),
            renamed_items: HashMap::new(),
            password,
            compression_level: crate::format::CompressionLevel::Normal,
            compression_method: None,
            dictionary_size: None,
            word_size: None,
            solid: false,
        })
    }

    /// Set compression level
    pub fn compression_level(&mut self, level: crate::format::CompressionLevel) -> &mut Self {
        self.compression_level = level;
        self
    }

    /// Set compression method
    pub fn compression_method(&mut self, method: crate::format::CompressionMethod) -> &mut Self {
        self.compression_method = Some(method);
        self
    }

    /// Set dictionary size
    pub fn dictionary_size(&mut self, size: u32) -> &mut Self {
        self.dictionary_size = Some(size);
        self
    }

    /// Set word size
    pub fn word_size(&mut self, size: u32) -> &mut Self {
        self.word_size = Some(size);
        self
    }

    /// Set solid compression mode
    pub fn solid(&mut self, solid: bool) -> &mut Self {
        self.solid = solid;
        self
    }

    /// Rename an item in the archive
    ///
    /// # Arguments
    /// * `index` - Index of item to rename
    /// * `new_path` - New path/name in the archive
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(Bit7zError)` - Error if index is invalid
    pub fn rename_item(&mut self, index: u32, new_path: String) -> Result<()> {
        // Validate index by reading the archive
        self.validate_index(index)?;
        
        self.renamed_items.insert(index, new_path);
        Ok(())
    }

    /// Update an item's content from a file
    ///
    /// # Arguments
    /// * `index` - Index of item to update
    /// * `file_path` - Path to new content file
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(Bit7zError)` - Error if index is invalid or file doesn't exist
    pub fn update_item<P: AsRef<Path>>(&mut self, index: u32, file_path: P) -> Result<()> {
        // Validate index
        self.validate_index(index)?;
        
        let file_path = file_path.as_ref().to_path_buf();
        
        if !file_path.exists() {
            return Err(Bit7zError::CompressFailed(
                format!("File does not exist: {:?}", file_path)
            ));
        }

        let input_item = InputItem::new(&file_path);
        self.edited_items.insert(index, input_item);
        Ok(())
    }

    /// Update an item's content from a buffer
    ///
    /// # Arguments
    /// * `index` - Index of item to update
    /// * `buffer` - New content data
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(Bit7zError)` - Error if index is invalid
    pub fn update_item_from_buffer(&mut self, index: u32, buffer: Vec<u8>) -> Result<()> {
        // Validate index
        self.validate_index(index)?;
        
        // For buffer updates, we need to store the buffer somewhere
        // We'll use a temp file approach
        let temp_path = std::env::temp_dir().join(format!(
            "bit7z_edit_{}_{}.tmp",
            std::process::id(),
            index
        ));
        
        std::fs::write(&temp_path, &buffer)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;
        
        let input_item = InputItem::new(&temp_path);
        self.edited_items.insert(index, input_item);
        
        // Store temp path for cleanup later
        // Note: This is a simplified approach - production code should
        // track these temp files properly
        Ok(())
    }

    /// Update an item's content from a stream
    ///
    /// # Arguments
    /// * `index` - Index of item to update
    /// * `stream` - Stream providing new content data
    /// * `item_name` - Name to give the item in the archive
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(Bit7zError)` - Error if index is invalid
    pub fn update_item_from_stream<R: std::io::Read>(
        &mut self,
        index: u32,
        mut stream: R,
        item_name: String,
    ) -> Result<()> {
        // Validate index
        self.validate_index(index)?;
        
        // Create a temporary file to store stream content
        let temp_path = std::env::temp_dir().join(format!(
            "bit7z_stream_{}_{}.tmp",
            std::process::id(),
            index
        ));

        // Copy stream to temp file
        let mut temp_file = std::fs::File::create(&temp_path)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;
        
        std::io::copy(&mut stream, &mut temp_file)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;
        
        drop(temp_file);

        // Create input item
        let input_item = InputItem::with_name(&temp_path, item_name);
        self.edited_items.insert(index, input_item);
        
        Ok(())
    }

    /// Delete an item from the archive
    ///
    /// # Arguments
    /// * `index` - Index of item to delete
    /// * `policy` - Delete policy (ItemOnly or RecurseDirs)
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(Bit7zError)` - Error if index is invalid
    pub fn delete_item(&mut self, index: u32, policy: DeletePolicy) -> Result<()> {
        // Validate index
        self.validate_index(index)?;
        
        self.deleted_indices.insert(index);
        
        // If RecurseDirs and item is a directory, find and delete all children
        if policy == DeletePolicy::RecurseDirs {
            self.delete_children(index)?;
        }
        
        Ok(())
    }

    /// Delete an item by path from the archive
    ///
    /// # Arguments
    /// * `item_path` - Path of item to delete
    /// * `policy` - Delete policy (ItemOnly or RecurseDirs)
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(Bit7zError)` - Error if path is invalid
    pub fn delete_item_by_path(&mut self, item_path: &str, policy: DeletePolicy) -> Result<()> {
        // Find item by path
        let reader = BitArchiveReader::new(self.library, self.format.into());
        let mut reader_owned = reader;
        reader_owned.open(&self.source_archive)?;
        
        let items = reader_owned.items()?;
        
        // Find the first item with matching path
        let index = items
            .iter()
            .find(|item| item.path == *item_path)
            .map(|item| item.index)
            .ok_or_else(|| Bit7zError::UnknownError(
                format!("Item with path '{}' not found", item_path)
            ))?;
        
        self.delete_item(index, policy)
    }

    /// Apply all changes to the archive
    ///
    /// This creates a new archive with all changes applied, then replaces
    /// the original archive.
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(Bit7zError)` - Error if operation fails
    pub fn apply_changes(&mut self) -> Result<()> {
        // Create a temporary file path for the new archive
        let temp_path = std::env::temp_dir().join(format!(
            "bit7z_archive_{}_{}.tmp",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?
                .as_millis()
        ));
        
        self.temp_archive_path = Some(temp_path.clone());

        let work_dir = std::env::temp_dir().join(format!(
            "bit7z_edit_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?
                .as_millis()
        ));
        if work_dir.exists() {
            std::fs::remove_dir_all(&work_dir).map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;
        }
        std::fs::create_dir_all(&work_dir)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;

        let mut reader = BitArchiveReader::new(self.library, self.format.into());
        reader.open(&self.source_archive)?;
        let existing_items = reader.items()?;

        let extractor = BitExtractor::new(self.library, self.format.into());
        extractor.extract(&self.source_archive, &work_dir)?;

        self.apply_changes_to_dir(&work_dir, &existing_items)?;

        let mut compressor = BitCompressor::new(self.library, self.format);
        compressor
            .compression_level(self.compression_level)
            .solid(self.solid);

        if let Some(method) = self.compression_method {
            compressor.compression_method(method);
        }

        if let Some(size) = self.dictionary_size {
            compressor.dictionary_size(size);
        }

        if let Some(size) = self.word_size {
            compressor.word_size(size);
        }

        if let Some(ref password) = self.password {
            compressor.password(password.clone());
        }

        let files_with_aliases = self.collect_files_with_aliases(&work_dir)?;
        compressor.compress_with_aliases(&files_with_aliases, temp_path.clone())?;

        // Replace original archive with temporary archive
        std::fs::rename(&temp_path, &self.source_archive)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;
        
        self.temp_archive_path = None;
        let _ = std::fs::remove_dir_all(&work_dir);

        Ok(())
    }

    /// Apply changes and return the modified archive data as a buffer
    ///
    /// This method applies all changes and returns the resulting archive data
    /// without replacing the original archive.
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - Modified archive data
    /// * `Err(Bit7zError)` - Error if operation fails
    pub fn apply_changes_to_buffer(&mut self) -> Result<Vec<u8>> {
        // Create a temporary file path for the new archive
        let temp_path = std::env::temp_dir().join(format!(
            "bit7z_archive_buffer_{}_{}.tmp",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?
                .as_millis()
        ));
        
        let work_dir = std::env::temp_dir().join(format!(
            "bit7z_edit_buffer_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?
                .as_millis()
        ));
        if work_dir.exists() {
            std::fs::remove_dir_all(&work_dir).map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;
        }
        std::fs::create_dir_all(&work_dir)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;

        let mut reader = BitArchiveReader::new(self.library, self.format.into());
        reader.open(&self.source_archive)?;
        let existing_items = reader.items()?;

        let extractor = BitExtractor::new(self.library, self.format.into());
        extractor.extract(&self.source_archive, &work_dir)?;

        self.apply_changes_to_dir(&work_dir, &existing_items)?;

        let mut compressor = BitCompressor::new(self.library, self.format);
        compressor
            .compression_level(self.compression_level)
            .solid(self.solid);

        if let Some(method) = self.compression_method {
            compressor.compression_method(method);
        }

        if let Some(size) = self.dictionary_size {
            compressor.dictionary_size(size);
        }

        if let Some(size) = self.word_size {
            compressor.word_size(size);
        }

        if let Some(ref password) = self.password {
            compressor.password(password.clone());
        }

        let files_with_aliases = self.collect_files_with_aliases(&work_dir)?;
        compressor.compress_with_aliases(&files_with_aliases, temp_path.clone())?;

        // Read the compressed data
        let buffer = std::fs::read(&temp_path)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);
        let _ = std::fs::remove_dir_all(&work_dir);

        Ok(buffer)
    }

    fn apply_changes_to_dir(
        &self,
        work_dir: &Path,
        items: &[crate::archive_reader::ArchiveItem],
    ) -> Result<()> {
        let mut index_to_path = HashMap::new();
        for item in items {
            index_to_path.insert(item.index, item.path.clone());
        }

        for index in &self.deleted_indices {
            if let Some(path) = index_to_path.get(index) {
                let target = work_dir.join(path);
                if target.is_dir() {
                    let _ = std::fs::remove_dir_all(&target);
                } else {
                    let _ = std::fs::remove_file(&target);
                }
            }
        }

        for (index, new_path) in &self.renamed_items {
            if let Some(old_path) = index_to_path.get(index) {
                let from = work_dir.join(old_path);
                let to = work_dir.join(new_path);
                if from.exists() {
                    if let Some(parent) = to.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;
                    }
                    std::fs::rename(&from, &to)
                        .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;
                }
            }
        }

        for (index, input_item) in &self.edited_items {
            let target_rel = if let Some(rename) = self.renamed_items.get(index) {
                rename.clone()
            } else if let Some(ref name) = input_item.name_in_archive {
                name.clone()
            } else {
                index_to_path
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| {
                        input_item
                            .path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default()
                    })
            };

            let target_path = work_dir.join(&target_rel);
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;
            }
            std::fs::copy(&input_item.path, &target_path)
                .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;

            if let Some(old_path) = index_to_path.get(index) {
                if old_path != &target_rel {
                    let old = work_dir.join(old_path);
                    let _ = std::fs::remove_file(&old);
                }
            }
        }

        Ok(())
    }

    fn collect_files_with_aliases(
        &self,
        root: &Path,
    ) -> Result<Vec<(PathBuf, String)>> {
        let mut files = Vec::new();
        self.collect_files_recursive(root, root, &mut files)?;
        Ok(files)
    }

    fn collect_files_recursive(
        &self,
        root: &Path,
        current: &Path,
        files: &mut Vec<(PathBuf, String)>,
    ) -> Result<()> {
        for entry in std::fs::read_dir(current)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))? {
            let entry = entry.map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;
            let path = entry.path();
            if path.is_dir() {
                self.collect_files_recursive(root, &path, files)?;
            } else if path.is_file() {
                let relative = path.strip_prefix(root).unwrap_or(&path);
                let alias = relative.to_string_lossy().replace('\\', "/");
                files.push((path, alias));
            }
        }
        Ok(())
    }

    /// Validate an item index
    fn validate_index(&self, index: u32) -> Result<()> {
        let reader = BitArchiveReader::new(self.library, self.format.into());
        let mut reader_owned = reader;
        reader_owned.open(&self.source_archive)?;
        
        let items_count = reader_owned.items_count()?;
        
        if index >= items_count {
            return Err(Bit7zError::UnknownError(
                format!("Invalid item index: {} (archive has {} items)", index, items_count)
            ));
        }
        
        Ok(())
    }

    /// Delete children of a directory item
    fn delete_children(&mut self, dir_index: u32) -> Result<()> {
        let reader = BitArchiveReader::new(self.library, self.format.into());
        let mut reader_owned = reader;
        reader_owned.open(&self.source_archive)?;
        
        let items = reader_owned.items()?;
        
        // Get the directory path
        let dir_item = items.iter()
            .find(|i| i.index == dir_index)
            .ok_or_else(|| Bit7zError::UnknownError(
                format!("Directory item {} not found", dir_index)
            ))?;
        
        let dir_path = dir_item.path.clone();
        
        // Find all items that start with this directory path
        for item in items {
            if item.index != dir_index && item.path.starts_with(&dir_path) {
                self.deleted_indices.insert(item.index);
            }
        }
        
        Ok(())
    }
}

impl<'a> Drop for BitArchiveEditor<'a> {
    fn drop(&mut self) {
        // Clean up temporary files
        if let Some(ref temp_path) = self.temp_archive_path {
            let _ = std::fs::remove_file(temp_path);
        }
    }
}
