# 内存与流操作

除了对本地文件进行操作，`bit7z-rust` 还支持直接处理内存缓冲区（Buffer）和流（Stream）。这在处理 Web 上传、数据库 blob 或网络流时非常有用。

## 1. 内存压缩与解压

### 内存压缩 (BitMemCompressor)

`BitMemCompressor` 允许您将内存中的字节数组直接压缩为存档格式的字节数组。

```rust
use bit7z_rust::{BitLibrary, BitMemCompressor, CompressionFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lib = BitLibrary::new::<String>(None)?;
    let mut compressor = BitMemCompressor::new(&lib, CompressionFormat::Zip);
    
    let data = b"Hello, this is some data to compress";
    
    // 压缩数据，并指定存档中的文件名
    let compressed_bytes: Vec<u8> = compressor.compress_from_buffer(
        data, 
        Some("hello.txt".to_string())
    )?;
    
    println!("原始大小: {}, 压缩后大小: {}", data.len(), compressed_bytes.len());
    Ok(())
}
```

### 内存解压 (BitMemExtractor)

`BitMemExtractor` 允许您从内存中的存档数据直接解压出文件内容。

```rust
use bit7z_rust::{BitLibrary, BitMemExtractor, ExtractFormat};

let extractor = BitMemExtractor::new(&lib, ExtractFormat::Zip);

// 将内存中的 ZIP 数据解压出索引为 0 的文件
let file_data: Vec<u8> = extractor.extract_to_buffer(&compressed_bytes, 0)?;
```

## 2. 基于流的操作

`BitStreamCompressor` 和 `BitStreamExtractor` 支持实现 `std::io::Read` 和 `std::io::Write` 接口的流。

### 流压缩示例

```rust
use bit7z_rust::{BitLibrary, BitStreamCompressor, CompressionFormat};
use std::io::Cursor;

let mut compressor = BitStreamCompressor::new(&lib, CompressionFormat::SevenZip);

let mut input = Cursor::new(b"Stream data content");
let mut output = Vec::new(); // Vec 实现了 Write 接口

compressor.compress_stream(&mut input, &mut output, "data.bin")?;
```

### 流解压示例

```rust
use bit7z_rust::{BitLibrary, BitStreamExtractor, ExtractFormat};

let mut extractor = BitStreamExtractor::new(&lib, ExtractFormat::SevenZip);
let mut archive_stream = Cursor::new(compressed_bytes);
let mut out_data = Vec::new();

// 从流中解压索引为 0 的项目
extractor.extract_item_to_stream(&mut archive_stream, 0, &mut out_data)?;
```

## 注意事项

- **临时文件**: 目前的内存与流实现内部可能会使用临时文件（由 `tempfile` 库管理）来绕过 7-Zip 原生接口的一些限制。这些临时文件在操作完成后会自动清理。
- **内存占用**: 压缩大文件到内存时请注意内存限制，避免 `OutOfMemory` 错误。
