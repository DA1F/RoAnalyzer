use crate::fs::FileInfo;
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};

/// Unix file permissions

// impl FilePermissions {
//     /// Parse permissions from ls -l format (e.g., "rw-r--r--")
//     pub fn from_ls_string(perms: &str, owner: &str, group: &str) -> Self {
//         let mut mode = 0u32;

//         // Skip first char (file type indicator)
//         let perms = if perms.len() > 9 { &perms[1..] } else { perms };

//         for (i, c) in perms.chars().take(9).enumerate() {
//             if c != '-' {
//                 let bit = match i % 3 {
//                     0 => 4, // read
//                     1 => 2, // write
//                     2 => 1, // execute
//                     _ => 0,
//                 };
//                 mode |= bit << (3 * (2 - i / 3));
//             }
//         }

//         Self {
//             mode,
//             owner: owner.to_string(),
//             group: group.to_string(),
//         }
//     }

//     /// Get octal representation (e.g., 0644)
//     pub fn octal(&self) -> String {
//         format!("{:04o}", self.mode)
//     }
// }

/// ADB-based filesystem client for Android emulator
#[derive(Clone)]
pub struct AdbHelper {
    device_serial: Option<String>,
    adb_path: String,
    root: bool,
}

impl AdbHelper {
    /// Create a new ADB filesystem client
    ///
    /// # Arguments
    /// * `device_serial` - Optional device serial (e.g., "emulator-5554"). If None, uses first available device.
    pub fn new(device_serial: Option<String>) -> Self {
        Self {
            device_serial,
            adb_path: "adb".to_string(), // Assumes adb is in PATH
            root: false,
        }
    }

    /// Set whether to use root (su) for shell commands
    pub fn with_root(mut self) -> Self {
        self.root = true;
        self
    }

    /// Set custom ADB executable path
    pub fn with_adb_path(mut self, path: String) -> Self {
        self.adb_path = path;
        self
    }

    pub fn exec_pty(&self, command: &str) -> Result<Vec<String>> {
        // Execute multiple commands in interactive shell with root access
        let mut child = Command::new(&self.adb_path)
            .args(&["shell"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout);

        // Send commands
        writeln!(stdin, "su root")?; // TODO: change the SU command when needed
        writeln!(stdin, "{}", command)?;
        //writeln!(stdin, "find / -path /proc -prune -o -print0 | xargs -0 stat -c \"%i|%A|%Z|%Y|%X|%U|%G|%s|%N\"")?;
        writeln!(stdin, "echo ___DF_LV_RO___")?; //TODO: change to unique random token
        stdin.flush()?;

        let mut output: Vec<String> = Vec::new();
        // Read output
        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            line.pop(); // Remove newline
            if line.starts_with("___DF_LV_RO___") {
                break;
            }
            output.push(line.clone());
            line.clear();
        }

        Ok(output)
    }

    /// Example usage:
    /// ```ignore
    /// let adb = AdbHelper::new(None);
    /// let output = adb.exec_tty(&[
    ///     "whoami",
    ///     "id",
    ///     "ls /data",
    ///     "cat /system/build.prop | head -5",
    /// ])?;
    /// println!("Combined output:\n{}", output);
    /// ```
    /// Execute an ADB shell command and return stdout
    pub fn exec_shell(&self, command: &str) -> Result<String> {
        let mut cmd = Command::new(&self.adb_path);

        if let Some(serial) = &self.device_serial {
            cmd.arg("-s").arg(serial);
        }

        if self.root {
            cmd.arg("shell").arg(format!("su root {}", command));
        } else {
            cmd.arg("shell").arg(command);
        }

        let output = cmd.output().context("Failed to execute adb command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "ADB command failed: {},{}",
                output.stdout.len(),
                stderr
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Execute an ADB pull command to get file content
    fn exec_pull(&self, remote_path: &str) -> Result<Vec<u8>> {
        use std::fs;

        // Create a temporary file for the pull operation
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!(
            "adb_pull_{}_{}.tmp",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));

        let mut cmd = Command::new(&self.adb_path);

        if let Some(serial) = &self.device_serial {
            cmd.arg("-s").arg(serial);
        }

        // Pull to temporary file
        cmd.arg("pull").arg(remote_path).arg(&temp_file);

        let output = cmd.output().context("Failed to execute adb pull")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let _ = fs::remove_file(&temp_file); // Clean up if it exists
            return Err(anyhow!("ADB pull failed: {}", stderr));
        }

        // Read the temporary file
        let data = fs::read(&temp_file).context("Failed to read temporary file")?;

        // Clean up
        let _ = fs::remove_file(&temp_file);

        Ok(data)
    }

