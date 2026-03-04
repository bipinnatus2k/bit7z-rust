# BitArchiveEditor

`BitArchiveEditor` 用于修改现有的存档文件，包括重命名项目、更新内容以及删除项目。

## 结构体定义

```rust
pub struct BitArchiveEditor<'a> {
    // 内部字段
}

pub enum DeletePolicy {
    ItemOnly,      // 仅删除指定项目
    RecurseDirs,   // 递归删除目录及其所有内容
}
```

## 创建方法

### `new`

创建一个新的编辑器实例。

```rust
pub fn new<P: AsRef<Path>>(
    library: &'a BitLibrary,
    archive_path: P,
    format: CompressionFormat,
    password: Option<String>,
) -> Result<Self>
```

## 修改方法

### `rename_item`

重命名存档中的项目。

```rust
pub fn rename_item(&mut self, index: u32, new_path: String) -> Result<()>
```

### `update_item`

从文件更新存档中项目的内容。

```rust
pub fn update_item<P: AsRef<Path>>(&mut self, index: u32, file_path: P) -> Result<()>
```

### `update_item_from_buffer`

从内存缓冲区更新存档中项目的内容。

```rust
pub fn update_item_from_buffer(&mut self, index: u32, buffer: Vec<u8>) -> Result<()>
```

### `delete_item`

从存档中删除项目。

```rust
pub fn delete_item(&mut self, index: u32, policy: DeletePolicy) -> Result<()>
```

### `delete_item_by_path`

按路径从存档中删除项目。

```rust
pub fn delete_item_by_path(&mut self, item_path: &str, policy: DeletePolicy) -> Result<()>
```

## 应用更改

### `apply_changes`

应用所有挂起的更改并更新原始存档文件。

```rust
pub fn apply_changes(&mut self) -> Result<()>
```

### `apply_changes_to_buffer`

应用所有更改并返回修改后的存档数据作为内存缓冲区，而不修改原始文件。

```rust
pub fn apply_changes_to_buffer(&mut self) -> Result<Vec<u8>>
```

## 示例

```rust
use bit7z_rust::{BitLibrary, BitArchiveEditor, CompressionFormat, DeletePolicy};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lib = BitLibrary::new::<String>(None)?;
    let mut editor = BitArchiveEditor::new(&lib, "archive.7z", CompressionFormat::SevenZip, None)?;
    
    // 重命名
    editor.rename_item(0, "new_name.txt".to_string())?;
    
    // 删除
    editor.delete_item_by_path("old_dir/", DeletePolicy::RecurseDirs)?;
    
    // 更新内容
    editor.update_item(1, "updated_content.txt")?;
    
    // 提交更改
    editor.apply_changes()?;
    
    Ok(())
}
```
