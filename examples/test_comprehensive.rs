//! 全面格式测试工具
//! 
//! 测试 bit7z-rust 支持的所有格式

use bit7z_rust::{BitLibrary, BitExtractor, ExtractFormat};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 测试结果
#[derive(Debug, Clone)]
struct TestResult {
    format_name: String,
    extension: String,
    file_exists: bool,
    extract_success: bool,
    error_message: Option<String>,
    extracted_files: Vec<String>,
}

/// 格式信息
struct FormatInfo {
    name: &'static str,
    format: ExtractFormat,
    extension: &'static str,
    category: &'static str,
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║          bit7z-rust 全面格式测试工具                     ║");
    println!("║              Comprehensive Format Test Suite             ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // 加载 7-Zip 库
    let lib = match BitLibrary::new(Some("/usr/lib/7zip/7z.so")) {
        Ok(l) => {
            println!("✓ 7-Zip 库加载成功 (/usr/lib/7zip/7z.so)\n");
            l
        }
        Err(e) => {
            eprintln!("✗ 库加载失败：{}", e);
            eprintln!("提示：请确保已安装 7-Zip 并找到正确的库文件路径");
            return;
        }
    };

    // 定义所有要测试的格式
    let formats = vec![
        // 压缩/解压格式
        FormatInfo { name: "7z", format: ExtractFormat::SevenZip, extension: "7z", category: "压缩格式" },
        FormatInfo { name: "ZIP", format: ExtractFormat::Zip, extension: "zip", category: "压缩格式" },
        FormatInfo { name: "GZIP", format: ExtractFormat::GZip, extension: "gz", category: "压缩格式" },
        FormatInfo { name: "BZIP2", format: ExtractFormat::BZip2, extension: "bz2", category: "压缩格式" },
        FormatInfo { name: "TAR", format: ExtractFormat::Tar, extension: "tar", category: "压缩格式" },
        FormatInfo { name: "XZ", format: ExtractFormat::Xz, extension: "xz", category: "压缩格式" },
        FormatInfo { name: "WIM", format: ExtractFormat::Wim, extension: "wim", category: "压缩格式" },
        FormatInfo { name: "LZMA", format: ExtractFormat::Lzma, extension: "lzma", category: "压缩格式" },
        
        // 只读压缩格式
        FormatInfo { name: "RAR", format: ExtractFormat::Rar, extension: "rar", category: "只读格式" },
        FormatInfo { name: "RAR5", format: ExtractFormat::Rar5, extension: "rar", category: "只读格式" },
        FormatInfo { name: "ARJ", format: ExtractFormat::Arj, extension: "arj", category: "只读格式" },
        FormatInfo { name: "LZH", format: ExtractFormat::Lzh, extension: "lzh", category: "只读格式" },
        FormatInfo { name: "CAB", format: ExtractFormat::Cab, extension: "cab", category: "只读格式" },
        FormatInfo { name: "NSIS", format: ExtractFormat::Nsis, extension: "nsis", category: "只读格式" },
        FormatInfo { name: "CPIO", format: ExtractFormat::Cpio, extension: "cpio", category: "只读格式" },
        FormatInfo { name: "RPM", format: ExtractFormat::Rpm, extension: "rpm", category: "只读格式" },
        FormatInfo { name: "DEB", format: ExtractFormat::Deb, extension: "deb", category: "只读格式" },
        FormatInfo { name: "Z", format: ExtractFormat::Z, extension: "z", category: "只读格式" },
        FormatInfo { name: "SPLIT", format: ExtractFormat::Split, extension: "001", category: "只读格式" },
        
        // 磁盘镜像格式
        FormatInfo { name: "ISO", format: ExtractFormat::Iso, extension: "iso", category: "磁盘镜像" },
        FormatInfo { name: "UDF", format: ExtractFormat::Udf, extension: "udf", category: "磁盘镜像" },
        FormatInfo { name: "DMG", format: ExtractFormat::Dmg, extension: "dmg", category: "磁盘镜像" },
        FormatInfo { name: "FAT", format: ExtractFormat::Fat, extension: "fat", category: "磁盘镜像" },
        FormatInfo { name: "NTFS", format: ExtractFormat::Ntfs, extension: "ntfs", category: "磁盘镜像" },
        FormatInfo { name: "HFS", format: ExtractFormat::Hfs, extension: "hfs", category: "磁盘镜像" },
        FormatInfo { name: "Ext", format: ExtractFormat::Ext, extension: "ext", category: "磁盘镜像" },
        FormatInfo { name: "APFS", format: ExtractFormat::Apfs, extension: "apfs", category: "磁盘镜像" },
        FormatInfo { name: "QCOW", format: ExtractFormat::Qcow, extension: "qcow2", category: "磁盘镜像" },
        FormatInfo { name: "VDI", format: ExtractFormat::Vdi, extension: "vdi", category: "磁盘镜像" },
        FormatInfo { name: "VHD", format: ExtractFormat::Vhd, extension: "vhd", category: "磁盘镜像" },
        FormatInfo { name: "VHDX", format: ExtractFormat::Vhdx, extension: "vhdx", category: "磁盘镜像" },
        FormatInfo { name: "VMDK", format: ExtractFormat::Vmdk, extension: "vmdk", category: "磁盘镜像" },
        
        // 文件系统格式
        FormatInfo { name: "CramFS", format: ExtractFormat::Cramfs, extension: "cramfs", category: "文件系统" },
        FormatInfo { name: "SquashFS", format: ExtractFormat::Squashfs, extension: "squashfs", category: "文件系统" },
        
        // 可执行文件格式
        FormatInfo { name: "ELF", format: ExtractFormat::Elf, extension: "elf", category: "可执行文件" },
        FormatInfo { name: "Mach-O", format: ExtractFormat::Macho, extension: "macho", category: "可执行文件" },
        FormatInfo { name: "PE", format: ExtractFormat::Pe, extension: "exe", category: "可执行文件" },
        
        // 固件格式
        FormatInfo { name: "UEFIc", format: ExtractFormat::Uefic, extension: "scap", category: "固件" },
        FormatInfo { name: "UEFIf", format: ExtractFormat::Uefif, extension: "uefif", category: "固件" },
        FormatInfo { name: "TE", format: ExtractFormat::Te, extension: "te", category: "固件" },
        
        // 分区格式
        FormatInfo { name: "GPT", format: ExtractFormat::Gpt, extension: "gpt", category: "分区" },
        FormatInfo { name: "MBR", format: ExtractFormat::Mbr, extension: "mbr", category: "分区" },
        FormatInfo { name: "APM", format: ExtractFormat::Apm, extension: "apm", category: "分区" },
        
        // 其他格式
        FormatInfo { name: "CHM", format: ExtractFormat::Chm, extension: "chm", category: "其他" },
        FormatInfo { name: "Xar", format: ExtractFormat::Xar, extension: "xar", category: "其他" },
        FormatInfo { name: "Ar", format: ExtractFormat::Ar, extension: "a", category: "其他" },
        FormatInfo { name: "Compound", format: ExtractFormat::Compound, extension: "msi", category: "其他" },
        FormatInfo { name: "Base64", format: ExtractFormat::Base64, extension: "b64", category: "其他" },
        FormatInfo { name: "COFF", format: ExtractFormat::Coff, extension: "obj", category: "其他" },
        FormatInfo { name: "IHex", format: ExtractFormat::IHex, extension: "ihex", category: "其他" },
        FormatInfo { name: "Mub", format: ExtractFormat::Mub, extension: "mub", category: "其他" },
        FormatInfo { name: "LP", format: ExtractFormat::LP, extension: "lpimg", category: "其他" },
        FormatInfo { name: "Hxs", format: ExtractFormat::Hxs, extension: "hxs", category: "其他" },
    ];

    println!("测试目录：test_debug/\n");
    println!("共 {} 种格式待测试...\n", formats.len());

    let mut results: Vec<TestResult> = Vec::new();
    let mut category_stats: HashMap<String, (usize, usize)> = HashMap::new();

    // 按类别分组测试
    let categories: Vec<String> = formats.iter()
        .map(|f| f.category.to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    for category in &categories {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║  类别：{:<50} ║", category);
        println!("╚══════════════════════════════════════════════════════════╝\n");

        let category_formats: Vec<&FormatInfo> = formats.iter()
            .filter(|f| f.category == category.as_str())
            .collect();

        for format_info in &category_formats {
            let result = test_format(&lib, format_info);
            print_result(&result);
            
            // 统计
            let entry = category_stats.entry(category.clone()).or_insert((0, 0));
            entry.1 += 1;
            if result.extract_success {
                entry.0 += 1;
            }
            
            results.push(result);
        }
    }

    // 生成汇总报告
    print_summary(&results, &category_stats);
    
    // 生成 HTML 报告
    generate_html_report(&results);
}

fn test_format(lib: &BitLibrary, format_info: &FormatInfo) -> TestResult {
    let archive_path = format!("test_debug/test.{}", format_info.extension);
    let output_dir = format!("test_debug/extracted_{}", format_info.name.to_lowercase());
    
    // 检查文件是否存在
    let file_exists = Path::new(&archive_path).exists();
    
    if !file_exists {
        return TestResult {
            format_name: format_info.name.to_string(),
            extension: format_info.extension.to_string(),
            file_exists: false,
            extract_success: false,
            error_message: Some("测试文件不存在".to_string()),
            extracted_files: vec![],
        };
    }
    
    // 清理输出目录
    let _ = fs::remove_dir_all(&output_dir);
    
    // 尝试解压
    let extractor = BitExtractor::new(lib, format_info.format);
    let extract_result = extractor.extract(&archive_path, &output_dir);
    
    let (extract_success, error_message, extracted_files) = match extract_result {
        Ok(_) => {
            // 检查解压的文件
            let files = list_extracted_files(&output_dir);
            (true, None, files)
        }
        Err(e) => {
            (false, Some(e.to_string()), vec![])
        }
    };
    
    TestResult {
        format_name: format_info.name.to_string(),
        extension: format_info.extension.to_string(),
        file_exists: true,
        extract_success,
        error_message,
        extracted_files,
    }
}

fn list_extracted_files(dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                files.push(name.to_string());
            }
        }
    }
    files
}

