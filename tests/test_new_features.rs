//! Integration tests for new bit7z-rust features
//! 
//! Tests for:
//! - BitArchiveEditor
//! - Callbacks
//! - Memory/Stream operations
//! - Selective extraction
//! - Archive testing
//! - Archive properties
//! - Static methods

use bit7z_rust::{
    BitLibrary, BitCompressor, BitExtractor, BitArchiveReader, 
    BitArchiveEditor, DeletePolicy,
    CompressionFormat, ExtractFormat,
};
use std::path::Path;
use std::fs;
use std::sync::{Arc, Mutex};

/// Helper function to get library instance
fn get_library() -> Option<BitLibrary> {
    BitLibrary::new(None::<&str>).ok()
}

/// Helper function to create test files
fn create_test_files(dir: &Path, files: &[(&str, &str)]) {
    for (filename, content) in files {
        let file_path = dir.join(filename);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(&file_path, content).expect("Failed to write test file");
    }
}

/// Helper function to cleanup test directory
fn cleanup_test_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

// ========== BitArchiveEditor Tests ==========

#[test]
fn test_archive_editor_rename() {
    let lib = match get_library() {
        Some(l) => l,
        None => {
            eprintln!("Skipping test: 7-Zip library not available");
            return;
        }
    };
    let test_dir = std::env::temp_dir().join("bit7z_test_editor_rename");
    let archive_path = test_dir.join("test.7z");
    
    // Cleanup if exists
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    // Create test archive
    {
        create_test_files(&test_dir, &[("file1.txt", "content1"), ("file2.txt", "content2")]);
        
        let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
        let files = vec![test_dir.join("file1.txt"), test_dir.join("file2.txt")];
        compressor.compress(&files, &archive_path).unwrap();
    }
    
    // Test rename
    {
        let mut editor = BitArchiveEditor::new(&lib, &archive_path, CompressionFormat::SevenZip, None)
            .expect("Failed to create editor");
        
        editor.rename_item(0, "renamed_file1.txt".to_string()).unwrap();
        editor.apply_changes().unwrap();
        
        // Verify rename
        let mut reader = BitArchiveReader::new(&lib, ExtractFormat::SevenZip);
        reader.open(&archive_path).unwrap();
        
        let items = reader.items().unwrap();
        assert_eq!(items.len(), 2);
        assert!(items[0].path.contains("renamed_file1"));
    }
    
    cleanup_test_dir(&test_dir);
}

#[test]
fn test_archive_editor_update() {
    let lib = get_library();
    let test_dir = std::env::temp_dir().join("bit7z_test_editor_update");
    let archive_path = test_dir.join("test.7z");
    
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    // Create test archive
    {
        fs::write(test_dir.join("file.txt"), "original content").unwrap();
        
        let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
        compressor.compress(&[test_dir.join("file.txt")], &archive_path).unwrap();
    }
    
    // Test update
    {
        let mut editor = BitArchiveEditor::new(&lib, &archive_path, CompressionFormat::SevenZip, None)
            .expect("Failed to create editor");
        
        fs::write(test_dir.join("file_updated.txt"), "updated content").unwrap();
        editor.update_item(0, test_dir.join("file_updated.txt")).unwrap();
        editor.apply_changes().unwrap();
        
        // Extract and verify
        let extractor = BitExtractor::new(&lib, ExtractFormat::SevenZip);
        let output_dir = test_dir.join("output");
        extractor.extract(&archive_path, &output_dir).unwrap();
        
        let content = fs::read_to_string(output_dir.join("file.txt")).unwrap();
        assert_eq!(content, "updated content");
    }
    
    cleanup_test_dir(&test_dir);
}

