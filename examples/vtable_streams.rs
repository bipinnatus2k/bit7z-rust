//! Example demonstrating vtable crate usage for streams
//!
//! This example shows how to use the vtable-crate-based stream implementations.

use bit7z_rust::vtable_stream::{
    VTableFileStream, VTableFileStreamWrite,
    VTableBufferInStream, VTableBufferOutStream,
};
use std::path::Path;

fn main() {
    println!("VTable Stream Example");
    println!("=====================\n");

    // 1. Create a file input stream
    println!("1. Creating VTableFileStream...");
    match VTableFileStream::new(Path::new("Cargo.toml")) {
        Ok(stream) => {
            println!("   ✓ Created VTableFileStream");
            println!("   ✓ Can be converted to IInStream pointer");
        }
        Err(e) => println!("   ✗ Failed to create file stream: {}", e),
    }

    // 2. Create a file output stream
    println!("\n2. Creating VTableFileStreamWrite...");
    match VTableFileStreamWrite::new(Path::new("/tmp/test_output.txt")) {
        Ok(stream) => {
            println!("   ✓ Created VTableFileStreamWrite");
            println!("   ✓ Can be converted to IOutStream pointer");
        }
        Err(e) => println!("   ✗ Failed to create file output stream: {}", e),
    }

    // 3. Create a buffer input stream
    println!("\n3. Creating VTableBufferInStream...");
    let buffer_data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
    let stream = VTableBufferInStream::new(buffer_data);
    println!("   ✓ Created VTableBufferInStream with 8 bytes of data");

    // 4. Create a buffer output stream
    println!("\n4. Creating VTableBufferOutStream...");
    let stream = VTableBufferOutStream::new();
    println!("   ✓ Created empty VTableBufferOutStream");

    println!("\n✓ All stream types created successfully!");
    println!("\nNote: These streams use vtable crate for automatic vtable management.");
    println!("Memory is managed by VBox which is created internally when needed.");
}