fn print_result(result: &TestResult) {
    let status = if result.file_exists && result.extract_success {
        "✓"
    } else if !result.file_exists {
        "⊘"
    } else {
        "✗"
    };
    
    print!("  [{}] {:<12} ({:<6})", status, result.format_name, result.extension);
    
    if !result.file_exists {
        println!(" - 文件不存在");
    } else if result.extract_success {
        println!(" - 解压成功 ({} 个文件)", result.extracted_files.len());
    } else {
        if let Some(ref err) = result.error_message {
            // 缩短错误消息
            let short_err = if err.len() > 60 {
                format!("{}...", &err[..57])
            } else {
                err.clone()
            };
            println!(" - 失败：{}", short_err);
        }
    }
}

fn print_summary(results: &[TestResult], category_stats: &HashMap<String, (usize, usize)>) {
    println!("\n\n╔══════════════════════════════════════════════════════════╗");
    println!("║                    测试结果汇总                        ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    let total = results.len();
    let success = results.iter().filter(|r| r.extract_success).count();
    let missing = results.iter().filter(|r| !r.file_exists).count();
    let failed = results.iter().filter(|r| r.file_exists && !r.extract_success).count();
    
    println!("总体统计:");
    println!("  总格式数：{}", total);
    println!("  解压成功：{} ({:.1}%)", success, if total > 0 { (success as f64 / total as f64) * 100.0 } else { 0.0 });
    println!("  文件缺失：{}", missing);
    println!("  解压失败：{}\n", failed);
    
    println!("按类别统计:");
    for (category, (success_count, total_count)) in category_stats {
        let rate = (*success_count as f64 / *total_count as f64) * 100.0;
        println!("  {:<15} {}/{} ({:.1}%)", category, success_count, total_count, rate);
    }
    
    // 列出失败的格式
    let failed_results: Vec<&TestResult> = results.iter()
        .filter(|r| r.file_exists && !r.extract_success)
        .collect();
    
    if !failed_results.is_empty() {
        println!("\n失败的格式:");
        for result in &failed_results {
            if let Some(ref err) = result.error_message {
                println!("  - {}: {}", result.format_name, err);
            }
        }
    }
    
    // 列出缺失测试文件的格式
    let missing_results: Vec<&TestResult> = results.iter()
        .filter(|r| !r.file_exists)
        .collect();
    
    if !missing_results.is_empty() {
        println!("\n缺失测试文件的格式:");
        for result in &missing_results {
            println!("  - {} (.{}", result.format_name, result.extension);
        }
    }
}

fn generate_html_report(results: &[TestResult]) {
    let html_path = "test_debug/test_report.html";
    
    let mut html = String::from(r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>bit7z-rust 格式测试报告</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; background: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        h1 { color: #333; border-bottom: 2px solid #4CAF50; padding-bottom: 10px; }
        h2 { color: #555; margin-top: 30px; }
        table { width: 100%; border-collapse: collapse; margin: 20px 0; }
        th, td { padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }
        th { background: #4CAF50; color: white; }
        tr:hover { background: #f5f5f5; }
        .success { color: #4CAF50; font-weight: bold; }
        .failure { color: #f44336; font-weight: bold; }
        .missing { color: #ff9800; font-weight: bold; }
        .summary { display: flex; gap: 20px; margin: 20px 0; }
        .summary-card { flex: 1; padding: 20px; border-radius: 8px; text-align: center; }
        .summary-card.success { background: #e8f5e9; }
        .summary-card.failure { background: #ffebee; }
        .summary-card.missing { background: #fff3e0; }
        .summary-card h3 { margin: 0; font-size: 2em; }
        .summary-card p { margin: 5px 0 0 0; color: #666; }
        .error-msg { color: #f44336; font-size: 0.9em; }
    </style>
</head>
<body>
    <div class="container">
        <h1>📊 bit7z-rust 格式测试报告</h1>
"#);

    // 统计
    let total = results.len();
    let success = results.iter().filter(|r| r.extract_success).count();
    let missing = results.iter().filter(|r| !r.file_exists).count();
    let failed = results.iter().filter(|r| r.file_exists && !r.extract_success).count();

    html.push_str(&format!(r#"
        <div class="summary">
            <div class="summary-card success">
                <h3>{}</h3>
                <p>成功</p>
            </div>
            <div class="summary-card failure">
                <h3>{}</h3>
                <p>失败</p>
            </div>
            <div class="summary-card missing">
                <h3>{}</h3>
                <p>缺失文件</p>
            </div>
        </div>
        <p>总格式数：{} | 成功率：{:.1}%</p>
"#, success, failed, missing, total, if total > 0 { (success as f64 / total as f64) * 100.0 } else { 0.0 }));

    html.push_str(r#"
        <h2>详细结果</h2>
        <table>
            <tr>
                <th>格式</th>
                <th>扩展名</th>
                <th>状态</th>
                <th>详情</th>
            </tr>
"#);

    for result in results {
        let (status_class, status_text) = if result.extract_success {
            ("success", "✓ 成功")
        } else if !result.file_exists {
            ("missing", "⊘ 文件缺失")
        } else {
            ("failure", "✗ 失败")
        };

        let detail = if result.extract_success {
            format!("解压了 {} 个文件", result.extracted_files.len())
        } else if let Some(ref err) = result.error_message {
            err.clone()
        } else {
            String::new()
        };

        html.push_str(&format!(r#"
            <tr>
                <td>{}</td>
                <td>{}</td>
                <td class="{}">{}</td>
                <td class="error-msg">{}</td>
            </tr>
"#, result.format_name, result.extension, status_class, status_text, detail));
    }

    html.push_str(r#"
        </table>
    </div>
</body>
</html>
"#);

    if let Err(e) = fs::write(html_path, html) {
        eprintln!("生成 HTML 报告失败：{}", e);
    } else {
        println!("\n✓ HTML 报告已生成：{}", html_path);
    }
}
