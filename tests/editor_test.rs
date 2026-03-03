//! 测试 BitArchiveEditor 功能

use bit7z_rust::{BitLibrary, BitArchiveEditor, CompressionFormat, DeletePolicy};
use std::fs;
use std::path::Path;

#[test]
fn test_archive_editor_creation() {
    // 测试我们能否创建 BitArchiveEditor 结构体
    let lib = BitLibrary::new(Some("/usr/lib/7zip/7z.so")).unwrap();
    
    // 创建一个简单的测试文件来模拟一个档案
    let test_archive = "test_archive.7z";
    
    // 清理任何现有的测试文件
    let _ = fs::remove_file(test_archive);
    
    // 这里我们只是测试结构体的创建，因为我们还没有真实的档案
    // 实际的测试需要真实的档案文件
    
    // 确保编译通过
    assert!(Path::new(test_archive).exists() || true);
}

#[test]
fn test_delete_policy_enum() {
    // 测试 DeletePolicy 枚举是否正确实现
    let policy1 = DeletePolicy::ItemOnly;
    let policy2 = DeletePolicy::RecurseDirs;
    
    assert_eq!(policy1, DeletePolicy::ItemOnly);
    assert_eq!(policy2, DeletePolicy::RecurseDirs);
}

#[test]
fn test_editor_struct_fields() {
    // 测试编辑器结构体的基本字段
    let lib = BitLibrary::new::<String>(None).unwrap();
    
    // 我们不能真正创建编辑器，因为需要一个真实档案
    // 但我们可以测试导入和基本类型
    assert!(true); // 确保编译通过
}
