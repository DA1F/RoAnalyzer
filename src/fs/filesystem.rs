use crate::fs::AdbHelper;
use crate::fs::FileInfo;
use crate::fs::FileType;

use serde::Serialize;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

///---------------------------------------------------------------------------
/// In-memory tree node representing a file or directory.
///---------------------------------------------------------------------------
/// Example:
/// ```
/// let mut fs = FileSystem::new();
/// fs.add_entry("/data/local/tmp/file.txt", FileType::File, Some(Metadata { size: 1024, permissions: "rw-r--r--".into(), created_time: "2024-01-01T12:00:00Z".into(), accessed_time: "2024-01-02T12:00:00Z".into(), modified_time: "2024-01-03T12:00:00Z".into() }));
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct FSNode {
    #[serde(skip)]
    metadata: FileInfo,
    #[serde(skip)]
    file_type: FileType,
    #[serde(rename = "rows")]
    pub children: HashMap<OsString, FSNode>, //TODO private
}

impl FSNode {
    pub fn new(metadata: FileInfo) -> Self {
        Self {
            metadata,
            file_type: FileType::Directory,
            children: HashMap::new(),
        }
    }

    pub fn add_child(&mut self, path: &Path, file_type: FileType, metadata: FileInfo) -> usize {
        let mut current = self;
        let mut count = 0;
        for part in path.iter() {
            if !current.children.contains_key(part) {
                current
                    .children
                    .insert(part.to_os_string(), FSNode::new(FileInfo::default()));
                count += 1;
            }
            current = current.children.get_mut(part).unwrap();
        }
        current.file_type = file_type;
        current.metadata = metadata;
        count
    }
    pub fn get_child_mut(&mut self, path: &Path) -> Option<&mut FSNode> {
        //TODO private
        let mut current = self;
        for part in path.iter() {
            if current.children.contains_key(part) {
                current = current.children.get_mut(part).unwrap();
            } else {
                return None;
            }
        }
        Some(current)
    }

    pub fn list_children(&mut self, path: &Path) -> Vec<(OsString, FileType, FileInfo)> {
        // Return chilren names and grandchildren ... in formane /name/child/grandchild/...
        let mut result = Vec::new();
        let current = self.get_child_mut(path);
        if current.is_none() {
            return result;
        }
        let current = current.unwrap();
        current.children.iter().for_each(|(name, child)| {
            result.push((
                name.clone(),
                child.file_type.clone(),
                child.metadata.clone(),
            ));
        });
        result
    }

    pub fn list_folders_tree(&mut self, path: &Path) -> Vec<(PathBuf, FileType, usize)> {
        let mut result: Vec<(PathBuf, FileType, usize)> = Vec::new();
        let current = self.get_child_mut(Path::new(path));
        if current.is_none() {
            return result;
        }
        let current = current.unwrap();
        let mut notes_to_list: VecDeque<(PathBuf, Box<&FSNode>)> = VecDeque::new();
        notes_to_list.push_back((PathBuf::from(path), Box::new(current)));

        while notes_to_list.len() > 0 {
            let item = notes_to_list.pop_front().unwrap();
            let current = item.1;
            let _current_path = item.0;
            current.children.iter().for_each(|(name, child)| {
                result.push((
                    _current_path.join(name),
                    child.file_type.clone(),
                    child.children.len() as usize,
                ));
                if child.file_type == FileType::Directory {
                    notes_to_list
                        .push_back((PathBuf::from(&_current_path).join(name), Box::new(child)));
                }
            });
        }
        result
    }
}

pub struct FileSystem {
    pub root: FSNode, //TODO private
    adb: AdbHelper,
    pub count: usize,
}
impl FileSystem {
    pub fn new(device_serial: Option<String>) -> Self {
        let adb = AdbHelper::new(device_serial).with_root();
        let test = adb.exec_shell("whoami").ok();
        println!("ADB Exec whoami: {:?}", test);
        Self {
            root: FSNode::new(FileInfo::default()),
            adb,
            count: 0,
        }
    }

    pub fn refresh(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.root = FSNode::new(FileInfo::default()); // Reset
        for (path, file_info) in self.adb.load_all()? {
            let file_type = file_info.permissions.chars().next().unwrap_or('?');
            self.count +=
                self.root
                    .add_child(Path::new(&path), FileType::from(&file_type), file_info);
        }
        Ok(())
    }

