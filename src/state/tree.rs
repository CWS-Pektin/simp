use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Category,
    MarkdownFile,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub path: PathBuf,
    pub name: String,
    pub kind: NodeKind,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn find_first_md_path(&self) -> Option<PathBuf> {
        match self.kind {
            NodeKind::MarkdownFile => Some(self.path.clone()),
            NodeKind::Category => {
                for c in &self.children {
                    if let Some(p) = c.find_first_md_path() {
                        return Some(p);
                    }
                }
                None
            }
        }
    }

    pub fn find_readme_at_root(&self, root: &std::path::Path) -> Option<PathBuf> {
        if self.path != root {
            return None;
        }
        for c in &self.children {
            if c.kind == NodeKind::MarkdownFile {
                let n = c.name.to_ascii_lowercase();
                if n == "readme.md" {
                    return Some(c.path.clone());
                }
            }
        }
        None
    }
}
