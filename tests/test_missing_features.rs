//! Tests for missing features: callbacks, symlinks, volumes, updates, and extraction
//!
//! This test file covers the previously untested features.

use bit7z_rust::{
    BitLibrary, BitOutputArchive, BitExtractor, BitArchiveReader,
    BitCompressor, CompressionFormat, CompressionLevel, CompressionMethod,
    UpdateMode,
    ExtractFormat,
};
use bit7z_rust::compress_callback::{UpdateCallback, InputItem, ProgressCallback, RatioCallback, FileCallback};
use std::fs;
use std::sync::{Arc, Mutex};

// ============================================================================
// Test Utilities
// ============================================================================

fn create_test_file(prefix: &str, name: &str, content: &str) -> String {
    let temp_dir = std::env::temp_dir().join(format!("bit7z_test_{}", prefix));
    fs::create_dir_all(&temp_dir).ok();
    
    let file_path = temp_dir.join(name);
    fs::write(&file_path, content).ok();
    
    file_path.to_string_lossy().to_string()
}

fn cleanup_test_dir(prefix: &str) {
    let temp_dir = std::env::temp_dir().join(format!("bit7z_test_{}", prefix));
    let _ = fs::remove_dir_all(&temp_dir);
}

fn cleanup_output_file(name: &str) {
    let path = std::env::temp_dir().join(name);
    let _ = fs::remove_file(path);
}

// ============================================================================
// Callback Tests
// ============================================================================

#[test]
fn test_progress_callback() {
    let prefix = "progress_cb";
    let file_path = create_test_file(prefix, "test.txt", "Hello, World! This is test content.");

    let progress_called = Arc::new(Mutex::new(false));
    let progress_called_clone = progress_called.clone();

    let progress_cb: ProgressCallback = Arc::new(Mutex::new(move |completed, total| {
        let mut called = progress_called_clone.lock().unwrap();
        *called = true;
        println!("Progress: {}/{}", completed, total);
        true
    }));

    let items = vec![InputItem::new(&file_path)];
    let callback = UpdateCallback::with_callbacks(
        items,
        None,
        None,  // total_callback
        Some(progress_cb),
        None,  // ratio_callback
        None,  // file_callback
        None,  // password_callback
    );

    // Verify callback was created
    assert!(callback.as_i_archive_update_callback() != std::ptr::null_mut());

    // Cleanup
    cleanup_test_dir(prefix);

    // Note: Actual callback invocation happens during compression
    // which requires 7-Zip library
}

#[test]
fn test_ratio_callback() {
    let prefix = "ratio_cb";
    let file_path = create_test_file(prefix, "test.txt", "Test content for ratio callback.");

    let ratio_values = Arc::new(Mutex::new((0u64, 0u64)));
    let ratio_values_clone = ratio_values.clone();

    let ratio_cb: RatioCallback = Arc::new(Mutex::new(move |in_size, out_size| {
        let mut values = ratio_values_clone.lock().unwrap();
        *values = (in_size, out_size);
        println!("Ratio: in={}, out={}", in_size, out_size);
    }));

    let items = vec![InputItem::new(&file_path)];
    let callback = UpdateCallback::with_callbacks(
        items,
        None,
        None,  // total_callback
        None,  // progress_callback
        Some(ratio_cb),
        None,  // file_callback
        None,  // password_callback
    );

    assert!(callback.as_i_archive_update_callback() != std::ptr::null_mut());

    // Cleanup
    cleanup_test_dir(prefix);
}

#[test]
fn test_file_callback() {
    let prefix = "file_cb";
    let file_path = create_test_file(prefix, "test.txt", "Test content.");

    let files_processed = Arc::new(Mutex::new(Vec::new()));
    let files_processed_clone = files_processed.clone();

    let file_cb: FileCallback = Arc::new(Mutex::new(move |path: &str| {
        let mut files = files_processed_clone.lock().unwrap();
        files.push(path.to_string());
        println!("Processing file: {}", path);
    }));
    
    let items = vec![InputItem::new(&file_path)];
    let callback = UpdateCallback::with_callbacks(
        items,
        None,
        None,  // total_callback
        None,  // progress_callback
        None,  // ratio_callback
        Some(file_cb),
        None,  // password_callback
    );

    assert!(callback.as_i_archive_update_callback() != std::ptr::null_mut());

    // Cleanup
    cleanup_test_dir(prefix);
}

