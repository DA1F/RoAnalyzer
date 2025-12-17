fn main() {
    // Use vendored protoc and compile proto files to Rust at build time
    println!("cargo:rerun-if-changed=proto/emulator_controller.proto");

    tonic_build::configure()
        .build_server(false) // client-only library by default
        .protoc_arg("--experimental_allow_proto3_optional") // for newer protoc compatibility
        .compile(&["proto/emulator_controller.proto"], &["proto"])
        .expect("Failed to compile proto files");

    // On macOS, embed runtime search paths for FFmpeg and Qt frameworks
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        use std::{env, fs, process::Command};

        fn add_rpath(p: &str) {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", p);
        }

        // FFmpeg: allow override via FFMPEG_LIB_DIR
        if let Ok(ff_lib) = env::var("FFMPEG_LIB_DIR") {
            if fs::metadata(&ff_lib).is_ok() {
                add_rpath(&ff_lib);
            }
        } else {
            // Common Homebrew location
            let brew_ff = "/opt/homebrew/opt/ffmpeg/lib";
            if fs::metadata(brew_ff).is_ok() {
                add_rpath(brew_ff);
            }
        }

        // Qt: prefer explicit env overrides
        let qt_env_vars = [
            "QT_LIB_DIR",
            "QT_FRAMEWORK_DIR",
            "QTDIR",
            "QT_DIR",
            "QT_PREFIX",
        ];
        let mut qt_paths: Vec<String> = Vec::new();
        for var in qt_env_vars {
            if let Ok(v) = env::var(var) {
                qt_paths.push(v);
            }
        }

        // If not provided, try qtpaths/qmake to discover install libs path
        if qt_paths.is_empty() {
            let qtpaths = env::var("QTPATHS").ok().unwrap_or_else(|| "qtpaths".into());
            if let Ok(out) = Command::new(&qtpaths)
                .args(["--query", "QT_INSTALL_LIBS"])
                .output()
            {
                if out.status.success() {
                    if let Ok(s) = String::from_utf8(out.stdout) {
                        let p = s.trim();
                        if !p.is_empty() {
                            qt_paths.push(p.to_string());
                        }
                    }
                }
            }
        }
        if qt_paths.is_empty() {
            // qmake (Qt5) fallback
            if let Ok(out) = Command::new("qmake")
                .args(["-query", "QT_INSTALL_LIBS"])
                .output()
            {
                if out.status.success() {
                    if let Ok(s) = String::from_utf8(out.stdout) {
                        let p = s.trim();
                        if !p.is_empty() {
                            qt_paths.push(p.to_string());
                        }
                    }
                }
            }
        }

        // Last-resort fallbacks: common Homebrew paths
        if qt_paths.is_empty() {
            for &p in &["/opt/homebrew/opt/qt/lib", "/usr/local/opt/qt/lib"] {
                if fs::metadata(p).is_ok() {
                    qt_paths.push(p.to_string());
                }
            }
        }

        // Add all valid qt paths as rpaths
        for p in qt_paths {
            if fs::metadata(&p).is_ok() {
                add_rpath(&p);
            }
        }
    }
}
