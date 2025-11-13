use std::env;
use std::path::PathBuf;

fn main() {
    // Tauri build script
    tauri_build::build();

    // Early return if kws_real feature is not enabled
    let real = env::var("CARGO_FEATURE_KWS_REAL").is_ok();
    if !real {
        println!("cargo:warning=Building without Sherpa-ONNX (kws_real feature not enabled)");
        println!(
            "cargo:warning=Use 'npm run dev:real' or '--features kws_real' to enable real KWS"
        );
        return;
    }

    // Check for Sherpa-ONNX directory
    let sherpa_dir = match env::var("SHERPA_ONNX_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => {
            eprintln!("\n==========================================================");
            eprintln!("WARNING: SHERPA_ONNX_DIR environment variable not set!");
            eprintln!("==========================================================");
            eprintln!();
            eprintln!("Sherpa-ONNX FFI bindings will not be generated.");
            eprintln!();
            eprintln!("To build with Sherpa-ONNX support:");
            eprintln!();
            eprintln!("1. Build or install Sherpa-ONNX:");
            eprintln!("   git clone https://github.com/k2-fsa/sherpa-onnx");
            eprintln!("   cd sherpa-onnx");
            eprintln!("   mkdir build && cd build");
            eprintln!("   cmake -DCMAKE_BUILD_TYPE=Release ..");
            eprintln!("   make -j4");
            eprintln!();
            eprintln!("2. Set SHERPA_ONNX_DIR to your build or install directory:");
            eprintln!("   export SHERPA_ONNX_DIR=/path/to/sherpa-onnx/build");
            eprintln!();
            eprintln!("3. Rebuild:");
            eprintln!("   cargo build");
            eprintln!();
            eprintln!("==========================================================\n");

            // In dev mode, allow building without Sherpa-ONNX
            // The placeholder implementation will be used
            if cfg!(debug_assertions) {
                eprintln!("Debug build: continuing without Sherpa-ONNX (using placeholder)");
                return;
            } else {
                panic!("SHERPA_ONNX_DIR must be set for release builds");
            }
        }
    };

    // Verify Sherpa-ONNX directory exists
    if !sherpa_dir.exists() {
        panic!(
            "SHERPA_ONNX_DIR points to non-existent directory: {}",
            sherpa_dir.display()
        );
    }

    println!(
        "cargo:warning=Using Sherpa-ONNX from: {}",
        sherpa_dir.display()
    );

    // Look for headers in common locations
    let header_paths = [
        sherpa_dir.join("include/sherpa-onnx/c-api/c-api.h"), // Standard install prefix
        sherpa_dir.join("sherpa-onnx/csrc/sherpa-onnx-c-api.h"),
        sherpa_dir.join("install/include/sherpa-onnx/c-api/c-api.h"),
        sherpa_dir.join("../sherpa-onnx/csrc/sherpa-onnx-c-api.h"),
        PathBuf::from("/usr/local/include/sherpa-onnx/c-api/c-api.h"),
        PathBuf::from("/usr/include/sherpa-onnx/c-api/c-api.h"),
    ];

    let header_path = header_paths
        .iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Could not find sherpa-onnx-c-api.h in any of:");
            for p in &header_paths {
                eprintln!("  - {}", p.display());
            }
            panic!("Sherpa-ONNX C API header not found");
        });

    println!("cargo:warning=Found header: {}", header_path.display());

    // Look for libraries in common locations
    let lib_paths = [
        sherpa_dir.join("lib"),
        sherpa_dir.join("install/lib"),
        sherpa_dir.clone(),
        PathBuf::from("/usr/local/lib"),
        PathBuf::from("/usr/lib"),
    ];

    let lib_path = lib_paths
        .iter()
        .find(|p| {
            p.join("libsherpa-onnx-c-api.so").exists()
                || p.join("libsherpa-onnx-c-api.dylib").exists()
                || p.join("libsherpa-onnx-c-api.dll").exists()
                || p.join("sherpa-onnx-c-api.lib").exists()
        })
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Could not find libsherpa-onnx-c-api library in any of:");
            for p in &lib_paths {
                eprintln!("  - {}", p.display());
            }
            panic!("Sherpa-ONNX library not found");
        });

    println!("cargo:warning=Found library in: {}", lib_path.display());

    // Enable the sherpa_onnx_ffi cfg
    println!("cargo:rustc-cfg=sherpa_onnx_ffi");

    // Tell cargo to link the library
    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-lib=dylib=sherpa-onnx-c-api");

    // Also link onnxruntime if needed
    println!("cargo:rustc-link-lib=dylib=onnxruntime");

    // Set rpath for runtime linking (Unix-like systems)
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path.display());

    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path.display());

    // Generate Rust bindings
    println!("cargo:rerun-if-changed={}", header_path.display());

    let out_path = PathBuf::from("src").join("ffi");
    std::fs::create_dir_all(&out_path).expect("Failed to create ffi directory");

    let bindings_path = out_path.join("sherpa_onnx_bindings.rs");

    // Only regenerate if bindings don't exist or header is newer
    let should_regenerate = if !bindings_path.exists() {
        true
    } else {
        use std::time::SystemTime;
        let header_time = std::fs::metadata(&header_path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let bindings_time = std::fs::metadata(&bindings_path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        header_time > bindings_time
    };

    if should_regenerate {
        println!("cargo:warning=Generating Sherpa-ONNX bindings...");

        let bindings = bindgen::Builder::default()
            .header(header_path.to_str().unwrap())
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            // Only generate bindings for sherpa-onnx types (v1.10.30 API)
            .allowlist_type("SherpaOnnx.*")
            .allowlist_function("SherpaOnnx.*")
            .allowlist_var("SHERPA_ONNX_.*")
            // Generate simpler C types
            .derive_default(true)
            .derive_debug(true)
            .derive_copy(false) // Some structs contain pointers
            .derive_eq(false)
            // Use core types
            .use_core()
            .ctypes_prefix("::std::os::raw")
            // Finish
            .generate()
            .expect("Unable to generate Sherpa-ONNX bindings");

        bindings
            .write_to_file(&bindings_path)
            .expect("Couldn't write bindings!");

        println!(
            "cargo:warning=Bindings written to: {}",
            bindings_path.display()
        );
    } else {
        println!("cargo:warning=Using existing Sherpa-ONNX bindings (header unchanged)");
    }
}
