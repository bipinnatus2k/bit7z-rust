# 存档编辑与删除

本章介绍如何使用 `BitArchiveEditor` 修改现有存档，包括重命名项目、更新内容以及从存档中删除项目。

## 1. 创建编辑器

创建一个新的 `BitArchiveEditor` 实例。

```rust
use bit7z_rust::{BitLibrary, BitArchiveEditor, CompressionFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lib = BitLibrary::new::<String>(None)?;
    let mut editor = BitArchiveEditor::new(
        &lib, 
        "backup.7z", 
        CompressionFormat::SevenZip, 
        None // 如果存档已加密，请在此提供密码
    )?;
    
    // 设置压缩参数
    editor.compression_level(crate::format::CompressionLevel::Normal);
    
    Ok(())
}
```

## 2. 重命名项目

重命名存档中的文件或目录。

```rust
// 重命名索引为 0 的项目
editor.rename_item(0, "docs/new_readme.txt".to_string())?;
```

## 3. 更新项目内容

您可以从文件、内存缓冲区或流中更新存档中项目的内容。

```rust
// 从本地文件更新内容
editor.update_item(1, "updated_file.txt")?;

// 从内存缓冲区更新内容
editor.update_item_from_buffer(2, b"Updated content from memory".to_vec())?;

// 从流更新内容
use std::io::Cursor;
let mut stream = Cursor::new(b"Stream data to update");
editor.update_item_from_stream(3, &mut stream, "data_from_stream.bin".to_string())?;
```

## 4. 删除项目

按索引或路径从存档中删除项目。

```rust
use bit7z_rust::DeletePolicy;

// 仅删除索引为 4 的文件
editor.delete_item(4, DeletePolicy::ItemOnly)?;

// 递归删除指定路径的目录及其所有内容
editor.delete_item_by_path("old_dir/", DeletePolicy::RecurseDirs)?;
```

## 5. 应用更改并保存

所有的修改操作都是“挂起”的，必须调用 `apply_changes` 才能实际执行并将结果保存回原始文件。

```rust
// 实际执行所有挂起的修改、重命名和删除操作
editor.apply_changes()?;

println!("存档已更新");
```

## 6. 应用更改到内存

如果您不想修改原始文件，而是想获得修改后的存档数据的字节数组，可以使用 `apply_changes_to_buffer`。

```rust
// 返回修改后的存档数据，而不修改原始文件
let modified_bytes: Vec<u8> = editor.apply_changes_to_buffer()?;
```

## 7. 删除策略 (DeletePolicy)

| 枚举值 | 说明 |
| :--- | :--- |
| `ItemOnly` | 仅删除指定的单个项目（适用于文件）。 |
| `RecurseDirs` | 递归删除目录及其所有子项。 |

## 注意事项

- **备份**: 强烈建议在进行大规模编辑之前备份您的原始存档文件。
- **临时文件**: 存档编辑过程中内部会创建临时文件和工作目录，并在操作完成后自动清理。
- **性能**: 编辑大型存档可能会很耗时，因为这通常涉及解压部分内容、修改并重新压缩整个存档。