#[test]
fn test_archive_editor_delete() {
    let lib = get_library();
    let test_dir = std::env::temp_dir().join("bit7z_test_editor_delete");
    let archive_path = test_dir.join("test.7z");
    
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    // Create test archive with 3 files
    {
        create_test_files(&test_dir, &[
            ("file1.txt", "content1"),
            ("file2.txt", "content2"),
            ("file3.txt", "content3"),
        ]);
        
        let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
        compressor.compress(
            &[
                test_dir.join("file1.txt"),
                test_dir.join("file2.txt"),
                test_dir.join("file3.txt"),
            ],
            &archive_path
        ).unwrap();
    }
    
    // Test delete
    {
        let mut editor = BitArchiveEditor::new(&lib, &archive_path, CompressionFormat::SevenZip, None)
            .expect("Failed to create editor");
        
        editor.delete_item(1, DeletePolicy::ItemOnly).unwrap(); // Delete file2.txt
        editor.apply_changes().unwrap();
        
        // Verify deletion
        let mut reader = BitArchiveReader::new(&lib, ExtractFormat::SevenZip);
        reader.open(&archive_path).unwrap();
        
        let items = reader.items().unwrap();
        assert_eq!(items.len(), 2);
    }
    
    cleanup_test_dir(&test_dir);
}

// ========== Callback Tests ==========

#[test]
fn test_compressor_callbacks() {
    let lib = get_library();
    let test_dir = std::env::temp_dir().join("bit7z_test_callbacks");
    let archive_path = test_dir.join("test.7z");
    
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    // Create test files
    create_test_files(&test_dir, &[
        ("file1.txt", "content1"),
        ("file2.txt", "content2"),
    ]);
    
    // Track callback calls
    use std::sync::{Arc, Mutex};
    let file_calls = Arc::new(Mutex::new(Vec::new()));
    let file_calls_clone = file_calls.clone();
    
    let progress_calls = Arc::new(Mutex::new(Vec::new()));
    let progress_calls_clone = progress_calls.clone();
    
    // Create compressor with callbacks
    let mut compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
    
    compressor.set_file_callback(move |path| {
        let mut calls = file_calls_clone.lock().unwrap();
        calls.push(path.to_string());
    });
    
    compressor.set_progress_callback(move |completed, total| {
        let mut calls = progress_calls_clone.lock().unwrap();
        calls.push((completed, total));
        true // Continue
    });
    
    // Compress
    compressor.compress(
        &[test_dir.join("file1.txt"), test_dir.join("file2.txt")],
        &archive_path
    ).unwrap();
    
    // Verify callbacks were called
    let calls = file_calls.lock().unwrap();
    assert!(!calls.is_empty(), "File callback should have been called");
    
    cleanup_test_dir(&test_dir);
}

#[test]
fn test_compressor_password_callback() {
    let lib = get_library();
    let test_dir = std::env::temp_dir().join("bit7z_test_password_cb");
    let archive_path = test_dir.join("test.7z");
    
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    // Create test file
    fs::write(test_dir.join("file.txt"), "secret content").unwrap();
    
    // Create compressor with password callback
    let mut compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
    compressor.password("test123");
    
    let password_called = Arc::new(Mutex::new(false));
    let password_called_clone = password_called.clone();
    
    compressor.set_password_callback(move || {
        let mut called = password_called_clone.lock().unwrap();
        *called = true;
        "test123".to_string()
    });
    
    // Compress with encryption
    compressor.compress(&[test_dir.join("file.txt")], &archive_path).unwrap();
    
    cleanup_test_dir(&test_dir);
}

// ========== Memory/Stream Operations Tests ==========

#[test]
fn test_compress_from_buffer() {
    let lib = get_library();
    let test_dir = std::env::temp_dir().join("bit7z_test_buffer");
    let archive_path = test_dir.join("test.7z");
    
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    let data = b"Hello, World! This is test data.";
    
    let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
    compressor.compress_from_buffer(
        data,
        &archive_path,
        Some("test_file.txt".to_string())
    ).unwrap();
    
    // Verify archive was created
    assert!(archive_path.exists());
    
    // Extract and verify content
    let extractor = BitExtractor::new(&lib, ExtractFormat::SevenZip);
    let buffer = extractor.extract_to_buffer(&archive_path, 0).unwrap();
    
    assert_eq!(&buffer, data);
    
    cleanup_test_dir(&test_dir);
}

#[test]
fn test_compress_from_stream() {
    let lib = get_library();
    let test_dir = std::env::temp_dir().join("bit7z_test_stream");
    let archive_path = test_dir.join("test.7z");
    
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    let data = b"Stream test data";
    let cursor = std::io::Cursor::new(data);
    
    let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
    compressor.compress_from_stream(
        cursor,
        &archive_path,
        "stream_data.txt".to_string()
    ).unwrap();
    
    // Verify archive was created
    assert!(archive_path.exists());
    
    cleanup_test_dir(&test_dir);
}

