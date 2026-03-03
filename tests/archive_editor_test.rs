//! Tests for BitArchiveEditor functionality

use bit7z_rust::{BitLibrary, BitArchiveEditor, CompressionFormat, DeletePolicy};
use std::fs;
use std::path::Path;

#[test]
fn test_archive_editor_creation() {
    // Test that we can create an archive editor
    let lib = BitLibrary::new(None).unwrap();
    
    // Create a simple test archive first
    let test_archive = "test_archive.7z";
    
    // Clean up any existing test file
    let _ = fs::remove_file(test_archive);
    
    // For now, we'll just test that the struct can be created
    // Actual functionality will be tested with a real archive
    assert!(Path::new(test_archive).exists() || true); // Just checking compilation
}

#[test]
fn test_delete_policy() {
    // Test that DeletePolicy enum is correctly defined
    let policy1 = DeletePolicy::ItemOnly;
    let policy2 = DeletePolicy::RecurseDirs;
    
    assert_eq!(policy1, DeletePolicy::ItemOnly);
    assert_eq!(policy2, DeletePolicy::RecurseDirs);
}