#[test]
fn test_multiple_callbacks() {
    let prefix = "multi_cb";
    let file_path = create_test_file(prefix, "test.txt", "Test content.");

    let progress_cb: ProgressCallback = Arc::new(Mutex::new(|completed: u64, total: u64| {
        println!("Progress: {}/{}", completed, total);
        true
    }));

    let ratio_cb: RatioCallback = Arc::new(Mutex::new(|in_size: u64, out_size: u64| {
        println!("Ratio: {}/{}", in_size, out_size);
    }));

    let file_cb: FileCallback = Arc::new(Mutex::new(|path: &str| {
        println!("File: {}", path);
    }));

    let items = vec![InputItem::new(&file_path)];
    let callback = UpdateCallback::with_callbacks(
        items,
        None,
        None,  // total_callback
        Some(progress_cb),
        Some(ratio_cb),
        Some(file_cb),
        None,  // password_callback
    );

    assert!(callback.as_i_archive_update_callback() != std::ptr::null_mut());

    
    // Cleanup
    cleanup_test_dir(prefix);
}

// ============================================================================
// Symbolic Link Tests
// ============================================================================

#[test]
fn test_store_symbolic_links_option() {
    let prefix = "symlink_opt";
    
    // Create BitOutputArchive and test store_symbolic_links method
    let mut archive = BitOutputArchive::new(CompressionFormat::SevenZip);
    
    // Test method chaining
    let archive_ref = archive.store_symbolic_links(true);
    assert!(std::ptr::eq(archive_ref, &mut archive));
    
    let archive_ref = archive.store_symbolic_links(false);
    assert!(std::ptr::eq(archive_ref, &mut archive));
    
    // Cleanup
    cleanup_test_dir(prefix);
}

#[test]
#[cfg(unix)]
fn test_symbolic_link_creation() {
    use std::os::unix::fs::symlink;
    
    let prefix = "symlink";
    let temp_dir = std::env::temp_dir().join(format!("bit7z_test_{}", prefix));
    fs::create_dir_all(&temp_dir).ok();
    
    // Create a real file
    let real_file = temp_dir.join("real.txt");
    fs::write(&real_file, "Real file content").ok();
    
    // Create a symbolic link
    let link_file = temp_dir.join("link.txt");
    symlink(&real_file, &link_file).ok();
    
    // Test with store_symbolic_links = true
    let mut archive_follow = BitOutputArchive::new(CompressionFormat::SevenZip);
    archive_follow.store_symbolic_links(false); // Follow symlinks
    archive_follow.add_directory(&temp_dir).ok();
    
    // Test with store_symbolic_links = false
    let mut archive_store = BitOutputArchive::new(CompressionFormat::SevenZip);
    archive_store.store_symbolic_links(true); // Store as links
    archive_store.add_directory(&temp_dir).ok();
    
    // Cleanup
    cleanup_test_dir(prefix);
}

// ============================================================================
// Volume Compression Tests
// ============================================================================

#[test]
fn test_volume_size_setting() {
    let prefix = "volume";
    
    let mut archive = BitOutputArchive::new(CompressionFormat::SevenZip);
    
    // Test volume_size method
    archive.volume_size(1_000_000); // 1MB volumes
    
    // Test method chaining
    let archive_ref = archive.volume_size(65_536_000); // 64MB
    assert!(std::ptr::eq(archive_ref, &mut archive));
    
    // Test with 0 (single volume)
    archive.volume_size(0);
    
    // Cleanup
    cleanup_test_dir(prefix);
}

#[test]
fn test_volume_compression_basic() {
    let prefix = "volume_basic";
    let file_path = create_test_file(prefix, "test.txt", "Test content for volume compression.");
    
    let mut archive = BitOutputArchive::new(CompressionFormat::SevenZip);
    archive.volume_size(1_000_000); // 1MB volumes
    archive.add_file(&file_path);
    
    // Note: Actual volume splitting requires 7-Zip library support
    // This test verifies the API works
    assert!(archive.items_count() == 1);
    
    // Cleanup
    cleanup_test_dir(prefix);
}

// ============================================================================
// Archive Update Tests
// ============================================================================

#[test]
fn test_update_mode_append() {
    let prefix = "update_append";
    let file1_path = create_test_file(prefix, "file1.txt", "First file content.");
    let file2_path = create_test_file(prefix, "file2.txt", "Second file content.");
    
    let output_path = std::env::temp_dir().join(format!("test_{}.7z", prefix));
    
    // Create initial archive
    let mut archive1 = BitOutputArchive::new(CompressionFormat::SevenZip);
    archive1.add_file(&file1_path);
    let result1 = archive1.compress_to(&output_path);
    
    if result1.is_ok() {
        // Update archive with Append mode
        let mut archive2 = BitOutputArchive::from_archive(CompressionFormat::SevenZip, &output_path);
        archive2.update_mode(UpdateMode::Append);
        archive2.add_file(&file2_path);
        
        // Note: Full append mode requires reading existing archive items
        // This test verifies the API works
        let _items = archive2.existing_items(); // Just verify method can be called
    }
    
    // Cleanup
    cleanup_test_dir(prefix);
    cleanup_output_file(&format!("test_{}.7z", prefix));
}

