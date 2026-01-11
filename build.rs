/// Build script for ClarityPosture
///
/// This script copies required runtime files (ONNX runtime DLL and model) to the target directory
/// so they are available when the application runs.

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=movenet_singlepose_thunder.onnx");

    // Get the target directory
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir).parent().unwrap().parent().unwrap();

    // We'll copy files to both debug and release directories
    let debug_dir = target_dir.join("debug");
    let release_dir = target_dir.join("release");

    // List of files to copy
    let files_to_copy = vec![
        ("movenet_singlepose_thunder.onnx", "MoveNet Thunder model"),
        // Note: ONNX runtime DLL is typically handled by the ort crate,
        // but we'll include a fallback copy if needed
    ];

    // Copy files to debug directory
    if debug_dir.exists() {
        println!("Copying runtime files to: {:?}", debug_dir);
        for (file_name, description) in &files_to_copy {
            let src_path = Path::new(file_name);
            let dest_path = debug_dir.join(file_name);

            if src_path.exists() {
                if let Err(e) = fs::copy(src_path, dest_path) {
                    eprintln!("Failed to copy {} to debug directory: {}", description, e);
                } else {
                    println!("Copied {} to debug directory", description);
                }
            } else {
                println!("Warning: {} not found at {:?}", description, src_path);
            }
        }
    }

    // Copy files to release directory (if it exists)
    if release_dir.exists() {
        println!("Copying runtime files to: {:?}", release_dir);
        for (file_name, description) in &files_to_copy {
            let src_path = Path::new(file_name);
            let dest_path = release_dir.join(file_name);

            if src_path.exists() {
                if let Err(e) = fs::copy(src_path, dest_path) {
                    eprintln!("Failed to copy {} to release directory: {}", description, e);
                } else {
                    println!("Copied {} to release directory", description);
                }
            }
        }
    }

    // Also copy to the out directory for cargo run
    println!("Copying runtime files to: {:?}", out_dir);
    for (file_name, description) in &files_to_copy {
        let src_path = Path::new(file_name);
        let dest_path = Path::new(&out_dir).join(file_name);

        if src_path.exists() {
            if let Err(e) = fs::copy(src_path, dest_path) {
                eprintln!("Failed to copy {} to out directory: {}", description, e);
            } else {
                println!("Copied {} to out directory", description);
            }
        }
    }
}