    pub fn load_all(&self) -> Result<Vec<(OsString, FileInfo)>> {
        // find / -print0 | xargs -0 stat -c "%i|%A|%Z_%Y_%X|%U|%G|%s|%N"
        // find / -path /proc -prune -o -exec stat -c \"%i|%A|%Z|%Y|%X|%U|%G|%s|%N\" {} +
        let output = self.exec_pty(
            "find / -path /proc -prune -o -print0 | xargs -0 stat -c \"%i|%A|%Z|%Y|%X|%U|%G|%s|%N\"",
        )?;
        let mut results: Vec<(OsString, FileInfo)> = Vec::new();
        for line in output {
            let parts: Vec<&str> = line.splitn(9, '|').collect();
            if parts.len() < 9 {
                continue;
            }
            let path_part = parts[8];
            let path = path_part
                .split("->")
                .next()
                .unwrap_or("")
                .trim_matches('\'')
                .to_string();

            let file_info = FileInfo {
                inode: parts[0].parse().unwrap_or(0),
                permissions: parts[1].to_string(),
                modified_time: parts[3].parse().unwrap_or(0),
                accessed_time: parts[4].parse().unwrap_or(0),
                created_time: parts[2].parse().unwrap_or(0),
                user: parts[5].to_string(),
                group: parts[6].to_string(),
                size: parts[7].parse().unwrap_or(0),
            };

            results.push((path.into(), file_info));
        }
        println!("Loaded {} file entries from ADB", results.len());
        Ok(results)
    }

    //----------------------------------------------------------------------