#[test]
fn test_extract_to_buffer() {
    let lib = get_library();
    let test_dir = std::env::temp_dir().join("bit7z_test_extract_buffer");
    let archive_path = test_dir.join("test.7z");
    
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    // Create archive
    {
        fs::write(test_dir.join("file.txt"), "test content").unwrap();
        
        let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
        compressor.compress(&[test_dir.join("file.txt")], &archive_path).unwrap();
    }
    
    // Extract to buffer
    let extractor = BitExtractor::new(&lib, ExtractFormat::SevenZip);
    let buffer = extractor.extract_to_buffer(&archive_path, 0).unwrap();
    
    assert_eq!(&buffer, b"test content");
    
    cleanup_test_dir(&test_dir);
}

#[test]
fn test_extract_to_stream() {
    let lib = get_library();
    let test_dir = std::env::temp_dir().join("bit7z_test_extract_stream");
    let archive_path = test_dir.join("test.7z");
    
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    // Create archive
    {
        fs::write(test_dir.join("file.txt"), "stream extract test").unwrap();
        
        let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
        compressor.compress(&[test_dir.join("file.txt")], &archive_path).unwrap();
    }
    
    // Extract to stream
    let extractor = BitExtractor::new(&lib, ExtractFormat::SevenZip);
    let mut output = Vec::new();
    extractor.extract_to_stream(&archive_path, &mut output, 0).unwrap();
    
    assert_eq!(&output, b"stream extract test");
    
    cleanup_test_dir(&test_dir);
}

// ========== Selective Extraction Tests ==========

#[test]
fn test_extract_items_by_index() {
    let lib = get_library();
    let test_dir = std::env::temp_dir().join("bit7z_test_extract_items");
    let archive_path = test_dir.join("test.7z");
    let output_dir = test_dir.join("output");
    
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    // Create archive with 3 files
    {
        create_test_files(&test_dir, &[
            ("file1.txt", "content1"),
            ("file2.txt", "content2"),
            ("file3.txt", "content3"),
        ]);
        
        let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
        compressor.compress(
            &[
                test_dir.join("file1.txt"),
                test_dir.join("file2.txt"),
                test_dir.join("file3.txt"),
            ],
            &archive_path
        ).unwrap();
    }
    
    // Extract only items 0 and 2
    let extractor = BitExtractor::new(&lib, ExtractFormat::SevenZip);
    extractor.extract_items(&archive_path, &[0, 2], &output_dir).unwrap();
    
    // Verify only selected files were extracted
    assert!(output_dir.join("file1.txt").exists());
    assert!(!output_dir.join("file2.txt").exists());
    assert!(output_dir.join("file3.txt").exists());
    
    cleanup_test_dir(&test_dir);
}

#[test]
fn test_extract_matching_wildcard() {
    let lib = get_library();
    let test_dir = std::env::temp_dir().join("bit7z_test_wildcard");
    let archive_path = test_dir.join("test.7z");
    let output_dir = test_dir.join("output");
    
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    // Create archive with mixed extensions
    {
        create_test_files(&test_dir, &[
            ("file1.txt", "txt1"),
            ("file2.txt", "txt2"),
            ("file3.log", "log1"),
            ("file4.dat", "dat1"),
        ]);
        
        let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
        compressor.compress(
            &[
                test_dir.join("file1.txt"),
                test_dir.join("file2.txt"),
                test_dir.join("file3.log"),
                test_dir.join("file4.dat"),
            ],
            &archive_path
        ).unwrap();
    }
    
    // Extract only .txt files
    let extractor = BitExtractor::new(&lib, ExtractFormat::SevenZip);
    extractor.extract_matching(&archive_path, "*.txt", &output_dir).unwrap();
    
    // Verify only .txt files were extracted
    assert!(output_dir.join("file1.txt").exists());
    assert!(output_dir.join("file2.txt").exists());
    assert!(!output_dir.join("file3.log").exists());
    assert!(!output_dir.join("file4.dat").exists());
    
    cleanup_test_dir(&test_dir);
}

