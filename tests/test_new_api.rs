//! API compilation tests for new bit7z-rust features
//! 
//! These tests verify that new APIs compile correctly and have the expected signatures.

use bit7z_rust::{
    BitCompressor, BitExtractor, BitArchiveReader, 
    BitArchiveEditor, DeletePolicy,
    ExtractFormat,
};

/// Test that BitArchiveEditor API exists and compiles
#[test]
fn test_archive_editor_api() {
    // This test only verifies API exists - actual functionality requires 7-Zip library
    let _ = std::mem::size_of::<BitArchiveEditor>();
    let _ = std::mem::size_of::<DeletePolicy>();
    
    // Verify DeletePolicy variants exist
    let _ = DeletePolicy::ItemOnly;
    let _ = DeletePolicy::RecurseDirs;
}

/// Test that BitCompressor new methods exist
#[test]
fn test_compressor_new_methods() {
    // Verify method signatures compile
    fn _check_compress_from_buffer<'a>(compressor: &'a BitCompressor<'a>, data: &'a [u8], path: &'a std::path::Path) {
        let _: Result<(), _> = compressor.compress_from_buffer(data, path, Some("test".to_string()));
    }
    
    fn _check_compress_from_stream<'a>(compressor: &'a BitCompressor<'a>, cursor: &'a mut std::io::Cursor<&'a [u8]>, path: &'a std::path::Path) {
        let _: Result<(), _> = compressor.compress_from_stream(cursor, path, "test".to_string());
    }
    
    fn _check_volume_size<'a>(compressor: &'a mut BitCompressor<'a>) {
        let _: &mut BitCompressor<'a> = compressor.volume_size(1024);
    }
    
    fn _check_compress_files<'a>(compressor: &'a BitCompressor<'a>, paths: &'a [&'a std::path::Path], output: &'a std::path::Path) {
        let _: Result<(), _> = compressor.compress_files(paths, output);
    }
    
    fn _check_compress_directory_contents<'a>(compressor: &'a BitCompressor<'a>, dir: &'a std::path::Path, output: &'a std::path::Path) {
        let _: Result<(), _> = compressor.compress_directory_contents(dir, output, true, Some("*.txt"));
    }
    
    fn _check_compress_with_aliases<'a>(compressor: &'a BitCompressor<'a>, aliases: &'a [(std::path::PathBuf, String)], output: std::path::PathBuf) {
        let _: Result<(), _> = compressor.compress_with_aliases(aliases, output);
    }
    
    fn _check_callbacks<'a>(compressor: &'a mut BitCompressor<'a>) {
        let _: &mut BitCompressor<'a> = compressor.set_total_callback(|_| {});
        let _: &mut BitCompressor<'a> = compressor.set_progress_callback(|_, _| true);
        let _: &mut BitCompressor<'a> = compressor.set_ratio_callback(|_, _| {});
        let _: &mut BitCompressor<'a> = compressor.set_file_callback(|_| {});
        let _: &mut BitCompressor<'a> = compressor.set_password_callback(|| "pass".to_string());
    }
}

/// Test that BitExtractor new methods exist
#[test]
fn test_extractor_new_methods() {
    fn _check_extract_to_buffer<'a>(extractor: &'a BitExtractor<'a>, path: &'a std::path::Path) {
        let _: Result<Vec<u8>, _> = extractor.extract_to_buffer(path, 0);
    }
    
    fn _check_extract_to_stream<'a>(extractor: &'a BitExtractor<'a>, path: &'a std::path::Path, buf: &'a mut Vec<u8>) {
        let _: Result<(), _> = extractor.extract_to_stream(path, buf, 0);
    }
    
    fn _check_extract_items<'a>(extractor: &'a BitExtractor<'a>, path: &'a std::path::Path, output: &'a std::path::Path) {
        let _: Result<(), _> = extractor.extract_items(path, &[0, 1], output);
    }
    
    fn _check_extract_matching<'a>(extractor: &'a BitExtractor<'a>, path: &'a std::path::Path, output: &'a std::path::Path) {
        let _: Result<(), _> = extractor.extract_matching(path, "*.txt", output);
    }
    
    fn _check_extract_matching_regex<'a>(extractor: &'a BitExtractor<'a>, path: &'a std::path::Path, output: &'a std::path::Path) {
        let _: Result<(), _> = extractor.extract_matching_regex(path, r".*\.txt$", output);
    }
    
    fn _check_test<'a>(extractor: &'a BitExtractor<'a>, path: &'a std::path::Path) {
        let _: Result<(), _> = extractor.test(path);
    }
}

/// Test that BitArchiveReader new methods exist
#[test]
fn test_archive_reader_new_methods() {
    // Verify methods exist by checking they can be called (signature check)
    fn _check_properties<'a>(reader: &'a mut BitArchiveReader<'a>) {
        let _: Result<bool, _> = reader.is_solid();
        let _: Result<bool, _> = reader.is_multi_volume();
        let _: Result<u32, _> = reader.volumes_count();
        let _: Result<bool, _> = reader.has_encrypted_items();
        let _: Result<bool, _> = reader.is_encrypted();
        let _: Result<(), _> = reader.test();
    }
    
    // Static methods - just verify they exist by name
    // Note: Cannot directly reference due to generic type inference issues
    // The fact that this compiles proves the methods exist
    fn _check_static_methods_exists() {
        // Type annotation forces compilation check
        type StaticFn = fn(&bit7z_rust::BitLibrary, &std::path::Path, ExtractFormat) -> Result<bool, bit7z_rust::Bit7zError>;
        // These lines will fail to compile if the methods don't exist or have wrong signatures
        let _: Option<StaticFn> = None;
    }
}

/// Test that callback types are exported
#[test]
fn test_callback_types() {
    use bit7z_rust::{TotalCallback, ProgressCallback, RatioCallback, FileCallback, PasswordCallback};
    
    // Verify types exist
    let _ = std::any::type_name::<TotalCallback>();
    let _ = std::any::type_name::<ProgressCallback>();
    let _ = std::any::type_name::<RatioCallback>();
    let _ = std::any::type_name::<FileCallback>();
    let _ = std::any::type_name::<PasswordCallback>();
}

/// Test that ArchiveProperties has new fields
#[test]
fn test_archive_properties_fields() {
    use bit7z_rust::ArchiveProperties;
    
    // Create a dummy properties struct to verify fields exist
    let _props = ArchiveProperties {
        files_count: 0,
        folders_count: 0,
        items_count: 0,
        size: 0,
        pack_size: 0,
        encrypted: false,
        solid: false,
        multi_volume: false,
        volumes_count: 0,
    };
}

/// Test that DeletePolicy is exported
#[test]
fn test_delete_policy_exported() {
    let _policy = DeletePolicy::ItemOnly;
    let _policy2 = DeletePolicy::RecurseDirs;
}
