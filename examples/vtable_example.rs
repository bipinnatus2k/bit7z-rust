//! Example demonstrating vtable crate usage for OpenCallback
//!
//! This example shows how to use the vtable-crate-based VTableOpenCallback
//! instead of the manual implementation.

use bit7z_rust::vtable_callback::VTableOpenCallback;
use bit7z_rust::vtable_base::VBox;
use std::path::Path;

fn main() {
    println!("VTable Crate Example");
    println!("====================\n");

    // Create a VTableOpenCallback using vtable crate
    let callback = VTableOpenCallback::new(Path::new("test.7z"));
    
    println!("Created VTableOpenCallback for archive: test.7z");
    
    // Wrap in VBox for automatic vtable management
    let boxed: VBox<bit7z_rust::vtable_callback::OpenCallbackVTable> = VBox::new(callback);
    
    println!("  Wrapped in VBox");
    println!("  Reference count: 1");
    
    // Get raw pointer for FFI use
    let raw_ptr = &*boxed as *const _ as *const _;
    println!("  Raw pointer: {:p}", raw_ptr);
    
    // Note: In real usage, you would pass this to 7-Zip's Open method
    // The VBox will automatically manage the vtable and cleanup
    
    println!("\nExample completed successfully!");
    println!("Note: This is a demonstration. Real usage requires 7-Zip library.");
}
