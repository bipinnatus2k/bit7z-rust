//! Example: Using BitArchiveEditor to edit archives
//!
//! This example demonstrates how to:
//! - Rename items in an archive
//! - Update item contents
//! - Delete items from an archive

use bit7z_rust::{BitLibrary, BitArchiveEditor, DeletePolicy, CompressionFormat};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load 7-Zip library
    let lib = BitLibrary::new(None)?;
    
    // Create a test archive first
    let test_archive = "test_edit.7z";
    
    // Clean up if exists
    if Path::new(test_archive).exists() {
        std::fs::remove_file(test_archive)?;
    }
    
    // Create test archive with some files
    {
        use bit7z_rust::BitCompressor;
        let compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
        
        // Create some test files
        std::fs::write("test_file1.txt", "Hello World 1")?;
        std::fs::write("test_file2.txt", "Hello World 2")?;
        std::fs::write("test_file3.txt", "Hello World 3")?;
        
        compressor.compress(&["test_file1.txt", "test_file2.txt", "test_file3.txt"], test_archive)?;
        println!("测试档案已创建：{}", test_archive);
    }
    
    // Now edit the archive
    {
        let mut editor = BitArchiveEditor::new(&lib, test_archive, CompressionFormat::SevenZip, None)?;
        
        // Read original items
        use bit7z_rust::BitArchiveReader;
        let reader = BitArchiveReader::new(&lib, CompressionFormat::SevenZip.into());
        let mut reader_owned = reader;
        reader_owned.open(test_archive)?;
        
        println!("\n原始档案内容:");
        for item in reader_owned.items()? {
            println!("  [{}] {}", item.index, item.path());
        }
        
        // Rename item 0
        editor.rename_item(0, "renamed_file.txt".to_string())?;
        println!("\n重命名项目 0 -> renamed_file.txt");
        
        // Update item 1 content
        std::fs::write("test_file2_updated.txt", "Updated Content")?;
        editor.update_item(1, "test_file2_updated.txt")?;
        println!("更新项目 1 内容");
        
        // Delete item 2
        editor.delete_item(2, DeletePolicy::ItemOnly)?;
        println!("删除项目 2");
        
        // Apply changes
        editor.apply_changes()?;
        println!("\n更改已应用！");
        
        // Read edited archive
        let reader2 = BitArchiveReader::new(&lib, CompressionFormat::SevenZip.into());
        let mut reader2_owned = reader2;
        reader2_owned.open(test_archive)?;
        
        println!("\n编辑后的档案内容:");
        for item in reader2_owned.items()? {
            println!("  [{}] {}", item.index, item.path());
        }
    }
    
    // Clean up test files
    let _ = std::fs::remove_file(test_archive);
    let _ = std::fs::remove_file("test_file1.txt");
    let _ = std::fs::remove_file("test_file2.txt");
    let _ = std::fs::remove_file("test_file3.txt");
    let _ = std::fs::remove_file("test_file2_updated.txt");
    
    println!("\n示例完成！");
    
    Ok(())
}