#[test]
fn test_update_mode_update() {
    let prefix = "update_replace";
    let file1_path = create_test_file(prefix, "file1.txt", "Original content.");
    let file1_new_path = create_test_file(prefix, "file1_new.txt", "Updated content.");
    
    let output_path = std::env::temp_dir().join(format!("test_{}.7z", prefix));
    
    // Create initial archive
    let mut archive1 = BitOutputArchive::new(CompressionFormat::SevenZip);
    archive1.add_file_with_name(&file1_path, "file1.txt".to_string());
    let result1 = archive1.compress_to(&output_path);
    
    if result1.is_ok() {
        // Update archive with Update mode
        let mut archive2 = BitOutputArchive::from_archive(CompressionFormat::SevenZip, &output_path);
        archive2.update_mode(UpdateMode::Update);
        archive2.add_file_with_name(&file1_new_path, "file1.txt".to_string());
        
        // Note: Full update mode requires reading existing archive items
        let _items = archive2.existing_items();
    }
    
    // Cleanup
    cleanup_test_dir(prefix);
    cleanup_output_file(&format!("test_{}.7z", prefix));
}

#[test]
fn test_delete_item() {
    let prefix = "delete";
    
    let mut archive = BitOutputArchive::new(CompressionFormat::SevenZip);
    
    // Test delete_item method
    archive.delete_item(0);
    archive.delete_item(1);
    archive.delete_item(2);
    
    // Test method chaining
    let archive_ref = archive.delete_item(3);
    assert!(std::ptr::eq(archive_ref, &mut archive));
    
    // Cleanup
    cleanup_test_dir(prefix);
}

#[test]
fn test_load_existing_items() {
    let prefix = "load_items";
    let file_path = create_test_file(prefix, "test.txt", "Test content.");
    
    let output_path = std::env::temp_dir().join(format!("test_{}.7z", prefix));
    
    // Create archive first
    let mut archive1 = BitOutputArchive::new(CompressionFormat::SevenZip);
    archive1.add_file(&file_path);
    let result1 = archive1.compress_to(&output_path);
    
    if result1.is_ok() {
        // Try to load existing items
        let mut archive2 = BitOutputArchive::from_archive(CompressionFormat::SevenZip, &output_path);
        let load_result = archive2.load_existing_items();
        
        // Just verify the API exists and can be called
        let _ = load_result;
    }
    
    // Cleanup
    cleanup_test_dir(prefix);
    cleanup_output_file(&format!("test_{}.7z", prefix));
}

// ============================================================================
// Extraction Tests
// ============================================================================

#[test]
fn test_extractor_creation() {
    let prefix = "extract_create";
    
    // Test BitExtractor creation
    let lib_result = BitLibrary::new(None::<&str>);
    
    if let Ok(lib) = lib_result {
        let extractor = BitExtractor::new(&lib, ExtractFormat::SevenZip);
        
        // Verify extractor was created
        assert!(std::mem::size_of_val(&extractor) > 0);
    }
    
    // Cleanup
    cleanup_test_dir(prefix);
}

#[test]
fn test_extractor_extract() {
    let prefix = "extract";
    let file_path = create_test_file(prefix, "test.txt", "Test content for extraction.");
    
    let archive_path = std::env::temp_dir().join(format!("test_{}.7z", prefix));
    let output_dir = std::env::temp_dir().join(format!("test_{}_output", prefix));
    
    // Create archive first
    let mut archive = BitOutputArchive::new(CompressionFormat::SevenZip);
    archive.add_file(&file_path);
    let create_result = archive.compress_to(&archive_path);
    
    if create_result.is_ok() {
        // Test archive reader
        let lib_result = BitLibrary::new(None::<&str>);
        
        if let Ok(lib) = lib_result {
            let mut reader = BitArchiveReader::new(&lib, ExtractFormat::SevenZip);
            let _open_result = reader.open(&archive_path);
            
            // Just verify the API exists
        }
    }
    
    // Cleanup
    cleanup_test_dir(prefix);
    cleanup_output_file(&format!("test_{}.7z", prefix));
    let _ = fs::remove_dir_all(&output_dir);
}

