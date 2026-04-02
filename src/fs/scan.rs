use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::state::{NodeKind, TreeNode};

fn is_asset_dir(name: &str) -> bool {
    name == "asset"
}

fn is_markdown_extension(ext: &std::ffi::OsStr) -> bool {
    ext.to_string_lossy().eq_ignore_ascii_case("md")
}

pub fn is_markdown_file(path: &Path) -> bool {
    path
        .extension()
        .map(|e| is_markdown_extension(e))
        .unwrap_or(false)
}

/// Build a single root tree node for `root` with children sorted: dirs first, then .md files, alphabetical.
pub fn build_tree(root: &Path) -> anyhow::Result<TreeNode> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let name = root
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.to_string_lossy().into_owned());

    let children = scan_children(&root)?;
    Ok(TreeNode {
        path: root,
        name,
        kind: NodeKind::Category,
        children,
    })
}

fn scan_children(dir: &Path) -> anyhow::Result<Vec<TreeNode>> {
    let mut dirs: BTreeMap<String, PathBuf> = BTreeMap::new();
    let mut files: BTreeMap<String, PathBuf> = BTreeMap::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy().into_owned();

        if path.is_dir() {
            if is_asset_dir(&name) {
                continue;
            }
            dirs.insert(name, path);
        } else if path.is_file() && is_markdown_file(&path) {
            files.insert(name, path);
        }
    }

    let mut out = Vec::new();
    for (_k, path) in dirs {
        let n = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let children = scan_children(&path)?;
        out.push(TreeNode {
            path,
            name: n,
            kind: NodeKind::Category,
            children,
        });
    }
    for (_k, path) in files {
        let n = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        out.push(TreeNode {
            path,
            name: n,
            kind: NodeKind::MarkdownFile,
            children: vec![],
        });
    }
    Ok(out)
}

pub fn first_md_in_tree_order(root: &TreeNode) -> Option<PathBuf> {
    root.find_readme_at_root(&root.path)
        .or_else(|| root.find_first_md_path())
}

pub fn default_open_path(root: &TreeNode) -> Option<PathBuf> {
    first_md_in_tree_order(root)
}

#[allow(dead_code)]
pub fn all_markdown_paths(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let walker = WalkDir::new(root).follow_links(false).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            if let Some(name) = e.file_name().to_str() {
                if is_asset_dir(name) {
                    return false;
                }
            }
        }
        true
    });
    for e in walker.filter_map(|e| e.ok()) {
        if e.path().is_file() && is_markdown_file(e.path()) {
            out.push(e.path().to_path_buf());
        }
    }
    out.sort();
    out
}
