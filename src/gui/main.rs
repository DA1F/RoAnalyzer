use std::path::PathBuf;

use qmetaobject::QString;
use qmetaobject::*;
use ro_grpc::fs::FileSystem;

#[derive(QObject)]
struct AndroidFileExplorer {
    base: qt_base_class!(trait QObject),
    fs: FileSystem,

    pub json_data: qt_property!(QString; NOTIFY json_data_changed),
    // Properties exposed to QML
    pub current_path: qt_property!(QString; NOTIFY path_changed),
    pub path_changed: qt_signal!(),
    pub json_data_changed: qt_signal!(),
    pub refresh: qt_method!(fn(&mut self)),
    pub print_lol: qt_method!(fn(&self, json_data: QString)),
}

impl Default for AndroidFileExplorer {
    fn default() -> Self {
        Self {
            fs: FileSystem::new(None),
            base: Default::default(),
            current_path: QString::from("/data/"),
            path_changed: Default::default(),
            json_data: QString::from("[{\"name\": \"lol\", \"rows\": [{\"name\": \"xd\",\"rows\":[{\"name\": \"child1\"}]},{\"name\": \"aaa\"}]}]"),
            json_data_changed: Default::default(),
            refresh: Default::default(),
            print_lol: Default::default(),
        }
    }
}

impl AndroidFileExplorer {
    pub fn print_lol(&self, json_data: QString) {
        println!("print_lol: {:?}", json_data.to_string());
    }

    pub fn refresh(&mut self) {
        self.fs.refresh().unwrap();
        let json_data = self.fs.subtree_json(PathBuf::from("/").as_path());
        //println!("JSON Data: {}", json_data.to_string());
        self.json_data = QString::from(json_data.to_string());
        self.json_data_changed();
        // Build a QJsonArray that QML TreeModel accepts as "array"
        // Build a QJsonArray that QML TreeModel accepts as "array"
    }
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if size >= GB {
        format!("{:.1} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

fn main() {
    qml_register_type::<AndroidFileExplorer>(
        cstr::cstr!("AndroidFileExplorer"),
        1,
        0,
        cstr::cstr!("AndroidFileExplorer"),
    );

    let mut engine = QmlEngine::new();

    // Load QML from file
    let qml_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/gui/qml/main.qml");

    if qml_path.exists() {
        println!("Loading QML from: {:?}", qml_path);
        engine.load_file(qml_path.to_string_lossy().to_string().into());
    } else {
        eprintln!("QML file not found at: {:?}", qml_path);
    }

    engine.exec();
}
