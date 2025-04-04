use std::env;
use std::fs::File;
use std::io::Result;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    // Tell Cargo to rerun this build script if proto files or the build script change
    println!("cargo:rerun-if-changed=protos/");
    println!("cargo:rerun-if-changed=build.rs");

    // Get output directory from Cargo
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // List all proto files
    let proto_files = vec!["protos/monitord.proto", "protos/config.proto"];

    // Ensure the proto directory exists
    let proto_dir = Path::new("protos");
    if !proto_dir.exists() {
        panic!("Proto directory not found: {:?}", proto_dir);
    }

    // Verify the existence of all proto files
    for proto_file in &proto_files {
        let file_path = Path::new(proto_file);
        if !file_path.exists() {
            panic!("Proto file not found: {:?}", file_path);
        }
    }

    // Method 1: Use tonic_build directly without prost_build
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(&out_dir)
        // Compile proto files
        .compile_protos(&proto_files, &[proto_dir])?;

    // Generate a mod.rs file that re-exports all generated modules
    generate_mod_file(&out_dir)?;

    Ok(())
}

/// Generates a mod.rs file that exports all the generated protobuf modules
fn generate_mod_file(out_dir: &Path) -> Result<()> {
    let mod_path = out_dir.join("mod.rs");
    let mut file = File::create(&mod_path)?;

    writeln!(file, "// Generated protobuf code for monitord.")?;
    writeln!(file, "// This file is generated by build.rs")?;
    writeln!(file)?;

    // Re-export the generated protobuf modules
    writeln!(file, "// Main protobuf modules")?;
    writeln!(file, "pub mod monitord {{")?;
    writeln!(
        file,
        "    include!(concat!(env!(\"OUT_DIR\"), \"/monitord.rs\"));"
    )?;
    writeln!(file, "}}")?;
    writeln!(file)?;

    writeln!(file, "pub mod config {{")?;
    writeln!(
        file,
        "    include!(concat!(env!(\"OUT_DIR\"), \"/monitord.config.rs\"));"
    )?;
    writeln!(file, "}}")?;
    writeln!(file)?;

    // Create a module structure for easier access to important types
    writeln!(file, "// Re-export commonly used types for convenience")?;
    writeln!(file, "pub mod protocols {{")?;

    // System types
    writeln!(file, "    // System information types")?;
    writeln!(file, "    pub use super::monitord::SystemSnapshot;")?;
    writeln!(file, "    pub use super::monitord::SystemInfo;")?;
    writeln!(file, "    pub use super::monitord::CpuInfo;")?;
    writeln!(file, "    pub use super::monitord::MemoryInfo;")?;
    writeln!(file, "    pub use super::monitord::GpuInfo;")?;
    writeln!(file, "    pub use super::monitord::NetworkInfo;")?;
    writeln!(file, "    pub use super::monitord::ProcessInfo;")?;
    writeln!(file, "    pub use super::monitord::StorageInfo;")?;
    writeln!(file)?;

    // Service clients and servers
    writeln!(file, "    // Service clients and servers")?;
    writeln!(
        file,
        "    pub use super::monitord::monitord_service_client::MonitordServiceClient;"
    )?;
    writeln!(
        file,
        "    pub use super::monitord::monitord_service_server::MonitordServiceServer;"
    )?;
    writeln!(
        file,
        "    pub use super::config::config_service_client::ConfigServiceClient;"
    )?;
    writeln!(
        file,
        "    pub use super::config::config_service_server::ConfigServiceServer;"
    )?;
    writeln!(file)?;

    // Add prost_types re-export for timestamp conversion
    writeln!(file, "    // Re-export prost_types for timestamp handling")?;
    writeln!(file, "    pub use prost_types;")?;
    writeln!(file, "}}")?;

    Ok(())
}
