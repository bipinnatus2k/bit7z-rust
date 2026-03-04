# 进度追踪与回调

本章介绍如何在 `bit7z-rust` 中使用回调函数来追踪进度、处理密码请求以及记录操作日志。

## 1. 进度追踪

通过 `set_progress_callback`，您可以获取已处理的字节数和总字节数。

```rust
use bit7z_rust::{BitLibrary, BitCompressor, CompressionFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lib = BitLibrary::new::<String>(None)?;
    let mut compressor = BitCompressor::new(&lib, CompressionFormat::SevenZip);
    
    // 设置进度回调
    compressor.set_progress_callback(|processed, total| {
        if total > 0 {
            let percent = (processed as f64 / total as f64) * 100.0;
            println!("当前进度: {:.2}% ({} / {})", percent, processed, total);
        }
        true // 返回 true 继续，返回 false 则取消操作
    });
    
    compressor.compress(&["large_folder/"], "backup.7z")?;
    Ok(())
}
```

## 2. 文件处理日志

使用 `set_file_callback` 在处理每个文件前执行操作。

```rust
compressor.set_file_callback(|file_path| {
    println!("正在处理文件: {}", file_path);
});
```

## 3. 动态密码请求

如果您不想提前在代码中硬编码密码，可以使用 `set_password_callback` 在需要时动态获取。

```rust
use bit7z_rust::{BitLibrary, BitExtractor, ExtractFormat};

let mut extractor = BitExtractor::new(&lib, ExtractFormat::SevenZip);

extractor.set_password_callback(|| {
    // 这里可以弹出对话框或从终端读取
    println!("请输入存档密码:");
    let mut password = String::new();
    std::io::stdin().read_line(&mut password).unwrap();
    password.trim().to_string()
});

extractor.extract("encrypted.7z", "out/")?;
```

## 4. 线程安全注意事项

所有的回调接口都要求实现 `Send + Sync`。如果您需要在回调中修改外部状态，请使用 `Arc<Mutex<...>>` 或 `Arc<Atomic...>`。

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
let counter_clone = Arc::clone(&counter);

compressor.set_file_callback(move |_| {
    let mut num = counter_clone.lock().unwrap();
    *num += 1;
});
```

## 5. 比例回调 (Ratio Callback)

对于某些格式，您可以追踪输入与输出的压缩比例。

```rust
compressor.set_ratio_callback(|in_size, out_size| {
    println!("输入: {} -> 输出: {}", in_size, out_size);
});
```