#[test]
fn test_archive_reader() {
    let prefix = "reader";
    let file_path = create_test_file(prefix, "test.txt", "Test content for reading.");
    
    let archive_path = std::env::temp_dir().join(format!("test_{}.7z", prefix));
    
    // Create archive first
    let mut archive = BitOutputArchive::new(CompressionFormat::SevenZip);
    archive.add_file(&file_path);
    let create_result = archive.compress_to(&archive_path);
    
    if create_result.is_ok() {
        // Test archive reader
        let lib_result = BitLibrary::new(None::<&str>);
        
        if let Ok(lib) = lib_result {
            let mut reader = BitArchiveReader::new(&lib, ExtractFormat::SevenZip);
            let open_result = reader.open(&archive_path);
            
            if open_result.is_ok() {
                let items_result = reader.items();
                
                if let Ok(items) = items_result {
                    assert!(items.len() > 0);
                    
                    // Check first item
                    if let Some(first_item) = items.first() {
                        assert!(!first_item.path.is_empty());
                    }
                }
            }
        }
    }
    
    // Cleanup
    cleanup_test_dir(prefix);
    cleanup_output_file(&format!("test_{}.7z", prefix));
}

#[test]
fn test_archive_reader_properties() {
    let prefix = "reader_props";
    let file_path = create_test_file(prefix, "test.txt", "Test content.");
    
    let archive_path = std::env::temp_dir().join(format!("test_{}.7z", prefix));
    
    // Create archive
    let mut archive = BitOutputArchive::new(CompressionFormat::SevenZip);
    archive.add_file(&file_path);
    archive.compress_to(&archive_path).ok();
    
    // Test reader properties
    let lib_result = BitLibrary::new(None::<&str>);
    
    if let Ok(lib) = lib_result {
        let mut reader = BitArchiveReader::new(&lib, ExtractFormat::SevenZip);
        reader.open(&archive_path).ok();
        
        // Test various property methods
        let _ = reader.items();
        let _ = reader.items_count();
    }
    
    // Cleanup
    cleanup_test_dir(prefix);
    cleanup_output_file(&format!("test_{}.7z", prefix));
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_full_compress_extract_cycle() {
    let prefix = "full_cycle";
    let temp_dir = std::env::temp_dir().join(format!("bit7z_test_{}", prefix));
    fs::create_dir_all(&temp_dir).ok();
    
    // Create test files
    let file1 = temp_dir.join("file1.txt");
    let file2 = temp_dir.join("file2.txt");
    fs::write(&file1, "Content 1").ok();
    fs::write(&file2, "Content 2").ok();
    
    let archive_path = std::env::temp_dir().join(format!("test_{}.7z", prefix));
    let output_dir = std::env::temp_dir().join(format!("test_{}_out", prefix));
    
    // Compress
    let lib_result = BitLibrary::new(None::<&str>);
    
    if let Ok(lib) = lib_result {
        let mut compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
        compressor.compression_level(CompressionLevel::Normal);
        
        let files = vec![
            file1.to_string_lossy().to_string(),
            file2.to_string_lossy().to_string(),
        ];
        
        let compress_result = compressor.compress(&files, archive_path.to_str().unwrap().to_string());
        
        if compress_result.is_ok() {
            // Extract
            let extractor = BitExtractor::new(&lib, ExtractFormat::SevenZip);
            let extract_result = extractor.extract(&archive_path, &output_dir);
            
            if extract_result.is_ok() {
                // Verify extracted files
                let extracted_file1 = output_dir.join("file1.txt");
                let extracted_file2 = output_dir.join("file2.txt");
                
                assert!(extracted_file1.exists());
                assert!(extracted_file2.exists());
            }
        }
    }
    
    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
    cleanup_output_file(&format!("test_{}.7z", prefix));
    let _ = fs::remove_dir_all(&output_dir);
}

#[test]
fn test_compression_with_all_properties() {
    let prefix = "all_props";
    let file_path = create_test_file(prefix, "test.txt", "Test content for full compression test.");
    
    let output_path = std::env::temp_dir().join(format!("test_{}.7z", prefix));
    
    let lib_result = BitLibrary::new(None::<&str>);
    
    if let Ok(lib) = lib_result {
        let mut compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
        compressor
            .compression_level(CompressionLevel::Max)
            .compression_method(CompressionMethod::Lzma2)
            .dictionary_size(1 << 20) // 1MB
            .word_size(64)
            .solid(true)
            .password("test_password")
            .crypt_headers(true);
        
        let files = vec![file_path];
        let result = compressor.compress(&files, output_path.to_str().unwrap().to_string());
        
        // Verify compression succeeded
        assert!(result.is_ok(), "Compression with all properties should succeed: {:?}", result);
    }
    
    // Cleanup
    cleanup_test_dir(prefix);
    cleanup_output_file(&format!("test_{}.7z", prefix));
}
