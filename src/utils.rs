use anyhow::Result;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let rel = path.strip_prefix(src)?;
        let target = dst.join(rel);

        if path.is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_copy_dir_recursive_basic() {
        let src = tempdir().unwrap();
        let dst = tempdir().unwrap();

        // Create source structure: file.txt, sub/nested.txt
        fs::write(src.path().join("file.txt"), "hello").unwrap();
        fs::create_dir_all(src.path().join("sub")).unwrap();
        fs::write(src.path().join("sub/nested.txt"), "world").unwrap();

        copy_dir_recursive(src.path(), dst.path()).unwrap();

        assert_eq!(fs::read_to_string(dst.path().join("file.txt")).unwrap(), "hello");
        assert_eq!(fs::read_to_string(dst.path().join("sub/nested.txt")).unwrap(), "world");
    }

    #[test]
    fn test_copy_dir_recursive_empty() {
        let src = tempdir().unwrap();
        let dst = tempdir().unwrap();

        copy_dir_recursive(src.path(), dst.path()).unwrap();
        // Should succeed without error on empty source
    }
}
