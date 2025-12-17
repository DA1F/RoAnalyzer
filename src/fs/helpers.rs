#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
    Symlink,
    Other,
}
impl Default for FileType {
    fn default() -> Self {
        FileType::Directory
    }
}

impl From<&char> for FileType {
    fn from(s: &char) -> Self {
        match s {
            '-' => FileType::File,
            'd' => FileType::Directory,
            'l' => FileType::Symlink,
            _ => FileType::Other,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FileInfo {
    pub inode: usize,
    pub permissions: String,
    pub created_time: usize,
    pub modified_time: usize,
    pub accessed_time: usize,
    pub user: String,
    pub group: String,
    pub size: u64,
}
