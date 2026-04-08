use log::{debug, trace};
use crate::model::files_diagram::{FilesDiagram, FilesEntry, FilesEntryKind};
use crate::Result;

fn extract_files_block(source: &str) -> Option<String> {
    let mut inside = false;
    let mut lines = Vec::new();
    for line in source.lines() {
        let t = line.trim_start();
        if inside {
            if t.starts_with("@endfiles") {
                break;
            }
            lines.push(line);
        } else if t.starts_with("@startfiles") {
            inside = true;
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

/// Parse a files diagram block.
///
/// Matches Java `FilesDiagram` semantics exactly: only lines whose first character is `/`
/// are accepted as entries. Any line starting with whitespace or another character is
/// silently ignored (this is intentional — Java treats indentation as a visual hint,
/// not structural).
///
/// Each accepted line is then fed to the root entry via `addRawEntry` which splits on
/// `/` recursively: the head becomes a folder (created if necessary), the remainder is
/// added to that folder. If the raw input contains no `/`, a file entry is created.
pub fn parse_files_diagram(source: &str) -> Result<FilesDiagram> {
    let block = extract_files_block(source).unwrap_or_else(|| source.to_string());
    debug!("parse_files_diagram: {} bytes", block.len());

    let mut root_children: Vec<FilesEntry> = Vec::new();
    for (n, line) in block.lines().enumerate() {
        if !line.starts_with('/') {
            trace!("line {}: skip (does not start with '/')", n + 1);
            continue;
        }
        let raw = &line[1..];
        add_raw_entry(&mut root_children, raw);
    }
    Ok(FilesDiagram {
        entries: root_children,
    })
}

fn add_raw_entry(children: &mut Vec<FilesEntry>, raw: &str) {
    match raw.find('/') {
        None => {
            children.push(FilesEntry {
                name: raw.to_string(),
                kind: FilesEntryKind::File,
                children: Vec::new(),
            });
        }
        Some(idx) => {
            let folder_name = &raw[..idx];
            let remain = &raw[idx + 1..];
            let folder = get_or_create_folder(children, folder_name);
            if !remain.is_empty() {
                add_raw_entry(&mut folder.children, remain);
            }
        }
    }
}

fn get_or_create_folder<'a>(
    children: &'a mut Vec<FilesEntry>,
    name: &str,
) -> &'a mut FilesEntry {
    let existing = children
        .iter()
        .position(|c| c.kind == FilesEntryKind::Folder && c.name == name);
    match existing {
        Some(i) => &mut children[i],
        None => {
            children.push(FilesEntry {
                name: name.to_string(),
                kind: FilesEntryKind::Folder,
                children: Vec::new(),
            });
            children.last_mut().unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Java PlantUML behavior: only lines starting with `/` count; indentation ignored.
    #[test]
    fn indented_children_are_ignored() {
        let d = parse_files_diagram(
            "@startfiles\n/etc\n  nginx.conf\n  sshd_config\n/var\n  syslog\n@endfiles",
        )
        .unwrap();
        // Only `/etc` and `/var` are accepted; both become FILE entries (no '/' in raw).
        assert_eq!(d.entries.len(), 2);
        assert_eq!(d.entries[0].name, "etc");
        assert_eq!(d.entries[0].kind, FilesEntryKind::File);
        assert_eq!(d.entries[0].children.len(), 0);
        assert_eq!(d.entries[1].name, "var");
        assert_eq!(d.entries[1].kind, FilesEntryKind::File);
    }

    #[test]
    fn slash_notation_creates_nested_tree() {
        let d = parse_files_diagram(
            "@startfiles\n/etc/nginx/nginx.conf\n/etc/ssh/sshd_config\n@endfiles",
        )
        .unwrap();
        assert_eq!(d.entries.len(), 1);
        assert_eq!(d.entries[0].name, "etc");
        assert_eq!(d.entries[0].kind, FilesEntryKind::Folder);
        assert_eq!(d.entries[0].children.len(), 2);
        assert_eq!(d.entries[0].children[0].name, "nginx");
        assert_eq!(d.entries[0].children[0].kind, FilesEntryKind::Folder);
        assert_eq!(d.entries[0].children[0].children[0].name, "nginx.conf");
        assert_eq!(
            d.entries[0].children[0].children[0].kind,
            FilesEntryKind::File
        );
    }

    #[test]
    fn indented_slash_lines_are_ignored() {
        let d = parse_files_diagram(
            "@startfiles\n/etc\n  /nginx\n    nginx.conf\n    mime.types\n  /ssh\n    sshd_config\n@endfiles",
        )
        .unwrap();
        // Only `/etc` qualifies (top-level `/`), the rest are indented and ignored.
        assert_eq!(d.entries.len(), 1);
        assert_eq!(d.entries[0].name, "etc");
        assert_eq!(d.entries[0].kind, FilesEntryKind::File);
    }

    #[test]
    fn single_top_level_file() {
        let d = parse_files_diagram("@startfiles\n/home\n  /user\n@endfiles").unwrap();
        assert_eq!(d.entries.len(), 1);
        assert_eq!(d.entries[0].name, "home");
        assert_eq!(d.entries[0].kind, FilesEntryKind::File);
    }
}
