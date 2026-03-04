# BitLibrary

`BitLibrary` 是 `bit7z-rust` 的核心入口点，负责加载 7-Zip 共享库（如 `7z.dll` 或 `7z.so`）并管理其生命周期。

## 结构体定义

```rust
pub struct BitLibrary {
    // 内部字段
}
```

## 公开方法

### `new`

加载 7-Zip 共享库。

```rust
pub fn new<P: AsRef<Path>>(path: Option<P>) -> Result<Self, LibraryError>
```

- **参数**:
    - `path`: 可选。指定 7-Zip 共享库的路径。如果为 `None`，将按以下顺序查找：
        1. 环境变量 `BIT7Z_LIBRARY_PATH`
        2. 系统特定默认路径（如 Windows 下的 `C:\Program Files\7-Zip\7z.dll`）
        3. 当前工作目录或库搜索路径下的 `7z.dll`/`7z.so`/`7z.dylib`
- **返回值**:
    - `Ok(BitLibrary)`: 成功加载库。
    - `Err(LibraryError)`: 加载失败，如文件不存在或缺少必要符号。

### `set_large_page_mode`

启用大页面模式（如果库支持且系统权限允许）。

```rust
pub fn set_large_page_mode(&self) -> Result<(), LibraryError>
```

- **返回值**:
    - `Ok(())`: 成功或库不支持该功能。
    - `Err(LibraryError)`: 调用失败。

## 错误类型

### `LibraryError`

```rust
pub enum LibraryError {
    LoadFailed(libloading::Error),     // 加载库文件失败
    SymbolNotFound(String),           // 缺少 CreateObject 符号
    CreateFailed(HRESULT),            // 创建对象返回错误码
    NullPointer,                      // 返回了空指针
}
```

## 示例

```rust
use bit7z_rust::BitLibrary;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 自动查找并加载
    let lib = BitLibrary::new::<String>(None)?;
    
    // 或者指定路径
    // let lib = BitLibrary::new(Some("path/to/7z.dll"))?;
    
    Ok(())
}
```
