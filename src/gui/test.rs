use qmetaobject::prelude::*;
use std::path::PathBuf;

// Helper function to get file icon - accessible from QML via JavaScript
pub fn get_file_icon(filename: &str) -> &'static str {
    let name = filename.to_lowercase();

    // Get file extension
    if let Some(ext_start) = name.rfind('.') {
        let ext = &name[ext_start + 1..];
        match ext {
            // Images
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" => "🖼️",
            // Videos
            "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" => "🎬",
            // Audio
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" => "🎵",
            // Documents
            "pdf" => "📕",
            "doc" | "docx" => "📘",
            "xls" | "xlsx" => "📗",
            "ppt" | "pptx" => "📙",
            "txt" | "md" => "📄",
            // Archives
            "zip" | "rar" | "7z" | "tar" | "gz" => "📦",
            // Code
            "rs" | "py" | "js" | "ts" | "java" | "cpp" | "c" | "h" => "📝",
            "html" | "css" | "xml" | "json" | "yaml" | "yml" => "📋",
            // Android specific
            "apk" => "📱",
            "dex" => "⚙️",
            "so" => "🔧",
            // Default
            _ => "📄",
        }
    } else {
        "📄"
    }
}

#[derive(QObject, Default)]
struct DemoFileExplorer {
    base: qt_base_class!(trait QObject),
    file_list: qt_property!(QString; NOTIFY files_changed),
    files_changed: qt_signal!(),
}

impl DemoFileExplorer {}

fn main() {
    // Note: On macOS, the menu bar appears in the system menu bar at the top of the screen
    // This is the native macOS behavior and is correct

    qml_register_type::<DemoFileExplorer>(
        cstr::cstr!("DemoFileExplorer"),
        1,
        0,
        cstr::cstr!("DemoFileExplorer"),
    );

    let mut engine = QmlEngine::new();

    // Load QML from file
    let qml_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/gui/main.qml");

    if qml_path.exists() {
        println!("Loading QML from: {:?}", qml_path);
        engine.load_file(qml_path.to_string_lossy().to_string().into());
    } else {
        eprintln!("QML file not found at: {:?}", qml_path);
    }

    engine.exec();
}
