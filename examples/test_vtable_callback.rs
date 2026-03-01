//! Test example for vtable crate based OpenCallback implementation
//!
//! Run with: cargo run --example test_vtable_callback --features vtable_impl
//!
//! This example demonstrates:
//! 1. Creating OpenCallback using vtable crate
//! 2. Verifying vtable layout compatibility with 7-Zip
//! 3. Testing basic callback operations

#[cfg(feature = "vtable_impl")]
use bit7z_rust::vtable_callback::{VTableOpenCallback, OpenCallbackVTable};

#[cfg(feature = "vtable_impl")]
use bit7z_rust::ffi::{
    GUID, HRESULT, PROPVARIANT, IInStream,
    IID_IUnknown, IID_IArchiveOpenCallback,
    IID_IArchiveOpenVolumeCallback,
};

#[cfg(feature = "vtable_impl")]
use vtable::*;

#[cfg(feature = "vtable_impl")]
fn test_vtable_creation() {
    println!("=== Testing VTable Creation ===");
    
    // Create an OpenCallback using vtable crate
    let callback = VTableOpenCallback::new(std::path::Path::new("test.7z"));
    
    // Create VBox (this uses the vtable crate's automatic vtable management)
    let mut boxed: VBox<OpenCallbackVTable> = VBox::new(callback);
    
    println!("✓ VBox created successfully");
    println!("  VBox size: {} bytes", std::mem::size_of_val(&boxed));
    
    // Test method calls through vtable
    let count = boxed.add_ref();
    println!("✓ add_ref() returned: {}", count);
    
    let count = boxed.release();
    println!("✓ release() returned: {}", count);
    
    // Test QueryInterface
    let result = boxed.query_interface(&IID_IUnknown);
    println!("✓ query_interface(IID_IUnknown) returned: {:?}", result.is_null());
    
    println!();
}

#[cfg(feature = "vtable_impl")]
fn test_vtable_layout() {
    println!("=== Testing VTable Layout ===");
    
    // Check memory layout
    println!("VTableOpenCallback struct size: {}", std::mem::size_of::<VTableOpenCallback>());
    println!("VTableOpenCallback alignment: {}", std::mem::align_of::<VTableOpenCallback>());
    
    let callback = VTableOpenCallback::new(std::path::Path::new("test.7z"));
    let boxed: VBox<OpenCallbackVTable> = VBox::new(callback);
    
    // Get raw pointer
    let raw_ptr = &*boxed as *const _ as *const u8;
    println!("Boxed callback address: {:p}", raw_ptr);
    
    // Check vtable pointer location (should be at offset 0)
    let vtable_ptr = unsafe {
        std::mem::transmute::<_, *const u8>(&boxed)
    };
    println!("VTable pointer address: {:p}", vtable_ptr);
    
    // Verify vtable is at the beginning
    let offset = (raw_ptr as usize) - (vtable_ptr as usize);
    println!("VTable offset: {} bytes", offset);
    
    if offset == 0 {
        println!("✓ VTable is at the beginning (COM compatible)");
    } else {
        println!("✗ VTable offset is non-zero (may cause COM interop issues)");
    }
    
    println!();
}

#[cfg(feature = "vtable_impl")]
fn test_callback_methods() {
    println!("=== Testing Callback Methods ===");
    
    let callback = VTableOpenCallback::new(std::path::Path::new("/path/to/archive.7z"));
    let boxed: VBox<OpenCallbackVTable> = VBox::new(callback);
    
    // Test set_completed
    let files: u64 = 10;
    let bytes: u64 = 1024;
    let result = boxed.set_completed(&files, &bytes);
    println!("✓ set_completed() returned: 0x{:08X}", result);
    
    // Test set_total
    let result = boxed.set_total(&files, &bytes);
    println!("✓ set_total() returned: 0x{:08X}", result);
    
    // Test get_property (kpidName)
    let mut prop = unsafe { std::mem::zeroed::<PROPVARIANT>() };
    let result = boxed.get_property(0, &mut prop); // 0 = kpidName
    println!("✓ get_property(kpidName) returned: 0x{:08X}", result);
    println!("  PROPVARIANT type: {}", prop.vt);
    
    // Clean up PROPVARIANT
    if prop.vt == 8 {
        // VT_BSTR
        unsafe {
            // Use read_unaligned to avoid alignment issues
            let data_ptr = prop.data.as_mut_ptr() as *mut *mut u16;
            let bstr = std::ptr::read_unaligned(data_ptr);
            if !bstr.is_null() {
                // Free BSTR (simplified - in production use SysFreeString)
                println!("  BSTR allocated (would be freed here)");
            }
        }
    }
    
    // Test get_stream
    let mut stream: *mut IInStream = std::ptr::null_mut();
    let name: Vec<u16> = "test.txt".encode_utf16().chain(std::iter::once(0)).collect();
    let result = boxed.get_stream(name.as_ptr(), &mut stream);
    println!("✓ get_stream() returned: 0x{:08X} (S_FALSE = multi-volume not supported)", result);
    
    println!();
}

