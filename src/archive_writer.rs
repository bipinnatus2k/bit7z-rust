//! Archive writer for creating and updating existing archives
//!
//! This module provides BitArchiveWriter for modifying existing archives
//! by adding, renaming, or deleting items.

use crate::ffi::BitLibrary;
use crate::format::CompressionFormat;
use crate::error::{Bit7zError, Result};
use crate::output_archive::{BitOutputArchive, UpdateMode};
use std::path::Path;

/// Archive writer for creating and updating archives
/// 
/// This is a high-level API that combines `BitCompressor` and `BitOutputArchive`
/// for convenient archive creation and updating.
pub struct BitArchiveWriter<'a> {
    inner: BitOutputArchive<'a>,
    library: &'a BitLibrary,
}

impl<'a> BitArchiveWriter<'a> {
    /// Create a new archive writer for creating a new archive
    pub fn new(library: &'a BitLibrary, format: CompressionFormat) -> Self {
        BitArchiveWriter {
            inner: BitOutputArchive::new(format),
            library,
        }
    }

    /// Create a new archive writer for updating an existing archive
    pub fn from_archive<P: AsRef<Path>>(
        library: &'a BitLibrary,
        format: CompressionFormat,
        archive_path: P,
    ) -> Self {
        BitArchiveWriter {
            inner: BitOutputArchive::from_archive(format, archive_path),
            library,
        }
    }

    /// Set password for encrypted archives
    pub fn password(&mut self, password: impl Into<String>) -> &mut Self {
        self.inner.password(password);
        self
    }

    /// Set compression level
    pub fn compression_level(&mut self, level: crate::format::CompressionLevel) -> &mut Self {
        self.inner.compression_level(level);
        self
    }

    /// Set compression method
    pub fn compression_method(&mut self, method: crate::format::CompressionMethod) -> &mut Self {
        self.inner.compression_method(method);
        self
    }

    /// Set dictionary size
    pub fn dictionary_size(&mut self, size: u32) -> &mut Self {
        self.inner.dictionary_size(size);
        self
    }

    /// Set word size
    pub fn word_size(&mut self, size: u32) -> &mut Self {
        self.inner.word_size(size);
        self
    }

    /// Set solid compression mode
    pub fn solid(&mut self, solid: bool) -> &mut Self {
        self.inner.solid(solid);
        self
    }

    /// Set update mode for existing archives
    pub fn update_mode(&mut self, mode: UpdateMode) -> &mut Self {
        self.inner.update_mode(mode);
        self
    }

    /// Add a file to the archive
    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.inner.add_file(path);
        self
    }

    /// Add a file with custom name in archive
    pub fn add_file_with_name<P: AsRef<Path>>(&mut self, path: P, name: String) -> &mut Self {
        self.inner.add_file_with_name(path, name);
        self
    }

    /// Add multiple files
    pub fn add_files<P: AsRef<Path>, I: IntoIterator<Item = P>>(&mut self, paths: I) -> &mut Self {
        self.inner.add_files(paths);
        self
    }

    /// Add directory contents recursively
    pub fn add_directory<P: AsRef<Path>>(&mut self, dir_path: P) -> Result<()> {
        self.inner.add_directory(dir_path)
    }

    /// Delete an item from the archive (for update operations)
    pub fn delete_item(&mut self, index: u32) -> &mut Self {
        self.inner.delete_item(index);
        self
    }

    /// Get the number of items to be added
    pub fn items_count(&self) -> usize {
        self.inner.items_count()
    }

    /// Get total uncompressed size of items
    pub fn total_size(&self) -> u64 {
        self.inner.total_size()
    }

    /// Compress to a file
    pub fn compress_to<P: AsRef<Path>>(&self, output_path: P) -> Result<()> {
        self.inner.compress_to(output_path)
    }
}

