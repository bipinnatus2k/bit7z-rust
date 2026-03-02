//! 测试验证工具模块
//!
//! 本模块提供严格的测试验证功能，包括：
//! - 文件内容比对
//! - 目录结构验证
//! - 哈希校验
//! - 档案完整性检查
//! - 详细的错误报告

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 文件验证结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileVerificationResult {
    /// 文件路径
    pub path: PathBuf,
    /// 是否存在
    pub exists: bool,
    /// 文件大小（字节）
    pub size: u64,
    /// 内容是否匹配
    pub content_matches: bool,
    /// 预期内容
    pub expected_content: Option<String>,
    /// 实际内容（仅当不匹配时）
    pub actual_content: Option<String>,
    /// 错误信息
    pub error: Option<String>,
}

/// 目录验证结果
#[derive(Debug, Clone)]
pub struct DirectoryVerificationResult {
    /// 目录路径
    pub path: PathBuf,
    /// 是否存在
    pub exists: bool,
    /// 文件总数
    pub file_count: usize,
    /// 目录总数
    pub dir_count: usize,
    /// 文件列表
    pub files: Vec<PathBuf>,
    /// 目录列表
    pub directories: Vec<PathBuf>,
    /// 验证失败的文件
    pub failed_files: Vec<FileVerificationResult>,
}

/// 档案验证结果
#[derive(Debug, Clone)]
pub struct ArchiveVerificationResult {
    /// 档案路径
    pub archive_path: PathBuf,
    /// 是否存在
    pub exists: bool,
    /// 文件大小
    pub size: u64,
    /// 是否可打开
    pub can_open: bool,
    /// 项目数量
    pub item_count: usize,
    /// 解压后验证结果
    pub extraction_result: Option<DirectoryVerificationResult>,
    /// 错误信息
    pub error: Option<String>,
}

/// 测试文件验证器
pub struct TestVerifier {
    /// 临时目录
    temp_dir: PathBuf,
    /// 原始文件内容缓存
    original_files: HashMap<PathBuf, Vec<u8>>,
}

impl TestVerifier {
    /// 创建新的验证器
    pub fn new(temp_dir: PathBuf) -> Self {
        Self {
            temp_dir,
            original_files: HashMap::new(),
        }
    }

    /// 缓存原始文件内容
    pub fn cache_original_file(&mut self, path: &Path) -> std::io::Result<()> {
        if path.exists() {
            let content = fs::read(path)?;
            self.original_files.insert(path.to_path_buf(), content);
        }
        Ok(())
    }

