//! Output archive management for creating and updating archives
//!
//! This module provides BitOutputArchive for managing archive creation operations.

use crate::ffi::{
    IOutArchive, ISequentialOutStream, IArchiveUpdateCallback,
};
use crate::format::CompressionFormat;
use crate::error::{Bit7zError, Result};
use crate::stream::FileStreamWrite;
use crate::compress_callback::InputItem;
use crate::archive_reader::ArchiveItem;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::collections::HashMap;

/// Update mode for archive operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateMode {
    /// Fail if output archive already exists
    None,
    /// Append new items to existing archive
    Append,
    /// Update existing items with same names
    Update,
}

impl Default for UpdateMode {
    fn default() -> Self {
        UpdateMode::None
    }
}

/// Overwrite mode for output files
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverwriteMode {
    /// Fail if output file already exists
    None,
    /// Overwrite existing file
    Overwrite,
    /// Skip compression if file exists
    Skip,
}

impl Default for OverwriteMode {
    fn default() -> Self {
        OverwriteMode::None
    }
}

/// Output archive item representing a file to be added
#[derive(Debug, Clone)]
pub struct OutputItem {
    /// Path to the file on filesystem
    pub path: PathBuf,
    /// Name inside the archive (optional, uses filename if None)
    pub name_in_archive: Option<String>,
    /// Whether this is a directory
    pub is_dir: bool,
}

impl OutputItem {
    /// Create a new output item from a path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_path_buf();
        let is_dir = path.is_dir();
        OutputItem {
            path,
            name_in_archive: None,
            is_dir,
        }
    }

    /// Create a new output item with custom archive name
    pub fn with_name<P: AsRef<Path>>(path: P, name: String) -> Self {
        let path = path.as_ref().to_path_buf();
        let is_dir = path.is_dir();
        OutputItem {
            path,
            name_in_archive: Some(name),
            is_dir,
        }
    }
}