#[test]
fn test_extract_matching_regex() {
    let lib = get_library();
    let test_dir = std::env::temp_dir().join("bit7z_test_regex");
    let archive_path = test_dir.join("test.7z");
    let output_dir = test_dir.join("output");
    
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    // Create archive with subdirectories
    {
        create_test_files(&test_dir, &[
            ("docs/readme.md", "# README"),
            ("docs/guide.md", "# Guide"),
            ("docs/notes.txt", "Notes"),
            ("src/main.rs", "fn main() {}"),
        ]);
        
        let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
        
        let files = vec![
            test_dir.join("docs/readme.md"),
            test_dir.join("docs/guide.md"),
            test_dir.join("docs/notes.txt"),
            test_dir.join("src/main.rs"),
        ];
        compressor.compress(&files, &archive_path).unwrap();
    }
    
    // Extract only .md files from docs/
    let extractor = BitExtractor::new(&lib, ExtractFormat::SevenZip);
    extractor.extract_matching_regex(&archive_path, r"^docs/.*\.md$", &output_dir).unwrap();
    
    // Verify extraction
    assert!(output_dir.join("docs/readme.md").exists());
    assert!(output_dir.join("docs/guide.md").exists());
    assert!(!output_dir.join("docs/notes.txt").exists());
    assert!(!output_dir.join("src/main.rs").exists());
    
    cleanup_test_dir(&test_dir);
}

// ========== Archive Testing Tests ==========

#[test]
fn test_archive_integrity() {
    let lib = get_library();
    let test_dir = std::env::temp_dir().join("bit7z_test_integrity");
    let archive_path = test_dir.join("test.7z");
    
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    // Create valid archive
    {
        fs::write(test_dir.join("file.txt"), "test content").unwrap();
        
        let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
        compressor.compress(&[test_dir.join("file.txt")], &archive_path).unwrap();
    }
    
    // Test with extractor
    let extractor = BitExtractor::new(&lib, ExtractFormat::SevenZip);
    extractor.test(&archive_path).unwrap(); // Should not fail
    
    // Test with reader
    let mut reader = BitArchiveReader::new(&lib, ExtractFormat::SevenZip);
    reader.open(&archive_path).unwrap();
    reader.test().unwrap(); // Should not fail
    
    cleanup_test_dir(&test_dir);
}

// ========== Archive Properties Tests ==========

#[test]
fn test_archive_properties() {
    let lib = get_library();
    let test_dir = std::env::temp_dir().join("bit7z_test_properties");
    let archive_path = test_dir.join("test.7z");
    
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    // Create archive
    {
        create_test_files(&test_dir, &[
            ("file1.txt", "content1"),
            ("file2.txt", "content2"),
            ("subdir/file3.txt", "content3"),
        ]);
        
        let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
        compressor.compress(
            &[
                test_dir.join("file1.txt"),
                test_dir.join("file2.txt"),
                test_dir.join("subdir/file3.txt"),
            ],
            &archive_path
        ).unwrap();
    }
    
    // Test properties
    let mut reader = BitArchiveReader::new(&lib, ExtractFormat::SevenZip);
    reader.open(&archive_path).unwrap();
    
    let props = reader.archive_properties().unwrap();
    
    assert_eq!(props.items_count, 3);
    assert_eq!(props.files_count, 3);
    assert!(props.size > 0);
    assert!(props.pack_size > 0);
    
    // Test individual property methods
    assert!(!reader.is_solid().unwrap());
    assert!(!reader.is_multi_volume().unwrap());
    assert_eq!(reader.volumes_count().unwrap(), 1);
    assert!(!reader.has_encrypted_items().unwrap());
    assert!(!reader.is_encrypted().unwrap());
    
    cleanup_test_dir(&test_dir);
}

// ========== Static Methods Tests ==========

#[test]
fn test_static_encryption_check() {
    let lib = get_library();
    let test_dir = std::env::temp_dir().join("bit7z_test_static_enc");
    let archive_path = test_dir.join("test.7z");
    
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    // Create unencrypted archive
    {
        fs::write(test_dir.join("file.txt"), "public content").unwrap();
        
        let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
        compressor.compress(&[test_dir.join("file.txt")], &archive_path).unwrap();
    }
    
    // Test static methods
    let is_encrypted = BitArchiveReader::is_encrypted_static(
        &lib,
        &archive_path,
        ExtractFormat::SevenZip
    ).unwrap();
    
    assert!(!is_encrypted);
    
    let is_header_encrypted = BitArchiveReader::is_header_encrypted_static(
        &lib,
        &archive_path,
        ExtractFormat::SevenZip
    ).unwrap();
    
    assert!(!is_header_encrypted);
    
    cleanup_test_dir(&test_dir);
}

