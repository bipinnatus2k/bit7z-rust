# 压缩与解压格式

`bit7z-rust` 支持多种存档格式。为了区分仅支持解压的格式和同时支持压缩与解压的格式，库中定义了两个不同的枚举。

## 压缩格式 (CompressionFormat)

`CompressionFormat` 包含所有支持**创建（压缩）**的格式。

```rust
pub enum CompressionFormat {
    SevenZip,  // 7z
    Zip,       // ZIP
    GZip,      // GZIP
    BZip2,     // BZIP2
    Tar,       // TAR
    Xz,        // XZ
    Wim,       // WIM
}
```

## 解压格式 (ExtractFormat)

`ExtractFormat` 包含所有支持**读取（解压）**的格式，包括许多只读格式（如 RAR）。

```rust
pub enum ExtractFormat {
    SevenZip, Zip, GZip, BZip2, Tar, Xz, Wim, // 同时支持压缩的格式
    Rar, Rar5, Arj, Lzh, Cab, Nsis, Lzma, Iso, Udf, Chm, // 只读格式
    Dmg, Ext, Fat, Hfs, Ntfs, // 文件系统镜像
    Vdi, Vhd, Vhdx, Vmdk, // 虚拟机磁盘镜像
    // ... 以及更多
}
```

## 压缩级别 (CompressionLevel)

用于控制压缩速度与压缩率的权衡。

```rust
pub enum CompressionLevel {
    None,      // 不压缩 (Store)
    Fastest,   // 最快速度
    Fast,      // 快速
    Normal,    // 标准 (默认)
    Maximum,   // 最大压缩
    Ultra,     // 极致压缩
}
```

## 压缩方法 (CompressionMethod)

指定底层的压缩算法。

```rust
pub enum CompressionMethod {
    Lzma,
    Lzma2,
    Ppmd,
    BZip2,
    Deflate,
    Deflate64,
    Copy,
}
```

## 转换

`CompressionFormat` 可以方便地转换为 `ExtractFormat`：

```rust
let compress_fmt = CompressionFormat::SevenZip;
let extract_fmt: ExtractFormat = compress_fmt.into();
```
