#[derive(Debug, Clone, PartialEq)]
pub enum FilesEntryKind { Folder, File }
#[derive(Debug, Clone)]
pub struct FilesEntry {
    pub name: String,
    pub kind: FilesEntryKind,
    pub children: Vec<FilesEntry>,
}
#[derive(Debug, Clone)]
pub struct FilesDiagram { pub entries: Vec<FilesEntry> }
