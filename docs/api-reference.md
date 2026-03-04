# API 参考文档

本项目提供的公开 API 按照功能模块划分如下：

## 核心模块

- [BitLibrary](api-reference/library.md): 7-Zip 共享库加载与管理
- [BitCompressor](api-reference/compressor.md): 文件压缩功能
- [BitExtractor](api-reference/extractor.md): 文件解压功能
- [BitArchiveReader](api-reference/reader.md): 存档读取与信息查询
- [BitArchiveEditor](api-reference/editor.md): 存档编辑与删除

## 内存与流操作

- [BitMemCompressor](api-reference/mem-compressor.md): 内存数据压缩
- [BitMemExtractor](api-reference/mem-extractor.md): 内存数据解压
- [BitStreamCompressor](api-reference/stream-compressor.md): 基于流的压缩
- [BitStreamExtractor](api-reference/stream-extractor.md): 基于流的解压

## 辅助与配置

- [CompressionFormat](api-reference/formats.md): 压缩与解压格式定义
- [Bit7zError](api-reference/errors.md): 错误类型与语义说明
- [Callbacks](api-reference/callbacks.md): 进度、密码等回调接口
