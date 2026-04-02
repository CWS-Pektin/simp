use std::path::{Path, PathBuf};

/// Resolve a markdown link target to a local filesystem path.
///
/// Returns `None` for empty hrefs, `http`/`https`/`mailto`, or unresolvable `file:` URLs.
/// Strips `#fragment` and `?query`. Relative paths are resolved from `base_file`'s parent.
pub fn resolve_local_markdown_link(href: &str, base_file: &Path) -> Option<PathBuf> {
    let href = href.trim();
    if href.is_empty() {
        return None;
    }
    let href = href.split('#').next().unwrap_or(href);
    let href = href.split('?').next().unwrap_or(href);
    let href = href.trim();
    if href.is_empty() {
        return None;
    }

    let lower = href.to_ascii_lowercase();
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
    {
        return None;
    }

    let path = if let Some(rest) = href.strip_prefix("file://") {
        #[cfg(windows)]
        {
            PathBuf::from(rest.trim_start_matches('/'))
        }
        #[cfg(not(windows))]
        {
            PathBuf::from(rest)
        }
    } else if Path::new(href).is_absolute() {
        PathBuf::from(href)
    } else {
        let base = base_file.parent()?;
        base.join(href)
    };

    Some(if path.exists() {
        path.canonicalize().unwrap_or(path)
    } else {
        path
    })
}

/// Whether `path` lies under `root` (same project folder), using canonical paths when possible.
/// Plain `Path::starts_with` is unreliable on Windows (mixed separators, casing, `\\?\` prefix).
pub fn path_within_root(path: &Path, root: &Path) -> bool {
    let root_resolved = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let path_resolved = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    path_resolved.starts_with(&root_resolved)
}

/// Stable key for per-file settings so the same file matches after restart (Windows paths).
pub fn content_mode_key(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}