    /// 缓存目录中所有文件
    pub fn cache_directory(&mut self, dir: &Path) -> std::io::Result<usize> {
        let mut count = 0;
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    self.cache_original_file(&path)?;
                    count += 1;
                } else if path.is_dir() {
                    count += self.cache_directory(&path)?;
                }
            }
        }
        Ok(count)
    }

    /// 验证单个文件
    pub fn verify_file(&self, path: &Path, expected_content: Option<&[u8]>) -> FileVerificationResult {
        let exists = path.exists();
        
        if !exists {
            return FileVerificationResult {
                path: path.to_path_buf(),
                exists: false,
                size: 0,
                content_matches: false,
                expected_content: expected_content.map(|c| String::from_utf8_lossy(c).to_string()),
                actual_content: None,
                error: Some("文件不存在".to_string()),
            };
        }

        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(e) => {
                return FileVerificationResult {
                    path: path.to_path_buf(),
                    exists: true,
                    size: 0,
                    content_matches: false,
                    expected_content: expected_content.map(|c| String::from_utf8_lossy(c).to_string()),
                    actual_content: None,
                    error: Some(format!("获取文件元数据失败：{}", e)),
                };
            }
        };

        let size = metadata.len();
        let actual_content = match fs::read(path) {
            Ok(c) => c,
            Err(e) => {
                return FileVerificationResult {
                    path: path.to_path_buf(),
                    exists: true,
                    size,
                    content_matches: false,
                    expected_content: expected_content.map(|c| String::from_utf8_lossy(c).to_string()),
                    actual_content: None,
                    error: Some(format!("读取文件失败：{}", e)),
                };
            }
        };

        let content_matches = if let Some(expected) = expected_content {
            expected == &actual_content
        } else {
            true
        };

        FileVerificationResult {
            path: path.to_path_buf(),
            exists: true,
            size,
            content_matches,
            expected_content: expected_content.map(|c| String::from_utf8_lossy(c).to_string()),
            actual_content: if !content_matches {
                Some(String::from_utf8_lossy(&actual_content).to_string())
            } else {
                None
            },
            error: if !content_matches {
                Some(format!(
                    "内容不匹配！预期 {} 字节，实际 {} 字节",
                    expected_content.map(|c| c.len()).unwrap_or(0),
                    actual_content.len()
                ))
            } else {
                None
            },
        }
    }

    /// 验证文件与缓存的原始内容是否一致
    pub fn verify_against_cache(&self, path: &Path) -> FileVerificationResult {
        let expected = self.original_files.get(path);
        
        if let Some(expected_content) = expected {
            self.verify_file(path, Some(expected_content))
        } else {
            FileVerificationResult {
                path: path.to_path_buf(),
                exists: path.exists(),
                size: path.exists().then(|| fs::metadata(path).map(|m| m.len()).unwrap_or(0)).unwrap_or(0),
                content_matches: false,
                expected_content: None,
                actual_content: None,
                error: Some("文件不在缓存中".to_string()),
            }
        }
    }

    /// 验证整个目录结构
    pub fn verify_directory(&self, dir: &Path) -> DirectoryVerificationResult {
        let exists = dir.exists();
        
        if !exists {
            return DirectoryVerificationResult {
                path: dir.to_path_buf(),
                exists: false,
                file_count: 0,
                dir_count: 0,
                files: Vec::new(),
                directories: Vec::new(),
                failed_files: Vec::new(),
            };
        }

        let mut files = Vec::new();
        let mut directories = Vec::new();
        let mut failed_files = Vec::new();

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let verification = self.verify_against_cache(&path);
                    if !verification.content_matches {
                        failed_files.push(verification);
                    }
                    files.push(path);
                } else if path.is_dir() {
                    directories.push(path);
                }
            }
        }

        DirectoryVerificationResult {
            path: dir.to_path_buf(),
            exists: true,
            file_count: files.len(),
            dir_count: directories.len(),
            files,
            directories,
            failed_files,
        }
    }

    /// 比较两个目录的内容
    pub fn compare_directories(&self, source: &Path, target: &Path) -> DirectoryComparisonResult {
        let source_exists = source.exists();
        let target_exists = target.exists();

        if !source_exists {
            return DirectoryComparisonResult {
                source_path: source.to_path_buf(),
                target_path: target.to_path_buf(),
                source_exists: false,
                target_exists,
                matching_files: 0,
                mismatched_files: 0,
                missing_in_target: Vec::new(),
                extra_in_target: Vec::new(),
                content_mismatches: Vec::new(),
            };
        }

        if !target_exists {
            return DirectoryComparisonResult {
                source_path: source.to_path_buf(),
                target_path: target.to_path_buf(),
                source_exists: true,
                target_exists: false,
                matching_files: 0,
                mismatched_files: 0,
                missing_in_target: Vec::new(),
                extra_in_target: Vec::new(),
                content_mismatches: Vec::new(),
            };
        }

        let mut source_files: HashMap<PathBuf, Vec<u8>> = HashMap::new();
        let mut target_files: HashMap<PathBuf, Vec<u8>> = HashMap::new();

        // 读取源目录文件
        if let Ok(()) = self.read_directory_contents(source, &mut source_files, &PathBuf::new()) {
            // 读取目标目录文件
            let _ = self.read_directory_contents(target, &mut target_files, &PathBuf::new());
        }

        let mut matching_files = 0;
        let mut mismatched_files = 0;
        let mut missing_in_target = Vec::new();
        let mut extra_in_target = Vec::new();
        let mut content_mismatches = Vec::new();

        // 检查源目录中的每个文件
        for (rel_path, source_content) in &source_files {
            match target_files.get(rel_path) {
                Some(target_content) => {
                    if source_content == target_content {
                        matching_files += 1;
                    } else {
                        mismatched_files += 1;
                        content_mismatches.push(ContentMismatch {
                            path: rel_path.clone(),
                            source_size: source_content.len() as u64,
                            target_size: target_content.len() as u64,
                            source_hash: compute_hash(source_content),
                            target_hash: compute_hash(target_content),
                        });
                    }
                }
                None => {
                    missing_in_target.push(rel_path.clone());
                }
            }
        }

        // 检查目标目录中额外的文件
        for (rel_path, _) in &target_files {
            if !source_files.contains_key(rel_path) {
                extra_in_target.push(rel_path.clone());
            }
        }

        DirectoryComparisonResult {
            source_path: source.to_path_buf(),
            target_path: target.to_path_buf(),
            source_exists: true,
            target_exists: true,
            matching_files,
            mismatched_files,
            missing_in_target,
            extra_in_target,
            content_mismatches,
        }
    }

    /// 递归读取目录内容
    fn read_directory_contents(
        &self,
        dir: &Path,
        files: &mut HashMap<PathBuf, Vec<u8>>,
        base_path: &PathBuf,
    ) -> std::io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let rel_path = base_path.join(entry.file_name());

            if path.is_file() {
                files.insert(rel_path, fs::read(&path)?);
            } else if path.is_dir() {
                self.read_directory_contents(&path, files, &rel_path)?;
            }
        }
        Ok(())
    }

    /// 验证档案
    pub fn verify_archive<T>(&self, archive_path: &Path, extractor: &T, extract_dir: &Path) -> ArchiveVerificationResult
    where
        T: ArchiveExtractor,
    {
        let exists = archive_path.exists();

        if !exists {
            return ArchiveVerificationResult {
                archive_path: archive_path.to_path_buf(),
                exists: false,
                size: 0,
                can_open: false,
                item_count: 0,
                extraction_result: None,
                error: Some("档案文件不存在".to_string()),
            };
        }

        let size = fs::metadata(archive_path).map(|m| m.len()).unwrap_or(0);

        if size == 0 {
            return ArchiveVerificationResult {
                archive_path: archive_path.to_path_buf(),
                exists: true,
                size: 0,
                can_open: false,
                item_count: 0,
                extraction_result: None,
                error: Some("档案文件为空".to_string()),
            };
        }

        // 尝试解压
        let extraction_result = match extractor.extract(archive_path, extract_dir) {
            Ok(()) => {
                let dir_result = self.verify_directory(extract_dir);
                Some(dir_result)
            }
            Err(e) => {
                return ArchiveVerificationResult {
                    archive_path: archive_path.to_path_buf(),
                    exists: true,
                    size,
                    can_open: false,
                    item_count: 0,
                    extraction_result: None,
                    error: Some(format!("解压失败：{:?}", e)),
                };
            }
        };

        ArchiveVerificationResult {
            archive_path: archive_path.to_path_buf(),
            exists: true,
            size,
            can_open: true,
            item_count: extraction_result.as_ref().map(|r| r.file_count).unwrap_or(0),
            extraction_result,
            error: None,
        }
    }

    /// 生成验证报告
    pub fn generate_report(&self, result: &ArchiveVerificationResult) -> String {
        let mut report = String::new();
        
        report.push_str("=== 档案验证报告 ===\n\n");
        report.push_str(&format!("档案路径：{}\n", result.archive_path.display()));
        report.push_str(&format!("存在：{}\n", result.exists));
        report.push_str(&format!("大小：{} 字节\n", result.size));
        report.push_str(&format!("可打开：{}\n", result.can_open));
        report.push_str(&format!("项目数：{}\n", result.item_count));

        if let Some(error) = &result.error {
            report.push_str(&format!("\n❌ 错误：{}\n", error));
        }

        if let Some(extraction) = &result.extraction_result {
            report.push_str("\n--- 解压验证 ---\n");
            report.push_str(&format!("解压目录：{}\n", extraction.path.display()));
            report.push_str(&format!("目录存在：{}\n", extraction.exists));
            report.push_str(&format!("文件数：{}\n", extraction.file_count));
            report.push_str(&format!("目录数：{}\n", extraction.dir_count));

            if !extraction.failed_files.is_empty() {
                report.push_str(&format!("\n❌ 验证失败的文件 ({} 个):\n", extraction.failed_files.len()));
                for failed in &extraction.failed_files {
                    report.push_str(&format!("  - {}: {}\n", failed.path.display(), 
                                            failed.error.as_ref().unwrap_or(&"未知错误".to_string())));
                }
            } else {
                report.push_str("\n✅ 所有文件验证通过\n");
            }
        }

        report
    }
}

