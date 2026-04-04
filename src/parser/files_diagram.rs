use log::{debug, trace};
use crate::model::files_diagram::{FilesDiagram, FilesEntry, FilesEntryKind};
use crate::Result;
fn extract_files_block(source: &str) -> Option<String> {
    let mut inside = false; let mut lines = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        if inside { if t.starts_with("@endfiles") { break; } lines.push(line); }
        else if t.starts_with("@startfiles") { inside = true; }
    }
    if lines.is_empty() { None } else { Some(lines.join("\n")) }
}
pub fn parse_files_diagram(source: &str) -> Result<FilesDiagram> {
    let block = extract_files_block(source).unwrap_or_else(|| source.to_string());
    debug!("parse_files_diagram: {} bytes", block.len());
    let mut flat: Vec<(usize, String, bool)> = Vec::new();
    for (n, line) in block.lines().enumerate() {
        let t = line.trim();
        if t.is_empty() || t.starts_with("'") { trace!("line {}: skip", n+1); continue; }
        let ls = line.len() - line.trim_start().len();
        if ls == 0 && t.starts_with('/') {
            let parts: Vec<&str> = t.trim_start_matches('/').split('/').filter(|p| !p.is_empty()).collect();
            for (i, p) in parts.iter().enumerate() { flat.push((i, p.to_string(), i < parts.len()-1)); }
            continue;
        }
        let level = ls / 2;
        let is_folder = t.starts_with('/') || t.ends_with('/');
        let clean = t.trim_start_matches('/').trim_end_matches('/').to_string();
        if !clean.is_empty() { flat.push((level, clean, is_folder)); }
    }
    Ok(FilesDiagram { entries: build_tree(&flat) })
}
fn build_tree(flat: &[(usize, String, bool)]) -> Vec<FilesEntry> {
    let mut result: Vec<FilesEntry> = Vec::new();
    let mut i = 0;
    while i < flat.len() {
        let (level, ref name, is_folder) = flat[i];
        let cs = i + 1;
        let mut ce = cs;
        while ce < flat.len() && flat[ce].0 > level { ce += 1; }
        let children = if cs < ce { build_tree(&flat[cs..ce].iter().map(|(l,n,f)|(*l,n.clone(),*f)).collect::<Vec<_>>()) } else { vec![] };
        let kind = if is_folder || !children.is_empty() { FilesEntryKind::Folder } else { FilesEntryKind::File };
        if let Some(e) = result.iter_mut().find(|e| e.name == *name && e.kind == FilesEntryKind::Folder && kind == FilesEntryKind::Folder) {
            e.children.extend(children);
        } else { result.push(FilesEntry { name: name.clone(), kind, children }); }
        i = ce;
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_indent() {
        let d = parse_files_diagram("@startfiles\n/etc\n  nginx.conf\n  sshd_config\n/var\n  syslog\n@endfiles").unwrap();
        assert_eq!(d.entries.len(), 2); assert_eq!(d.entries[0].children.len(), 2);
    }
    #[test] fn test_slash() {
        let d = parse_files_diagram("@startfiles\n/etc/nginx/nginx.conf\n/etc/ssh/sshd_config\n@endfiles").unwrap();
        assert_eq!(d.entries.len(), 1); assert_eq!(d.entries[0].children.len(), 2);
    }
    #[test] fn test_deep() {
        let d = parse_files_diagram("@startfiles\n/etc\n  /nginx\n    nginx.conf\n    mime.types\n  /ssh\n    sshd_config\n@endfiles").unwrap();
        assert_eq!(d.entries.len(), 1); assert_eq!(d.entries[0].children.len(), 2);
    }
}
