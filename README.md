# bit7z-rust

> [!WARNING]
> **Experimental & Unstable**: This project is a Rust port of the [bit7z](https://github.com/rikyoz/bit7z) C++ library. It is currently in an early stage of development and is **highly unstable**. Use at your own risk.

A Rust wrapper for 7-Zip shared libraries (`7z.dll`, `7z.so`, `7z.dylib`).

This library provides a safe (work in progress), idiomatic Rust interface to 7-Zip compression library, supporting multiple archive formats including 7z, ZIP, GZIP, BZIP2, TAR, XZ, and WIM for compression, and many more for extraction.

## Features

- **Compression**: Support for 7z, ZIP, GZIP, BZIP2, TAR, XZ, and WIM.
- **Extraction**: Support for all common formats (including RAR, RAR5, ISO, CAB, etc.).
- **Archive Metadata**: List items, read properties, and metadata without extraction.
- **Archive Editing**: Rename, update, and delete items in existing archives.
- **Memory & Streams**: Compress/extract directly to/from memory buffers or streams.
- **Advanced Features**: Encryption (AES-256), multi-volume archives, solid compression.
- **Callbacks**: Progress tracking, password requests, and file processing notifications.

## Documentation

- [使用指南 (Usage Guide)](docs/usage-guide.md)
- [API 参考 (API Reference)](docs/api-reference.md)

## Quick Start

### 1. Load 7-Zip Library

```rust
use bit7z_rust::BitLibrary;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Automatically find and load 7-Zip shared library
    let lib = BitLibrary::new::<String>(None)?;
    Ok(())
}
```

### 2. Compress Files

```rust
use bit7z_rust::{BitCompressor, CompressionFormat};

let mut compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
compressor.compress(&["file1.txt", "docs/"], "output.7z")?;
```

### 3. Extract Archive

```rust
use bit7z_rust::{BitExtractor, ExtractFormat};

let mut extractor = BitExtractor::new(&lib, ExtractFormat::SevenZip);
extractor.extract("output.7z", "extracted_dir/")?;
```

## Requirements

- **Windows**: `7z.dll` (usually in `C:\Program Files\7-Zip\`)
- **Linux**: `7z.so` (usually in `/usr/lib/7zip/`)
- **macOS**: `7z.dylib` (available via Homebrew: `brew install 7zip`)

## AI Disclaimer

> [!IMPORTANT]
> This documentation, including API references, usage guides, and code examples, was generated with the assistance of AI. While every effort has been made to ensure accuracy and reliability, AI-generated content may have limitations or errors. Users are encouraged to verify the information and perform thorough testing before using this library in a production environment.
> 
> 本项目的文档（包括 API 参考、使用指南）以及示例代码由 AI 辅助生成。虽然我们努力确保内容的准确性与可靠性，但 AI 生成的内容可能存在局限性或偏差。在生产环境中使用本库前，请务必进行充分的测试与验证。

## License

This project is licensed under the MPL 2.0 License.