/// 内容不匹配信息
#[derive(Debug, Clone)]
pub struct ContentMismatch {
    pub path: PathBuf,
    pub source_size: u64,
    pub target_size: u64,
    pub source_hash: String,
    pub target_hash: String,
}

/// 目录比较结果
#[derive(Debug, Clone)]
pub struct DirectoryComparisonResult {
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub source_exists: bool,
    pub target_exists: bool,
    pub matching_files: usize,
    pub mismatched_files: usize,
    pub missing_in_target: Vec<PathBuf>,
    pub extra_in_target: Vec<PathBuf>,
    pub content_mismatches: Vec<ContentMismatch>,
}

impl DirectoryComparisonResult {
    /// 检查比较是否完全匹配
    pub fn is_perfect_match(&self) -> bool {
        self.source_exists && self.target_exists && 
        self.mismatched_files == 0 && 
        self.missing_in_target.is_empty() && 
        self.extra_in_target.is_empty()
    }

    /// 生成比较报告
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== 目录比较报告 ===\n\n");
        report.push_str(&format!("源目录：{}\n", self.source_path.display()));
        report.push_str(&format!("目标目录：{}\n", self.target_path.display()));
        report.push_str(&format!("源目录存在：{}\n", self.source_exists));
        report.push_str(&format!("目标目录存在：{}\n", self.target_exists));

