//build.rs
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::io;
fn main() {
    // Get the current working directory
    let current_dir = env::current_dir().expect("Failed to get current directory");

    // Paths for proto files and include directories
    let proto_dir = current_dir.join("protocols");
    let include_path = proto_dir.clone();

    // Collect all `.proto` files in the protocols directory (recursively if needed)
    let proto_files: Vec<PathBuf> = find_proto_files(&proto_dir)
        .expect("Failed to find proto files")
        .collect();

    // Compile all found .proto files
    prost_build::compile_protos(
        &proto_files.iter().map(|p| p.to_str().unwrap()).collect::<Vec<_>>(),
        &[include_path.to_str().unwrap()],
    )
    .expect("Failed to compile .proto files");

    // Rename generated files for each .proto file
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    for proto_file in proto_files {
        let file_name = proto_file.file_stem().unwrap().to_str().unwrap();
        let generated_file = Path::new(&out_dir).join("_.rs");
        let renamed_file = Path::new(&out_dir).join(format!("{}.rs", file_name));
        if generated_file.exists() {
            fs::rename(&generated_file, &renamed_file)
                .unwrap_or_else(|e| panic!("Failed to rename {}: {}", generated_file.display(), e));
        }
    }

    // Ensure rebuild when .proto files change
    println!("cargo:rerun-if-changed={}", proto_dir.display());
}

/// Recursively find all `.proto` files in a directory
fn find_proto_files(dir: &Path) -> io::Result<impl Iterator<Item = PathBuf>> {
    Ok(fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().map_or(false, |ext| ext == "proto")))
}
