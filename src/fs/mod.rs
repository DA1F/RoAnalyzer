mod adb;
mod filesystem;
mod helpers;

use adb::AdbHelper;
pub use filesystem::{FSNode, FileSystem};
pub use helpers::{FileInfo, FileType};

#[cfg(test)]
mod tests {
    use super::*; // <-- access private items!
    #[test]
    fn test_newfs() {
        use std::path::Path;
        use FileSystem;
        println!("Testing ADB Filesystem creation...");
        let mut fs = FileSystem::new(None);
        fs.refresh().expect("Failed to refresh filesystem");

        let jdata = fs.subtree_as_json(Path::new("/storage/emulated/0"));
        println!("{}", jdata);
        println!("DOne");
    }
    #[test]
    fn test_adb() {
        use super::AdbHelper;
        let adb = AdbHelper::new(None);

        let output = adb
            .exec_pty("find / -path /proc -prune -o -print0 | xargs -0 stat -c \"%i|%A|%Z|%Y|%X|%U|%G|%s|%N\"")
            .expect("Failed to exec shell");

        println!("Output:\n{}", output[0..10].join("\n"));

        //let output = adb.exec_tty(&["su root","find / -path /proc -prune -o -print0 | xargs -0 stat -c \"%i|%A|%Z|%Y|%X|%U|%G|%s|%N\""]).expect("Failed to exec shell");
        //println!("ADB whoami output: {}", output);
    }
}
