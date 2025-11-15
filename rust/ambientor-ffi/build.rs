// Build script that tries to generate a C header with `cbindgen`.
// If `cbindgen` is not available, it falls back to copying the
// checked-in `include/ambientor.h` to $OUT_DIR.
//
// Either way, your consumers can include the header from:
//   - <repo>/rust/ambientor-ffi/include/ambientor.h      (checked-in)
//   - $OUT_DIR/ambientor.h (Cargo exposes via env at build time)

use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    // Re-run build.rs if these change
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=include/ambientor.h");

    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let header_path_repo = crate_dir.join("include").join("ambientor.h");
    let header_path_out = out_dir.join("ambientor.h");

    // Try cbindgen first
    let cbindgen_ok = Command::new("cbindgen")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if cbindgen_ok {
        let status = Command::new("cbindgen")
            .arg("--crate")
            .arg("ambientor-ffi")
            .arg("--lang")
            .arg("C")
            .arg("--output")
            .arg(&header_path_out)
            .current_dir(&crate_dir)
            .status()
            .expect("failed to execute cbindgen");

        if status.success() {
            // Also write/update the repo copy so consumers can include it directly.
            if let Some(parent) = header_path_repo.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::copy(&header_path_out, &header_path_repo);
            println!("cargo:warning=ambientor-ffi: generated header with cbindgen -> {}", header_path_out.display());
            return;
        } else {
            println!("cargo:warning=ambientor-ffi: cbindgen failed; falling back to checked-in header");
        }
    } else {
        println!("cargo:warning=ambientor-ffi: cbindgen not found; falling back to checked-in header");
    }

    // Fallback: copy the checked-in header to OUT_DIR so downstream build systems can find it.
    if header_path_repo.exists() {
        fs::copy(&header_path_repo, &header_path_out)
            .expect("failed to copy include/ambientor.h to OUT_DIR");
    } else {
        // Last-ditch: write a tiny placeholder header to OUT_DIR
        let placeholder = b"/* ambientor.h placeholder: please install cbindgen or keep include/ambientor.h checked in */\n";
        fs::write(&header_path_out, placeholder).expect("failed to write placeholder header");
    }
}