    /// List all files and directories recursively with timestamps
    /// # Returns
    /// Vector of (path, modified_timestamp) tuples
    pub fn list_all(&self) -> Result<Vec<(String, usize)>> {
        let output = self.exec_shell("find / -d -printf \"%T@|%p\\n\"")?;
        let mut results = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.splitn(2, '|').collect();
            if parts.len() != 2 {
                continue;
            }
            let path = parts[1].to_string();
            let timestamp = parts[0]
                .split('.')
                .next()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
            results.push((path, timestamp as usize));
        }
        Ok(results)
    }

    pub fn list_active_apps_users(&self) -> Result<HashMap<String, String>> {
        let output = self.exec_shell("ps -o USER,NAME|grep ^u")?;
        let mut users: HashMap<String, String> = HashMap::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() != 2 {
                continue;
            }
            let app_name = parts[1].trim().to_string();
            users
                .entry(parts[0].to_string())
                .and_modify(|v| {
                    v.push(',');
                    v.push_str(&app_name);
                }) // if exists, append
                .or_insert(app_name);
        }
        Ok(users)
    }

    /// List files in a directory
    ///
    /// # Arguments
    /// * `path` - Path inside the emulator (e.g., "/sdcard/", "/data/local/tmp/")
    ///
    /// # Returns
    /// Vector of file/directory names (not full paths)
    pub fn list_files(&self, path: impl AsRef<Path>) -> Result<Vec<String>> {
        let path = path.as_ref().to_string_lossy();
        let output = self.exec_shell(&format!("ls '{}'", path))?;

        let files: Vec<String> = output
            .lines()
            .filter(|line| !line.is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        Ok(files)
    }

    /// List all folders recursively in a directory
    pub fn list_folders_tree(&self, path: impl AsRef<Path>) -> Result<Vec<String>> {
        let path = path.as_ref().to_string_lossy();
        let output = self.exec_shell(&format!("find '{}' -type d -print", path))?;

        let folders: Vec<String> = output
            .lines()
            .filter(|line| !line.is_empty())
            .filter(|line| !line.starts_with("find:"))
            .map(|s| s.trim().to_string())
            .collect();

        Ok(folders)
    }

    /// Get detailed file information (MAC times, size, permissions)
    ///
    /// # Arguments
    /// * `path` - Full path to file/directory in emulator
    ///
    /// # Returns
    /// FileInfo struct with all metadata
    // pub fn get_file_info(&self, path: impl AsRef<Path>) -> Result<FileInfo> {
    //     let path_str = path.as_ref().to_string_lossy();

    //     // Use stat command for detailed information
    //     // Format: %n (name) %s (size) %a (access time) %Y (modify time) %W (birth time) %f (raw mode) %U (owner) %G (group)
    //     let stat_output =
    //         self.exec_shell(&format!("stat -c '%n|%s|%X|%Y|%W|%f|%U|%G' '{}'", path_str))?;

    //     let parts: Vec<&str> = stat_output.trim().split('|').collect();
    //     if parts.len() < 8 {
    //         return Err(anyhow!("Invalid stat output: {}", stat_output));
    //     }

    //     let size = parts[1]
    //         .parse::<u64>()
    //         .context("Failed to parse file size")?;

    //     let access_time_secs = parts[2].parse::<u64>().ok();
    //     let modify_time_secs = parts[3]
    //         .parse::<u64>()
    //         .context("Failed to parse modification time")?;
    //     let birth_time_secs = parts[4].parse::<u64>().ok();

    //     let modified_time =
    //         SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(modify_time_secs);
    //     let accessed_time =
    //         access_time_secs.map(|s| SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(s));
    //     let created_time = birth_time_secs
    //         .filter(|&s| s != 0)
    //         .map(|s| SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(s));

    //     // Get file type and permissions using ls -ld
    //     let ls_output = self.exec_shell(&format!("ls -ld '{}'", path_str))?;
    //     let ls_parts: Vec<&str> = ls_output.trim().split_whitespace().collect();

    //     let file_type = if ls_parts.is_empty() {
    //         FileType::Other
    //     } else {
    //         let first_char = ls_parts[0].chars().next().unwrap_or('-');
    //         match first_char {
    //             'd' => FileType::Directory,
    //             'l' => FileType::Symlink,
    //             '-' => FileType::File,
    //             _ => FileType::Other,
    //         }
    //     };

    //     let permissions = if ls_parts.len() >= 3 {
    //         FilePermissions::from_ls_string(ls_parts[0], ls_parts[2], ls_parts[3])
    //     } else {
    //         FilePermissions {
    //             mode: 0,
    //             owner: parts[6].to_string(),
    //             group: parts[7].to_string(),
    //         }
    //     };

    //     Ok(FileInfo {
    //         path: PathBuf::from(parts[0]),
    //         file_type,
    //         size,
    //         permissions,
    //         modified_time,
    //         accessed_time,
    //         created_time,
    //     })
    // }

    // /// Read the entire content of a file (text or binary)
    ///
    /// # Arguments
    /// * `path` - Full path to file in emulator
    ///
    /// # Returns
    /// Raw bytes of the file content
    pub fn read_file(&self, path: impl AsRef<Path>) -> Result<Vec<u8>> {
        let path_str = path.as_ref().to_string_lossy();
        self.exec_pull(&path_str)
    }

    /// Read a text file as UTF-8 string
    ///
    /// # Arguments
    /// * `path` - Full path to file in emulator
    ///
    /// # Returns
    /// File content as string
    pub fn read_text_file(&self, path: impl AsRef<Path>) -> Result<String> {
        let bytes = self.read_file(path)?;
        String::from_utf8(bytes).context("File content is not valid UTF-8")
    }

    // pub fn list_files_detailed(&self, path: impl AsRef<Path>) -> Result<Vec<FileInfo>> {
    //     let path_ref = path.as_ref();
    //     let files = self.list_files(path_ref)?;

    //     let mut results = Vec::new();
    //     for file in files {
    //         let full_path = path_ref.join(&file);
    //         match self.get_file_info(&full_path) {
    //             Ok(info) => results.push(info),
    //             Err(e) => {
    //                 eprintln!(
    //                     "Warning: Failed to get info for {}: {}",
    //                     full_path.display(),
    //                     e
    //                 );
    //             }
    //         }
    //     }

    //     Ok(results)
    // }

    // #endregion
}
