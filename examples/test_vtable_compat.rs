//! Test to verify vtable layout compatibility with C++

use std::ffi::c_void;
use std::mem;

use bit7z_rust::ffi::{
    IUnknown, IUnknownVTable,
    IProgress, IProgressVTable,
    IArchiveUpdateCallback, IArchiveUpdateCallbackVTable,
    IArchiveUpdateCallback2, IArchiveUpdateCallback2VTable,
    ICompressProgressInfo, ICompressProgressInfoVTable,
    ICryptoGetTextPassword, ICryptoGetTextPasswordVTable,
    ICryptoGetTextPassword2, ICryptoGetTextPassword2VTable,
    IID_IUnknown, IID_IProgress, IID_IArchiveUpdateCallback,
    IID_IArchiveUpdateCallback2, IID_ICompressProgressInfo,
    IID_ICryptoGetTextPassword, IID_ICryptoGetTextPassword2,
};

fn main() {
    println!("=== Rust 虚函数表布局验证 ===\n");
    
    // 验证指针大小
    let ptr_size = mem::size_of::<*const c_void>();
    println!("指针大小：{} 字节\n", ptr_size);
    
    // 验证各虚函数表大小
    println!("虚函数表大小:");
    println!("  IUnknownVTable:               {} 字节", mem::size_of::<IUnknownVTable>());
    println!("  IProgressVTable:              {} 字节", mem::size_of::<IProgressVTable>());
    println!("  IArchiveUpdateCallbackVTable: {} 字节", mem::size_of::<IArchiveUpdateCallbackVTable>());
    println!("  IArchiveUpdateCallback2VTable: {} 字节", mem::size_of::<IArchiveUpdateCallback2VTable>());
    println!("  ICompressProgressInfoVTable:  {} 字节", mem::size_of::<ICompressProgressInfoVTable>());
    println!("  ICryptoGetTextPasswordVTable: {} 字节", mem::size_of::<ICryptoGetTextPasswordVTable>());
    println!("  ICryptoGetTextPassword2VTable: {} 字节", mem::size_of::<ICryptoGetTextPassword2VTable>());
    println!();
    
    // 计算预期的偏移量
    println!("预期的接口指针偏移:");
    println!("  IUnknown*:                offset = 0");
    println!("  IProgress*:               offset = {}", ptr_size);
    println!("  IArchiveUpdateCallback*:  offset = {}", ptr_size * 2);
    println!("  IArchiveUpdateCallback2*: offset = {}", ptr_size * 3);
    println!("  ICompressProgressInfo*:   offset = {}", ptr_size * 4);
    println!("  ICryptoGetTextPassword*:  offset = {}", ptr_size * 5);
    println!("  ICryptoGetTextPassword2*: offset = {}", ptr_size * 6);
    println!();
    
    // 验证 GUID 值
    println!("GUID 验证:");
    println!("  IID_IUnknown:                {:08X}-{:04X}-{:04X}-{:02X?}", 
             IID_IUnknown.data1, IID_IUnknown.data2, IID_IUnknown.data3, IID_IUnknown.data4);
    println!("  IID_IProgress:               {:08X}-{:04X}-{:04X}-{:02X?}", 
             IID_IProgress.data1, IID_IProgress.data2, IID_IProgress.data3, IID_IProgress.data4);
    println!("  IID_IArchiveUpdateCallback:  {:08X}-{:04X}-{:04X}-{:02X?}", 
             IID_IArchiveUpdateCallback.data1, IID_IArchiveUpdateCallback.data2, 
             IID_IArchiveUpdateCallback.data3, IID_IArchiveUpdateCallback.data4);
    println!("  IID_IArchiveUpdateCallback2: {:08X}-{:04X}-{:04X}-{:02X?}", 
             IID_IArchiveUpdateCallback2.data1, IID_IArchiveUpdateCallback2.data2,
             IID_IArchiveUpdateCallback2.data3, IID_IArchiveUpdateCallback2.data4);
    println!("  IID_ICompressProgressInfo:   {:08X}-{:04X}-{:04X}-{:02X?}", 
             IID_ICompressProgressInfo.data1, IID_ICompressProgressInfo.data2,
             IID_ICompressProgressInfo.data3, IID_ICompressProgressInfo.data4);
    println!();
    
    println!("=== 验证完成 ===");
}