    pub fn list_directory_as_json(&mut self, path: &Path) -> serde_json::Value {
        fn node_to_json(node: &FSNode) -> serde_json::Value {
            if node.file_type == FileType::Directory {
                let mut map = serde_json::Map::new();
                if node.children.is_empty() {
                    return serde_json::Value::Null;
                }
                let mut children_map = serde_json::Map::new();
                for (name, child) in &node.children {
                    children_map.insert(name.to_string_lossy().into(), node_to_json(child));
                }
                map.insert("rows".into(), serde_json::Value::Object(children_map));
                serde_json::Value::Object(map)
            } else {
                return serde_json::Value::Null;
            }
        }

        let target_node = self.root.get_child_mut(path);
        if target_node.is_none() {
            return serde_json::Value::Null;
        }
        node_to_json(target_node.unwrap())
    }

    // NEW: serialize full tree as { name:"/", rows:[...] }
    pub fn to_tree_json(&mut self) -> serde_json::Value {
        self.subtree_json(Path::new(""))
    }

    // NEW: serialize subtree at `path` (relative to root node keys)
    pub fn subtree_json(&mut self, path: &Path) -> serde_json::Value {
        use serde_json::{Map, Value};

        fn node_to_json(name: &str, node: &FSNode) -> Value {
            let mut obj = Map::new();
            obj.insert("name".into(), Value::String(name.to_string()));

            // For files (or empty dirs), rows is empty array.
            // For dirs, rows contains children serialized as {name, rows}.
            let mut rows: Vec<Value> = Vec::with_capacity(node.children.len());
            if node.file_type == FileType::Directory {
                // If you want deterministic output, sort keys here (costly for huge dirs).
                for (child_name, child_node) in node.children.iter() {
                    if child_node.file_type == FileType::Directory {
                        let child_name = child_name.to_string_lossy();
                        rows.push(node_to_json(&child_name, child_node));
                    }
                }
            }
            obj.insert("rows".into(), Value::Array(rows));
            Value::Object(obj)
        }

        // Resolve target node
        let target = match self.root.get_child_mut(path) {
            Some(n) => n,
            None => return serde_json::Value::Null,
        };

        // Derive displayed name for subtree root
        let display_name = if path.as_os_str().is_empty() {
            "[ROOT]"
        } else {
            path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("[ROOT]")
        };

        node_to_json(display_name, target)
    }

    pub fn subtree_as_json(&mut self, path: &Path) -> serde_json::Value {
        use serde_json::{json, Value};

        fn node_to_json(name: &str, full_path: &str, node: &FSNode) -> Value {
            let mut rows: Vec<Value> = Vec::new();

            // Recursively include all subdirectories
            if node.file_type == FileType::Directory {
                for (child_name, child_node) in node.children.iter() {
                    if child_node.file_type == FileType::Directory {
                        let child_name_str = child_name.to_string_lossy();
                        let child_full_path = if full_path == "/" {
                            format!("/{}", child_name_str)
                        } else {
                            format!("{}/{}", full_path, child_name_str)
                        };

                        // Recursive call to get all nested subfolders
                        rows.push(node_to_json(&child_name_str, &child_full_path, child_node));
                    }
                }
            }

            json!({
                "name": name.to_string(),
                "path": full_path.to_string(),
                "rows": rows
            })
        }

        // Resolve target node
        let target = match self.root.get_child_mut(path) {
            Some(n) => n,
            None => return Value::Array(vec![]),
        };

        let mut result: Vec<Value> = Vec::new();

        // Return only the children (not wrapped in parent)
        if target.file_type == FileType::Directory {
            for (child_name, child_node) in target.children.iter() {
                if child_node.file_type == FileType::Directory {
                    let child_name_str = child_name.to_string_lossy();

                    let child_full_path =
                        if path.as_os_str().is_empty() || path.to_str() == Some("/") {
                            format!("/{}", child_name_str)
                        } else {
                            format!("{}/{}", path.to_string_lossy(), child_name_str)
                        };

                    // Recursive call includes all nested subfolders
                    result.push(node_to_json(&child_name_str, &child_full_path, child_node));
                }
            }
        }
        Value::Array(result)
    }
}
