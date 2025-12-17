# FileSystem Structure Usage Guide

The `FileSystem` struct provides an in-memory tree representation of the filesystem with support for:

- Building from `FileInfo` entries (with full metadata)
- Building from `ls -lR` output (names/structure only, fast)
- Building from `find / -type d` output (directories only, fastest)
- Serializing to JSON for QML TreeView population
- Lazy loading and on-demand directory expansion

## Quick Start

### Building from ADB FileInfo entries

```rust
use ro_grpc::fs::{AdbFileSystem, FileSystem};

let adb = AdbFileSystem::new(None);
let entries = adb.list_files_detailed("/")?;

// Build tree structure
let fs = FileSystem::from_entries("/", entries);

// Access root directory
let root = fs.root();
println!("Subdirs: {}, Files: {}", 
    root.directories().len(), 
    root.files().len());

// Serialize to JSON for QML
let json = fs.to_json();
```

Output JSON format:

```json
{
  "type": "📁",
  "name": "/",
  "size": "",
  "lastModified": "1765287390",
  "rows": [
    {
      "type": "📁",
      "name": "data",
      "size": "",
      "lastModified": "1765287390",
      "rows": [
        {
          "type": "📄",
          "name": "config.txt",
          "size": "1.0KB",
          "lastModified": "1765287390"
        }
      ]
    }
  ]
}
```

### Building from `ls -lR` output (Fast, Structure Only)

When you only need folder structure and file names (no metadata), use `ls -lR`:

```rust
use ro_grpc::fs::{AdbFileSystem, FileSystem};

let adb = AdbFileSystem::new(None);

// Much faster: only gets names/structure
let output = adb.exec_shell("ls -lR /")?;
let fs = FileSystem::from_ls_lr("/", &output);

// Same API as before
let json = fs.to_json();
```

**Why use `ls -lR`?**

- ✅ Much faster: single command instead of per-file `stat` calls
- ✅ Lower overhead: fewer network round-trips to device
- ✅ Still builds complete tree structure with nesting
- ❌ No file sizes or permission metadata (but that's optional)

### Building from `find / -type d -print` output (Fastest, Directories Only)

When you only need the directory structure (no files), use `find -type d`:

```rust
use ro_grpc::fs::{AdbFileSystem, FileSystem};

let adb = AdbFileSystem::new(None);

// Fastest: only gets directories, minimal parsing
let output = adb.exec_shell("find / -type d -print")?;
let fs = FileSystem::from_find_type_d("/", &output);

// Same API as before
let json = fs.to_json();
```

**Why use `find -type d`?**

- ✅ Fastest: minimal output, simple parsing
- ✅ Lowest overhead: single command, very fast network transfer
- ✅ Perfect for exploring deep directory structures
- ✅ Includes nested directory relationships
- ❌ No files listed
- ❌ No metadata (sizes, permissions)

### Navigating the Tree

```rust
let fs = FileSystem::from_entries("/", entries);
let root = fs.root();

// Access subdirectories
for dir in root.directories() {
    println!("Directory: {}", dir.name());
    
    // Nested access
    for file in dir.files() {
        println!("  File: {}", 
            file.path.file_name().unwrap().to_string_lossy());
    }
}

// Get directory metadata
if let Some(info) = root.info() {
    println!("Root modified: {:?}", info.modified_time);
}
```

### Custom Tree Building

```rust
use ro_grpc::fs::{FileInfo, FileType, FilePermissions, FileSystem};

let mut fs = FileSystem::new("/");

// Add individual entries
let entry = FileInfo {
    path: PathBuf::from("/data/test.txt"),
    file_type: FileType::File,
    size: 1024,
    permissions: FilePermissions {
        mode: 0o644,
        owner: "user".to_string(),
        group: "user".to_string(),
    },
    modified_time: SystemTime::now(),
    accessed_time: None,
    created_time: None,
};

fs.add_entry(entry);
let json = fs.to_json();
```

## API Reference

### FileSystem

**Construction:**

- `FileSystem::new(root_path)` - Create empty tree
- `FileSystem::from_entries(root_path, entries)` - Build from FileInfo iterator
- `FileSystem::from_ls_lr(root_path, output)` - Build from `ls -lR` output
- `FileSystem::from_find_type_d(root_path, output)` - Build from `find -type d` output

**Methods:**

- `root()` -> `&DirectoryNode` - Access root directory
- `root_mut()` -> `&mut DirectoryNode` - Mutable access to root
- `add_entry(entry)` - Insert single FileInfo entry
- `to_json()` -> `String` - Serialize tree to JSON

### DirectoryNode

**Accessors:**

- `name()` -> `&str` - Directory name
- `path()` -> `&PathBuf` - Full path
- `info()` -> `Option<&FileInfo>` - Metadata (if available)
- `directories()` -> `&[DirectoryNode]` - Subdirectories
- `files()` -> `&[FileInfo]` - Files in this directory

**Mutators:**

- `directories_mut()` -> `&mut [DirectoryNode]`
- `files_mut()` -> `&mut [FileInfo]`

**Serialization:**

- `to_json_value()` -> `serde_json::Value` - Convert to JSON

## Performance Notes

| Method | Speed | Overhead | Use When |
|--------|-------|----------|----------|
| `list_files_detailed()` + `from_entries` | Slow (~N network calls) | Per-file stat | Need full metadata (size, permissions, times) |
| `ls -lR` + `from_ls_lr` | Fast (1 command) | Minimal | Need structure + filenames + sizes |
| `find -type d` + `from_find_type_d` | Fastest (1 command) | Minimal | Only need directory structure |

For `/data` with 100 items:

- `list_files_detailed()`: ~1-2 seconds (100+ adb calls)
- `ls -lR` + parse: ~100-200ms (1 adb call)
- `find -type d` + parse: ~50-100ms (1 adb call, simplest output)

## QML Integration Example

```qml
// RoFSView.qml
import QtQuick
import Qt.labs.qmlmodels

TreeView {
    model: TreeModel {
        id: treeModel
        TableModelColumn { display: "type" }
        TableModelColumn { display: "name" }
        rows: JSON.parse(explorer.tree_json)  // <- Use to_json() output here
    }
    
    delegate: TreeViewDelegate {}
}
```

In Rust:

```rust
fn refresh(&mut self) {
    // Fast path: use ls -lR
    let output = self.adb.borrow_mut().exec_shell("ls -lR /data")?;
    let fs = FileSystem::from_ls_lr("/data", &output);
    self.tree_json = fs.to_json().into();
}
```