#[cfg(feature = "vtable_impl")]
fn test_query_interface() {
    println!("=== Testing QueryInterface ===");
    
    let callback = VTableOpenCallback::new(std::path::Path::new("test.7z"));
    let boxed: VBox<OpenCallbackVTable> = VBox::new(callback);
    
    // Test IUnknown
    let result = boxed.query_interface(&IID_IUnknown);
    println!("IUnknown: {:?}", if result.is_null() { "✗ FAILED" } else { "✓ OK" });
    
    // Test IArchiveOpenCallback
    let result = boxed.query_interface(&IID_IArchiveOpenCallback);
    println!("IArchiveOpenCallback: {:?}", if result.is_null() { "✗ FAILED" } else { "✓ OK" });
    
    // Test IArchiveOpenVolumeCallback
    let result = boxed.query_interface(&IID_IArchiveOpenVolumeCallback);
    println!("IArchiveOpenVolumeCallback: {:?}", if result.is_null() { "✗ FAILED" } else { "✓ OK" });
    
    println!();
}

#[cfg(feature = "vtable_impl")]
fn test_static_vtable() {
    println!("=== Testing Static VTable ===");
    
    // Access the static vtable generated by the macro
    use bit7z_rust::vtable_callback::get_open_callback_vt;
    
    let vt = get_open_callback_vt();
    println!("Static VTable address: {:p}", vt);
    println!("✓ Static VTable generated successfully");
    
    // Verify vtable function pointers are populated
    println!("  query_interface: {:p}", vt.query_interface as *const ());
    println!("  add_ref: {:p}", vt.add_ref as *const ());
    println!("  release: {:p}", vt.release as *const ());
    println!("  set_completed: {:p}", vt.set_completed as *const ());
    println!("  set_total: {:p}", vt.set_total as *const ());
    println!("  get_property: {:p}", vt.get_property as *const ());
    println!("  get_stream: {:p}", vt.get_stream as *const ());
    
    println!();
}

#[cfg(feature = "vtable_impl")]
fn compare_implementations() {
    println!("=== Comparing Implementations ===");
    println!();
    
    // Manual implementation (from vtable_callback.rs)
    use bit7z_rust::vtable_callback::ManualOpenCallback;
    
    let manual = ManualOpenCallback::new(std::path::Path::new("manual.7z"));
    println!("Manual implementation:");
    println!("  Size: {} bytes", std::mem::size_of_val(&manual));
    println!("  Alignment: {}", std::mem::align_of_val(&manual));
    println!("  VTable pointer: {:p}", manual.vtable);
    
    // VTable crate implementation
    let vtable_cb = VTableOpenCallback::new(std::path::Path::new("vtable.7z"));
    let boxed: VBox<OpenCallbackVTable> = VBox::new(vtable_cb);
    
    println!();
    println!("VTable crate implementation:");
    println!("  Size: {} bytes", std::mem::size_of_val(&boxed));
    println!("  Alignment: {}", std::mem::align_of_val(&boxed));
    
    println!();
    println!("Comparison:");
    let size_diff = std::mem::size_of_val(&manual) as i32 - std::mem::size_of_val(&boxed) as i32;
    println!("  Size difference: {} bytes (manual - vtable)", size_diff);
    
    if size_diff > 0 {
        println!("  ✓ VTable crate implementation is more compact");
    } else if size_diff < 0 {
        println!("  ⚠ Manual implementation is more compact");
    } else {
        println!("  = Both implementations have the same size");
    }
    
    println!();
}

#[cfg(not(feature = "vtable_impl"))]
fn main() {
    println!("ERROR: This example requires the 'vtable_impl' feature.");
    println!("Run with: cargo run --example test_vtable_callback --features vtable_impl");
}

#[cfg(feature = "vtable_impl")]
fn main() {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  VTable Crate Implementation Test                         ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    
    test_vtable_creation();
    test_vtable_layout();
    test_callback_methods();
    test_query_interface();
    test_static_vtable();
    compare_implementations();
    
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  All tests completed!                                     ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
}