/// Output archive manager for creating and updating archives
pub struct BitOutputArchive<'a> {
    /// Compression format
    format: CompressionFormat,
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
    /// Store symbolic links as links (true) or follow them (false)
    store_symbolic_links: bool,
    /// Update mode
    update_mode: UpdateMode,
    /// Overwrite mode
    overwrite_mode: OverwriteMode,
    /// Volume size in bytes (0 = single volume)
    volume_size: u64,
    /// Items to add to the archive
    items: Vec<OutputItem>,
    /// Indices of items to delete (for update operations)
    deleted_indices: HashSet<u32>,
    /// Path to existing archive (for update operations)
    existing_archive_path: Option<PathBuf>,
    /// Items from existing archive (for update operations)
    existing_items: Vec<ArchiveItem>,
    /// Phantom data for lifetime
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> BitOutputArchive<'a> {
    /// Create a new output archive for creating a new archive
    pub fn new(format: CompressionFormat) -> Self {
        BitOutputArchive {
            format,
            password: None,
            compression_level: crate::format::CompressionLevel::Normal,
            compression_method: None,
            dictionary_size: None,
            word_size: None,
            solid: false,
            store_symbolic_links: false, // Default: follow symlinks
            update_mode: UpdateMode::None,
            overwrite_mode: OverwriteMode::None,
            volume_size: 0, // Single volume by default
            items: Vec::new(),
            deleted_indices: HashSet::new(),
            existing_archive_path: None,
            existing_items: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a new output archive for updating an existing archive
    pub fn from_archive<P: AsRef<Path>>(format: CompressionFormat, archive_path: P) -> Self {
        BitOutputArchive {
            format,
            password: None,
            compression_level: crate::format::CompressionLevel::Normal,
            compression_method: None,
            dictionary_size: None,
            word_size: None,
            solid: false,
            store_symbolic_links: false,
            update_mode: UpdateMode::Append,
            overwrite_mode: OverwriteMode::Overwrite,
            volume_size: 0,
            items: Vec::new(),
            deleted_indices: HashSet::new(),
            existing_archive_path: Some(archive_path.as_ref().to_path_buf()),
            existing_items: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Set password for encryption
    pub fn password(&mut self, password: impl Into<String>) -> &mut Self {
        self.password = Some(password.into());
        self
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

    /// Set whether to store symbolic links as links or follow them
    pub fn store_symbolic_links(&mut self, store: bool) -> &mut Self {
        self.store_symbolic_links = store;
        self
    }

    /// Set update mode
    pub fn update_mode(&mut self, mode: UpdateMode) -> &mut Self {
        self.update_mode = mode;
        self
    }

    /// Set overwrite mode
    pub fn overwrite_mode(&mut self, mode: OverwriteMode) -> &mut Self {
        self.overwrite_mode = mode;
        self
    }

    /// Set volume size for multi-volume archives (0 = single volume)
    pub fn volume_size(&mut self, size: u64) -> &mut Self {
        self.volume_size = size;
        self
    }

    /// Add a file to the archive
    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.items.push(OutputItem::from_path(path));
        self
    }

    /// Add a file with custom name in archive
    pub fn add_file_with_name<P: AsRef<Path>>(&mut self, path: P, name: String) -> &mut Self {
        self.items.push(OutputItem::with_name(path, name));
        self
    }

    /// Add multiple files
    pub fn add_files<P: AsRef<Path>, I: IntoIterator<Item = P>>(&mut self, paths: I) -> &mut Self {
        for path in paths {
            self.items.push(OutputItem::from_path(path));
        }
        self
    }

    /// Add directory contents recursively
    pub fn add_directory<P: AsRef<Path>>(&mut self, dir_path: P) -> Result<()> {
        let dir_path = dir_path.as_ref();
        if !dir_path.exists() {
            return Err(Bit7zError::CompressFailed(
                format!("Directory does not exist: {:?}", dir_path)
            ));
        }

        if !dir_path.is_dir() {
            return Err(Bit7zError::CompressFailed(
                format!("Not a directory: {:?}", dir_path)
            ));
        }

        // Walk directory recursively
        self.walk_directory(dir_path, dir_path);
        Ok(())
    }

    /// Walk directory recursively
    fn walk_directory(&mut self, root: &Path, current: &Path) {
        if let Ok(entries) = std::fs::read_dir(current) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                // Handle symbolic links
                if path.is_symlink() {
                    if self.store_symbolic_links {
                        // Store the symlink itself
                        if let Ok(relative_path) = path.strip_prefix(root) {
                            let name = relative_path.to_string_lossy().replace('\\', "/");
                            self.items.push(OutputItem::with_name(path, name));
                        }
                    } else {
                        // Follow the symlink and add the target
                        if let Ok(target_path) = std::fs::read_link(&path) {
                            if target_path.is_dir() {
                                self.walk_directory(root, &target_path);
                            } else if target_path.is_file() {
                                if let Ok(relative_path) = path.strip_prefix(root) {
                                    let name = relative_path.to_string_lossy().replace('\\', "/");
                                    self.items.push(OutputItem::with_name(path, name));
                                }
                            }
                        }
                    }
                } else if path.is_dir() {
                    // Recursively walk subdirectories
                    self.walk_directory(root, &path);
                } else if path.is_file() {
                    // Add file with relative path from root
                    if let Ok(relative_path) = path.strip_prefix(root) {
                        let name = relative_path.to_string_lossy().replace('\\', "/");
                        self.items.push(OutputItem::with_name(path, name));
                    }
                }
            }
        }
    }

    /// Delete an item from the archive (for update operations)
    pub fn delete_item(&mut self, index: u32) -> &mut Self {
        self.deleted_indices.insert(index);
        self
    }

    /// Get the number of items to be added
    pub fn items_count(&self) -> usize {
        self.items.len()
    }

    /// Get total uncompressed size of items
    pub fn total_size(&self) -> u64 {
        self.items.iter()
            .filter(|item| !item.is_dir)
            .filter_map(|item| std::fs::metadata(&item.path).ok())
            .map(|meta| meta.len())
            .sum()
    }

    /// Load items from existing archive
    pub fn load_existing_items(&mut self) -> Result<()> {
        if let Some(ref archive_path) = self.existing_archive_path {
            use crate::archive_reader::BitArchiveReader;
            use crate::ffi::BitLibrary;
            
            // Load the library
            let lib = BitLibrary::new(None::<&str>)
                .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;
            
            // Create a reader to read the existing archive
            let mut reader = BitArchiveReader::new(&lib, self.format.into());
            reader.open(archive_path)?;
            
            // Get items from the archive
            self.existing_items = reader.items()?;
        }
        Ok(())
    }

    /// Get items from existing archive
    pub fn existing_items(&self) -> &[ArchiveItem] {
        &self.existing_items
    }

    /// Prepare items for update operation (Append or Update mode)
    fn prepare_update_items(&self) -> Vec<InputItem> {
        let mut input_items = Vec::new();
        
        match self.update_mode {
            UpdateMode::None => {
                // Just add new items
                for item in &self.items {
                    if let Some(ref name) = item.name_in_archive {
                        input_items.push(InputItem::with_name(&item.path, name.clone()));
                    } else {
                        input_items.push(InputItem::new(&item.path));
                    }
                }
            }
            UpdateMode::Append => {
                // For Append mode, we need to:
                // 1. Keep all existing items (not deleted)
                // 2. Add new items
                
                // First, add existing items that are not deleted
                // Note: This requires special handling in UpdateCallback
                // For now, we just add new items - full implementation needs
                // ExtendedInputItem support
                for item in &self.items {
                    if let Some(ref name) = item.name_in_archive {
                        input_items.push(InputItem::with_name(&item.path, name.clone()));
                    } else {
                        input_items.push(InputItem::new(&item.path));
                    }
                }
            }
            UpdateMode::Update => {
                // For Update mode, we need to:
                // 1. Replace existing items with same name
                // 2. Keep existing items with no match
                // 3. Add new items with no match
                
                // Build a map of new items by name
                let mut new_items_map: HashMap<String, &OutputItem> = HashMap::new();
                for item in &self.items {
                    let name = item.name_in_archive.clone()
                        .unwrap_or_else(|| {
                            item.path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default()
                        });
                    new_items_map.insert(name.clone(), item);
                }
                
                // Process existing items
                for existing_item in &self.existing_items {
                    if self.deleted_indices.contains(&(existing_item.index)) {
                        continue; // Skip deleted items
                    }
                    
                    // Check if we have a new item with the same name
                    if let Some(new_item) = new_items_map.get(&existing_item.path) {
                        // Use new item (update)
                        if let Some(ref name) = new_item.name_in_archive {
                            input_items.push(InputItem::with_name(&new_item.path, name.clone()));
                        } else {
                            input_items.push(InputItem::new(&new_item.path));
                        }
                        new_items_map.remove(&existing_item.path);
                    }
                    // Note: Keeping existing items requires special handling
                    // For now, we only include updated/new items
                }
                
                // Add remaining new items (not in existing archive)
                for item in new_items_map.values() {
                    if let Some(ref name) = item.name_in_archive {
                        input_items.push(InputItem::with_name(&item.path, name.clone()));
                    } else {
                        input_items.push(InputItem::new(&item.path));
                    }
                }
            }
        }
        
        input_items
    }

    /// Compress to a file
    pub fn compress_to<P: AsRef<Path>>(&self, output_path: P) -> Result<()> {
        let output_path = output_path.as_ref();
        
        // Check overwrite mode
        if output_path.exists() {
            match self.overwrite_mode {
                OverwriteMode::None => {
                    return Err(Bit7zError::CompressFailed(
                        format!("Output file already exists: {:?}", output_path)
                    ));
                }
                OverwriteMode::Skip => {
                    // Skip compression, return success
                    return Ok(());
                }
                OverwriteMode::Overwrite => {
                    // Continue with compression, file will be overwritten
                }
            }
        }

        // Create output stream
        let out_stream = FileStreamWrite::new(output_path)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;

        // Prepare input items based on update mode
        let input_items = self.prepare_update_items();

        // Use internal compression logic
        self.compress_internal(&input_items, &out_stream)
    }

    /// Compress to a memory buffer
    pub fn compress_to_buffer(&self) -> Result<Vec<u8>> {
        
        
        // Create a temporary file for output
        let temp_path = std::env::temp_dir().join(format!(
            "bit7z_output_{}.tmp",
            std::process::id()
        ));

        {
            let out_stream = FileStreamWrite::new(&temp_path)
                .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;

            // Prepare input items based on update mode
            let input_items = self.prepare_update_items();

            // Use internal compression logic
            self.compress_internal(&input_items, &out_stream)?;
        }

        // Read the compressed data
        let buffer = std::fs::read(&temp_path)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);

        Ok(buffer)
    }

    /// Compress to a writer stream
    pub fn compress_to_stream<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        
        
        // Create a temporary file for output
        let temp_path = std::env::temp_dir().join(format!(
            "bit7z_output_stream_{}.tmp",
            std::process::id()
        ));

        {
            let out_stream = FileStreamWrite::new(&temp_path)
                .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;

            // Prepare input items based on update mode
            let input_items = self.prepare_update_items();

            // Use internal compression logic
            self.compress_internal(&input_items, &out_stream)?;
        }

        // Read the compressed data and write to the output stream
        let mut temp_file = std::fs::File::open(&temp_path)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;
        
        std::io::copy(&mut temp_file, writer)
            .map_err(|e| Bit7zError::CompressFailed(e.to_string()))?;

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);

        Ok(())
    }

    /// Internal compression implementation
    fn compress_internal(
        &self,
        input_items: &[InputItem],
        out_stream: &FileStreamWrite,
    ) -> Result<()> {
        use crate::ffi::BitLibrary;

        use crate::compress_callback::UpdateCallback;


        // Load 7-Zip library
        let lib = match BitLibrary::new(None::<&str>) {
            Ok(l) => l,
            Err(e) => return Err(Bit7zError::CompressFailed(format!("Failed to load 7-Zip library: {}", e))),
        };

        unsafe {
            // Create output archive object
            let format_guid = self.format.info().guid;
            let archive_ptr = match lib.create_out_archive(&format_guid) {
                Ok(ptr) => ptr,
                Err(e) => return Err(Bit7zError::CompressFailed(format!("Failed to create archive: {}", e))),
            };

            let archive = archive_ptr.as_ptr();

            // Set archive properties
            eprintln!("[output_archive] Setting archive properties");
            self.set_archive_properties(archive)?;
            eprintln!("[output_archive] Archive properties set");

            // Create update callback on the heap
            // We need to use Box::new and Box::into_raw because the COM interface
            // uses reference counting and may call release after update_items returns
            let update_callback = UpdateCallback::new(
                input_items.to_vec(),
                self.password.clone(),
            );
            let callback_box = Box::new(update_callback);
            let callback_ptr = Box::into_raw(callback_box);

            // Call UpdateItems
            let num_items = input_items.len() as u32;
            let archive_vtable = &*(*archive).vtable;
            let result = (archive_vtable.update_items)(
                archive,
                out_stream.as_i_out_stream() as *mut ISequentialOutStream,
                num_items,
                callback_ptr as *mut IArchiveUpdateCallback,
            );

            // After update_items returns, release our reference
            // The callback will be freed when ref_count reaches 0
            UpdateCallback::release_caller_reference(callback_ptr);

            // S_OK (0) and S_FALSE (1) are both success codes
            if result != 0 && result != 1 {
                return Err(Bit7zError::CompressFailed(
                    format!("UpdateItems failed with HRESULT: 0x{:X}", result)
                ));
            }
        }

        Ok(())
    }

    /// Set archive properties
    unsafe fn set_archive_properties(&self, archive: *mut IOutArchive) -> Result<()> {
        use crate::ffi::{IID_ISetProperties, ISetProperties, IUnknown, PROPVARIANT, VARENUM};
        use crate::ffi::variant::{alloc_bstr_from_utf32, free_bstr};
        use std::ptr;

        eprintln!("[set_archive_properties] Starting");

        // Try to get ISetProperties interface
        let mut set_props_ptr: *mut std::ffi::c_void = ptr::null_mut();
        let iid = IID_ISetProperties;

        let unknown = archive as *mut IUnknown;
        let archive_vtable = &*(*archive).vtable;
        eprintln!("[set_archive_properties] Calling QueryInterface");
        let result = (archive_vtable.base.query_interface)(
            unknown,
            &iid,
            &mut set_props_ptr,
        );
        eprintln!("[set_archive_properties] QueryInterface result: 0x{:X}, ptr: {:p}", result, set_props_ptr);

        if result != 0 || set_props_ptr.is_null() {
            // ISetProperties not supported, skip property setting
            return Ok(());
        }

        let set_properties: *mut ISetProperties = set_props_ptr as *mut ISetProperties;

        // Build property names and values
        let mut prop_names: Vec<*const u16> = Vec::new();
        let mut prop_values: Vec<PROPVARIANT> = Vec::new();

        // Add compression level property ("x")
        {
            let name_bstr = alloc_bstr_from_utf32("x");
            if !name_bstr.is_null() {
                prop_names.push(name_bstr);
                let mut prop_value = PROPVARIANT::default();
                prop_value.vt = VARENUM::VT_UI4 as u16;
                let value = self.compression_level.to_value();
                prop_value.data[0] = (value & 0xFF) as u8;
                prop_value.data[1] = ((value >> 8) & 0xFF) as u8;
                prop_value.data[2] = ((value >> 16) & 0xFF) as u8;
                prop_value.data[3] = ((value >> 24) & 0xFF) as u8;
                prop_values.push(prop_value);
            }
        }

        // Add compression method property ("m") if specified
        if let Some(method) = self.compression_method {
            let method_str = method.to_string();
            let name_bstr = alloc_bstr_from_utf32("m");
            if !name_bstr.is_null() {
                let value_bstr = alloc_bstr_from_utf32(&method_str);
                if !value_bstr.is_null() {
                    prop_names.push(name_bstr);
                    let mut prop_value = PROPVARIANT::default();
                    prop_value.vt = VARENUM::VT_BSTR as u16;
                    // Use write_unaligned to avoid alignment issues
                    let data_ptr = prop_value.data.as_mut_ptr() as *mut *mut u16;
                    std::ptr::write_unaligned(data_ptr, value_bstr);
                    prop_values.push(prop_value);
                } else {
                    free_bstr(name_bstr as *mut u16);
                }
            }
        }

        // Add dictionary size property ("d") if specified
        if let Some(size) = self.dictionary_size {
            let name_bstr = alloc_bstr_from_utf32("d");
            if !name_bstr.is_null() {
                prop_names.push(name_bstr);
                let mut prop_value = PROPVARIANT::default();
                prop_value.vt = VARENUM::VT_UI4 as u16;
                prop_value.data[0] = (size & 0xFF) as u8;
                prop_value.data[1] = ((size >> 8) & 0xFF) as u8;
                prop_value.data[2] = ((size >> 16) & 0xFF) as u8;
                prop_value.data[3] = ((size >> 24) & 0xFF) as u8;
                prop_values.push(prop_value);
            }
        }

        // Add word size property ("w") if specified
        if let Some(size) = self.word_size {
            let name_bstr = alloc_bstr_from_utf32("w");
            if !name_bstr.is_null() {
                prop_names.push(name_bstr);
                let mut prop_value = PROPVARIANT::default();
                prop_value.vt = VARENUM::VT_UI4 as u16;
                prop_value.data[0] = (size & 0xFF) as u8;
                prop_value.data[1] = ((size >> 8) & 0xFF) as u8;
                prop_value.data[2] = ((size >> 16) & 0xFF) as u8;
                prop_value.data[3] = ((size >> 24) & 0xFF) as u8;
                prop_values.push(prop_value);
            }
        }

        // Add solid compression property ("s") if enabled
        if self.solid {
            let name_bstr = alloc_bstr_from_utf32("s");
            if !name_bstr.is_null() {
                prop_names.push(name_bstr);
                let mut prop_value = PROPVARIANT::default();
                prop_value.vt = VARENUM::VT_BOOL as u16;
                prop_value.data[0] = 0xFF;
                prop_value.data[1] = 0xFF;
                prop_values.push(prop_value);
            }
        }

        // Call SetProperties if we have any properties
        if !prop_names.is_empty() && !prop_values.is_empty() {
            let set_vtable = &*(*set_properties).vtable;
            let names_ptr = prop_names.as_ptr();
            let values_ptr = prop_values.as_ptr();
            let num_props = prop_names.len() as u32;

            let set_result = (set_vtable.set_properties)(
                set_properties,
                names_ptr,
                values_ptr as *const *const std::ffi::c_void,
                num_props,
            );

            if set_result != 0 && set_result != 1 {
                eprintln!("[WARN] SetProperties returned 0x{:X}", set_result);
            }
        }

        // Release the interface
        let set_vtable = &*(*set_properties).vtable;
        (set_vtable.base.release)(set_properties as *mut IUnknown);

        // Free allocated BSTRs (property names)
        for name in prop_names {
            free_bstr(name as *mut u16);
        }

        // Free BSTRs in property values (VT_BSTR only)
        for value in prop_values {
            if value.vt == VARENUM::VT_BSTR as u16 {
                // Use read_unaligned to avoid alignment issues
                let data_ptr = value.data.as_ptr() as *const *const u16;
                let bstr = std::ptr::read_unaligned(data_ptr);
                if !bstr.is_null() {
                    free_bstr(bstr as *mut u16);
                }
            }
        }

        Ok(())
    }
}