// ========== Directory Compression Tests ==========

#[test]
fn test_compress_files_only() {
    let lib = get_library();
    let test_dir = std::env::temp_dir().join("bit7z_test_files_only");
    let archive_path = test_dir.join("test.7z");
    
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    // Create files and directories
    fs::write(test_dir.join("file1.txt"), "content1").unwrap();
    fs::write(test_dir.join("file2.txt"), "content2").unwrap();
    fs::create_dir_all(test_dir.join("empty_dir")).unwrap();
    
    let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
    compressor.compress_files(
        &[
            test_dir.join("file1.txt"),
            test_dir.join("file2.txt"),
            test_dir.join("empty_dir"), // Should be ignored
        ],
        &archive_path
    ).unwrap();
    
    // Verify archive was created
    assert!(archive_path.exists());
    
    // Verify only files are in archive
    let mut reader = BitArchiveReader::new(&lib, ExtractFormat::SevenZip);
    reader.open(&archive_path).unwrap();
    
    let items = reader.items().unwrap();
    assert_eq!(items.len(), 2); // Only 2 files
    
    cleanup_test_dir(&test_dir);
}

#[test]
fn test_compress_directory_with_filter() {
    let lib = get_library();
    let test_dir = std::env::temp_dir().join("bit7z_test_dir_filter");
    let archive_path = test_dir.join("test.7z");
    
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    // Create directory with mixed files
    create_test_files(&test_dir, &[
        ("file1.txt", "txt1"),
        ("file2.txt", "txt2"),
        ("file3.log", "log1"),
        ("file4.txt", "txt3"),
    ]);
    
    let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
    compressor.compress_directory_contents(
        &test_dir,
        &archive_path,
        true, // recursive
        Some("*.txt") // filter
    ).unwrap();
    
    // Verify only .txt files are in archive
    let mut reader = BitArchiveReader::new(&lib, ExtractFormat::SevenZip);
    reader.open(&archive_path).unwrap();
    
    let items = reader.items().unwrap();
    assert_eq!(items.len(), 3); // Only 3 .txt files
    
    cleanup_test_dir(&test_dir);
}

#[test]
fn test_compress_with_aliases() {
    let lib = get_library();
    let test_dir = std::env::temp_dir().join("bit7z_test_aliases");
    let archive_path = test_dir.join("test.7z");
    
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    // Create test files
    fs::write(test_dir.join("data.txt"), "content").unwrap();
    
    let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
    compressor.compress_with_aliases(
        &[(test_dir.join("data.txt"), "backup/renamed.txt".to_string())],
        archive_path.clone()
    ).unwrap();
    
    // Verify archive name
    let mut reader = BitArchiveReader::new(&lib, ExtractFormat::SevenZip);
    reader.open(&archive_path).unwrap();
    
    let items = reader.items().unwrap();
    assert_eq!(items.len(), 1);
    assert!(items[0].path.contains("backup/renamed.txt"));
    
    cleanup_test_dir(&test_dir);
}

// ========== Multi-volume Archive Tests ==========

#[test]
fn test_volume_size_setting() {
    let lib = get_library();
    let test_dir = std::env::temp_dir().join("bit7z_test_volume");
    let archive_path = test_dir.join("test.7z");
    
    cleanup_test_dir(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    // Create test data larger than volume size
    let large_data = vec![b'A'; 2000]; // 2KB
    fs::write(test_dir.join("large_file.bin"), &large_data).unwrap();
    
    let mut compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
    compressor.volume_size(500); // 500 bytes per volume
    
    // Note: Full multi-volume support requires IOutStream implementation
    // This test verifies the API exists and can be called
    compressor.compress(&[test_dir.join("large_file.bin")], &archive_path).unwrap();
    
    // Verify archive was created
    assert!(archive_path.exists());
    
    cleanup_test_dir(&test_dir);
}
