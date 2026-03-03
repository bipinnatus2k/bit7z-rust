//! Test BitOutputArchive functionality
//!
//! This test verifies the BitOutputArchive implementation.

use bit7z_rust::{BitOutputArchive, CompressionFormat, CompressionLevel, OverwriteMode};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Create test files for compression
fn create_test_files(case_name: &str) -> Result<(PathBuf, Vec<String>), Box<dyn std::error::Error>> {
    let temp_dir = unique_temp_dir(case_name);
    fs::create_dir_all(&temp_dir)?;

    let mut files = Vec::new();

    // Create test files
    let test_file1 = temp_dir.join("test1.txt");
    fs::write(&test_file1, "Hello, World! This is test file 1.")?;
    files.push(test_file1.to_string_lossy().to_string());

    let test_file2 = temp_dir.join("test2.txt");
    fs::write(&test_file2, "Hello, World! This is test file 2.")?;
    files.push(test_file2.to_string_lossy().to_string());

    Ok((temp_dir, files))
}

/// Clean up test files
fn cleanup_test_files(temp_dir: &PathBuf) {
    let _ = fs::remove_dir_all(temp_dir);
}

fn unique_temp_dir(case_name: &str) -> PathBuf {
    let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "bit7z_test_output_{}_{}_{}",
        case_name,
        std::process::id(),
        id
    ))
}

#[test]
fn test_output_archive_creation() {
    let (temp_dir, files) = match create_test_files("creation") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create test files: {}", e);
            return;
        }
    };

    let output_path = temp_dir.join("test_output.7z");

    let mut archive = BitOutputArchive::new(CompressionFormat::SevenZip);
    archive.compression_level(CompressionLevel::Normal);

    for file in &files {
        archive.add_file(file);
    }

    let result = archive.compress_to(&output_path);

    assert!(result.is_ok(), "Archive creation should succeed: {:?}", result);
    assert!(output_path.exists(), "Output file should exist");

    // Clean up
    let _ = fs::remove_file(&output_path);
    cleanup_test_files(&temp_dir);
}

#[test]
fn test_output_archive_buffer() {
    let (temp_dir, files) = match create_test_files("buffer") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create test files: {}", e);
            return;
        }
    };

    let mut archive = BitOutputArchive::new(CompressionFormat::Zip);
    archive.compression_level(CompressionLevel::Fast);

    for file in &files {
        archive.add_file(file);
    }

    let result = archive.compress_to_buffer();

    // Clean up
    cleanup_test_files(&temp_dir);

    assert!(result.is_ok(), "Buffer compression should succeed: {:?}", result);
    let buffer = result.unwrap();
    assert!(!buffer.is_empty(), "Buffer should not be empty");
}

#[test]
fn test_output_archive_stream() {
    let (temp_dir, files) = match create_test_files("stream") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create test files: {}", e);
            return;
        }
    };

    let mut archive = BitOutputArchive::new(CompressionFormat::SevenZip);

    for file in &files {
        archive.add_file(file);
    }

    let mut buffer = Vec::new();
    let result = archive.compress_to_stream(&mut buffer);

    // Clean up
    cleanup_test_files(&temp_dir);

    assert!(result.is_ok(), "Stream compression should succeed: {:?}", result);
    assert!(!buffer.is_empty(), "Buffer should not be empty");
}

#[test]
fn test_output_archive_with_custom_names() {
    let (temp_dir, files) = match create_test_files("custom_names") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create test files: {}", e);
            return;
        }
    };

    let output_path = temp_dir.join("test_custom_names.7z");

    let mut archive = BitOutputArchive::new(CompressionFormat::SevenZip);

    // Add files with custom names
    archive.add_file_with_name(&files[0], "custom_name1.txt".to_string());
    archive.add_file_with_name(&files[1], "custom_name2.txt".to_string());

    let result = archive.compress_to(&output_path);

    // Clean up
    let _ = fs::remove_file(&output_path);
    cleanup_test_files(&temp_dir);

    assert!(result.is_ok(), "Archive creation with custom names should succeed: {:?}", result);
}

#[test]
fn test_output_archive_overwrite_mode() {
    let (temp_dir, files) = match create_test_files("overwrite_mode") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create test files: {}", e);
            return;
        }
    };

    let output_path = temp_dir.join("test_overwrite.7z");

    // First compression
    let mut archive = BitOutputArchive::new(CompressionFormat::SevenZip);
    archive.add_file(&files[0]);
    let result1 = archive.compress_to(&output_path);
    assert!(result1.is_ok(), "First compression should succeed");

    // Second compression with OverwriteMode::None should fail
    let mut archive2 = BitOutputArchive::new(CompressionFormat::SevenZip);
    archive2.add_file(&files[1]);
    let result2 = archive2.compress_to(&output_path);
    assert!(result2.is_err(), "Second compression with None mode should fail");

    // Third compression with OverwriteMode::Overwrite should succeed
    let mut archive3 = BitOutputArchive::new(CompressionFormat::SevenZip);
    archive3.overwrite_mode(OverwriteMode::Overwrite);
    archive3.add_file(&files[1]);
    let result3 = archive3.compress_to(&output_path);
    assert!(result3.is_ok(), "Third compression with Overwrite mode should succeed");

    // Clean up
    let _ = fs::remove_file(&output_path);
    cleanup_test_files(&temp_dir);
}

fn create_directory_fixture(case_name: &str) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let temp_dir = unique_temp_dir(case_name);
    fs::create_dir_all(&temp_dir)?;

    fs::write(temp_dir.join("file1.txt"), "Content 1")?;
    fs::write(temp_dir.join("file2.txt"), "Content 2")?;

    let sub_dir = temp_dir.join("subdir");
    fs::create_dir_all(&sub_dir)?;
    fs::write(sub_dir.join("file3.txt"), "Content 3")?;

    let output_path = temp_dir.join("test_directory.7z");
    Ok((temp_dir, output_path))
}

#[test]
fn test_output_archive_directory() {
    let (temp_dir, output_path) = match create_directory_fixture("directory") {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to create directory fixture: {}", e);
            return;
        }
    };

    let mut archive = BitOutputArchive::new(CompressionFormat::SevenZip);
    let result = archive.add_directory(&temp_dir);

    assert!(result.is_ok(), "add_directory should succeed: {:?}", result);

    let compress_result = archive.compress_to(&output_path);
    assert!(compress_result.is_ok(), "Directory compression should succeed: {:?}", compress_result);
    assert!(output_path.exists(), "Directory archive should exist");

    // Clean up
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_dir_all(&temp_dir);
}