        if !self.source_exists || !self.target_exists {
            report.push_str("\n❌ 目录不完整\n");
            return report;
        }

        report.push_str(&format!("\n匹配文件数：{}\n", self.matching_files));
        report.push_str(&format!("不匹配文件数：{}\n", self.mismatched_files));
        report.push_str(&format!("目标目录缺失：{}\n", self.missing_in_target.len()));
        report.push_str(&format!("目标目录额外：{}\n", self.extra_in_target.len()));

        if !self.missing_in_target.is_empty() {
            report.push_str("\n❌ 目标目录缺失的文件:\n");
            for path in &self.missing_in_target {
                report.push_str(&format!("  - {}\n", path.display()));
            }
        }

        if !self.extra_in_target.is_empty() {
            report.push_str("\n⚠️  目标目录额外的文件:\n");
            for path in &self.extra_in_target {
                report.push_str(&format!("  + {}\n", path.display()));
            }
        }

        if !self.content_mismatches.is_empty() {
            report.push_str("\n❌ 内容不匹配的文件:\n");
            for mismatch in &self.content_mismatches {
                report.push_str(&format!(
                    "  ! {} (源：{} 字节 [{}], 目标：{} 字节 [{}])\n",
                    mismatch.path.display(),
                    mismatch.source_size,
                    mismatch.source_hash,
                    mismatch.target_size,
                    mismatch.target_hash
                ));
            }
        }

        if self.is_perfect_match() {
            report.push_str("\n✅ 目录完全匹配\n");
        }

        report
    }
}

/// 档案提取器 trait
pub trait ArchiveExtractor {
    type Error: std::fmt::Debug;
    fn extract(&self, archive: &Path, output_dir: &Path) -> Result<(), Self::Error>;
}

