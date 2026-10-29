use ignore::gitignore::{Gitignore, GitignoreBuilder};
use parking_lot::Mutex;
use std::{
    collections::VecDeque,
    fs::read_dir,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

pub(crate) type Queue = Arc<Mutex<VecDeque<(PathBuf, Vec<Gitignore>)>>>;

/// A directory entry
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DirEntry {
    pub(crate) path: PathBuf,

    /// The type of the entry
    pub file_type: Option<FileType>,
}

impl From<ignore::DirEntry> for DirEntry {
    fn from(value: ignore::DirEntry) -> Self {
        DirEntry::new(value.path())
    }
}

impl DirEntry {
    /// Creates a new directory entry
    ///
    /// ## Arguments
    ///
    /// * `path` - The path of the entry
    ///
    /// ## Returns
    ///
    /// The new directory entry
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        fn inner(path: &Path) -> DirEntry {
            let file_type = if path.is_dir() {
                Some(FileType::Dir)
            } else if path.is_file() {
                Some(FileType::File)
            } else if path.is_symlink() {
                Some(FileType::Symlink)
            } else {
                None
            };

            DirEntry {
                path: path.to_path_buf(),
                file_type,
            }
        }

        inner(path.as_ref())
    }

    /// Returns the path of the entry
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// The type of an entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FileType {
    /// A regular file
    File,

    /// A directory
    Dir,

    /// A symbolic link
    Symlink,
}

impl FileType {
    /// Returns whether the entry is a directory
    #[must_use]
    pub fn is_dir(&self) -> bool {
        matches!(self, Self::Dir)
    }

    /// Returns whether the entry is a file
    #[must_use]
    pub fn is_file(&self) -> bool {
        matches!(self, Self::File)
    }

    /// Returns whether the entry is a symbolic link
    #[must_use]
    pub fn is_symlink(&self) -> bool {
        matches!(self, Self::Symlink)
    }
}

pub(crate) fn worker_loop(
    queue: &Queue,
    results: &Arc<Mutex<Vec<DirEntry>>>,
    active_workers: &Arc<AtomicUsize>,
    respect_gitignore: bool,
) {
    loop {
        let work = {
            let mut q = queue.lock();
            q.pop_front()
        };

        let Some((path, ignore_stack)) = work else {
            break;
        };

        let Ok(entries) = read_dir(&path) else {
            continue;
        };

        let mut local_results = vec![DirEntry::new(path)];
        let mut subdirs = Vec::new();

        for entry in entries.flatten() {
            let p = entry.path();
            let is_dir = entry.file_type().is_ok_and(|ft| ft.is_dir());

            let mut ignored = false;
            if respect_gitignore {
                for gitignore in &ignore_stack {
                    if gitignore.matched(&p, is_dir).is_ignore() {
                        ignored = true;
                        break;
                    }
                }
            }

            if ignored {
                continue;
            }

            if is_dir {
                subdirs.push(p);
            } else {
                local_results.push(DirEntry::new(p));
            }
        }

        results.lock().extend(local_results);

        if !subdirs.is_empty() {
            let mut q = queue.lock();
            for subdir in subdirs {
                let mut child_stack = ignore_stack.clone();
                let gitignore_path = subdir.join(".gitignore");
                if gitignore_path.is_file() {
                    let mut builder = GitignoreBuilder::new(&subdir);
                    builder.add(&gitignore_path);
                    if let Ok(built) = builder.build() {
                        child_stack.push(built);
                    }
                }

                q.push_back((subdir, child_stack));
            }
        }
    }

    active_workers.fetch_sub(1, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Walker;
    use std::fs::{create_dir, write};
    use tempfile::{TempDir, tempdir};

    #[test]
    fn test_walk_directory_respects_gitignore() {
        let dir = setup_test_tree();
        let walker = Walker::new(dir.path());
        let entries = make_relative(dir.path(), &walker.walk());

        assert_eq!(dbg!(&entries).len(), 7);
        assert_eq!(entries[0], Path::new(""));
        assert_eq!(entries[1], Path::new("dir_0"));
        assert_eq!(entries[2], Path::new("dir_0\\file_0"));
        assert_eq!(entries[3], Path::new("dir_0\\file_1"));
        assert_eq!(entries[4], Path::new("dir_1"));
        assert_eq!(entries[5], Path::new("dir_1\\file_0"));
        assert_eq!(entries[6], Path::new("dir_1\\file_1"));
    }

    #[test]
    fn test_walk_directory_show_all() {
        let dir = setup_test_tree();
        let walker = Walker::new(dir.path()).all(true);
        let entries = make_relative(dir.path(), &walker.walk());

        assert_eq!(dbg!(&entries).len(), 13);
        assert_eq!(entries[0], Path::new(""));
        assert_eq!(entries[1], Path::new(".git"));
        assert_eq!(entries[2], Path::new(".gitignore"));
        assert_eq!(entries[3], Path::new("dir_0"));
        assert_eq!(entries[4], Path::new("dir_0\\file_0"));
        assert_eq!(entries[5], Path::new("dir_0\\file_1"));
        assert_eq!(entries[6], Path::new("dir_1"));
        assert_eq!(entries[7], Path::new("dir_1\\file_0"));
        assert_eq!(entries[8], Path::new("dir_1\\file_1"));
        assert_eq!(entries[9], Path::new("target"));
        assert_eq!(entries[10], Path::new("target\\file_0"));
        assert_eq!(entries[11], Path::new("target\\file_1"));
        assert_eq!(entries[12], Path::new("target\\file_2"));
    }

    fn make_relative(root_path: &Path, paths: &[DirEntry]) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = paths
            .iter()
            .map(|p| p.path.strip_prefix(root_path).unwrap().to_path_buf())
            .collect();

        paths.sort_unstable();
        paths
    }

    fn setup_test_tree() -> TempDir {
        let dir = tempdir().unwrap();
        for i in 0..2 {
            let dir_path = dir.path().join(format!("dir_{i}"));
            create_dir(&dir_path).unwrap();

            for j in 0..2 {
                let file_path = dir_path.join(format!("file_{j}"));
                write(&file_path, b"rustree test payload").unwrap();
            }
        }

        let git_path = dir.path().join(".git");
        create_dir(&git_path).unwrap();
        let target_path = dir.path().join("target");
        create_dir(&target_path).unwrap();
        for i in 0..3 {
            let file_path = target_path.join(format!("file_{i}"));
            write(&file_path, b"rustree test payload").unwrap();
        }

        write(dir.path().join(".gitignore"), b"target/\n.gitignore\n.git").unwrap();

        dir
    }
}