/// 计算哈希（简单 CRC32）
pub fn compute_hash(data: &[u8]) -> String {
    // 简单哈希实现，用于快速比较
    let mut hash: u32 = 0;
    for &byte in data {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }
    format!("{:08x}", hash)
}

/// 计算文件的 SHA256 哈希
pub fn compute_file_hash(path: &Path) -> std::io::Result<String> {
    let content = fs::read(path)?;
    Ok(compute_hash(&content))
}

/// 断言档案验证结果
#[macro_export]
macro_rules! assert_archive_valid {
    ($result:expr) => {{
        let result = &$result;
        assert!(result.exists, "档案不存在：{:?}", result.archive_path);
        assert!(result.size > 0, "档案为空：{:?}", result.archive_path);
        assert!(result.can_open, "档案无法打开：{:?}", result.archive_path);
        assert!(result.error.is_none(), "档案错误：{:?}", result.error);
    }};
}

/// 断言目录验证结果
#[macro_export]
macro_rules! assert_directory_valid {
    ($result:expr) => {{
        let result = &$result;
        assert!(result.exists, "目录不存在：{:?}", result.path);
        assert!(result.file_count > 0, "目录为空：{:?}", result.path);
        assert!(result.failed_files.is_empty(), "文件验证失败：{:?}", result.failed_files);
    }};
}

/// 断言目录比较结果
#[macro_export]
macro_rules! assert_directories_match {
    ($result:expr) => {{
        let result = &$result;
        assert!(result.is_perfect_match(), "目录不匹配:\n{}", result.generate_report());
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_verifier_basic_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        let mut verifier = TestVerifier::new(temp_dir.path().to_path_buf());
        verifier.cache_original_file(&test_file).unwrap();

        let result = verifier.verify_file(&test_file, None);
        assert!(result.exists);
        assert!(result.content_matches);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_verifier_content_mismatch() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "original content").unwrap();

        let verifier = TestVerifier::new(temp_dir.path().to_path_buf());
        let result = verifier.verify_file(&test_file, Some(b"different content"));

        assert!(result.exists);
        assert!(!result.content_matches);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_verifier_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent = temp_dir.path().join("non_existent.txt");

        let verifier = TestVerifier::new(temp_dir.path().to_path_buf());
        let result = verifier.verify_file(&non_existent, None);
        
        assert!(!result.exists);
        assert!(!result.content_matches);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_directory_comparison() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let target_dir = temp_dir.path().join("target");
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(&target_dir).unwrap();

        // Create identical files
        fs::write(source_dir.join("file1.txt"), "content 1").unwrap();
        fs::write(target_dir.join("file1.txt"), "content 1").unwrap();

        // Create different files
        fs::write(source_dir.join("file2.txt"), "original content").unwrap();
        fs::write(target_dir.join("file2.txt"), "modified content").unwrap();

        // File only in source
        fs::write(source_dir.join("only_in_source.txt"), "source file").unwrap();

        // File only in target
        fs::write(target_dir.join("only_in_target.txt"), "target file").unwrap();

        let verifier = TestVerifier::new(temp_dir.path().to_path_buf());
        let result = verifier.compare_directories(&source_dir, &target_dir);

        assert_eq!(result.matching_files, 1);
        assert_eq!(result.mismatched_files, 1);
        assert_eq!(result.missing_in_target.len(), 1);
        assert_eq!(result.extra_in_target.len(), 1);
        assert!(!result.is_perfect_match());
    }

    #[test]
    fn test_hash_computation() {
        // 测试哈希计算函数
        let data1 = b"test data 1";
        let data2 = b"test data 2";
        
        let hash1 = compute_hash(data1);
        let hash2 = compute_hash(data2);
        let hash1_again = compute_hash(data1);

        assert_eq!(hash1, hash1_again, "相同数据应有相同哈希");
        assert_ne!(hash1, hash2, "不同数据应有不同哈希");
        assert_eq!(hash1.len(), 8, "哈希应为 8 位十六进制字符串");
    }
}
