#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_debug_implementations, missing_docs)]

//! # rustree
//!
//! A Rust implementation of the `tree` command-line utility.
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use rustree::{Node, Walker, insert_entry, render_tree};
//! use std::{collections::BTreeMap, path::Path};
//!
//! let root = Path::new(".");
//! let entries = Walker::new(root).threshold(100).all(false).walk();
//!
//! // if you want to use ignore, this crate already provides
//! // a From<ignore::DirEntry> for DirEntry.
//! // if you want to use another crate for directory traversal,
//! // it's pretty easy to convert a DirEntry to a rustree::DirEntry,
//! // all you need is a PathBuf.
//!
//! let mut root_map: BTreeMap<String, Node> = BTreeMap::new();
//! for entry in entries {
//!     insert_entry(&mut root_map, root, &entry, true);
//! }
//!
//! render_tree(root, &root_map, false, "blue", true);
//! ```

mod error;
mod fs;
mod term;
mod tree;

pub use crate::{
    error::Error,
    fs::{DirEntry, FileType},
    term::{ASCII_STYLE, TreeStyle, UNICODE_STYLE, human_bytes, render_tree},
    tree::{Node, insert_entry},
};

use crate::fs::worker_loop;
use ignore::gitignore::GitignoreBuilder;
use parking_lot::Mutex;
use std::{
    collections::VecDeque,
    fs::read_dir,
    num::NonZero,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread::{available_parallelism, spawn, yield_now},
};

/// A directory walker with gitignore support
#[derive(Debug, Clone)]
pub struct Walker {
    root: PathBuf,
    threshold: usize,
    show_all: bool,
}

impl Walker {
    /// Creates a new directory walker
    ///
    /// ## Arguments
    ///
    /// * `root` - The root directory to walk
    ///
    /// ## Returns
    ///
    /// A new directory walker
    pub fn new<P>(root: P) -> Self
    where
        P: Into<PathBuf>,
    {
        fn inner(root: PathBuf) -> Walker {
            Walker {
                root,
                threshold: 100,
                show_all: false,
            }
        }

        inner(root.into())
    }

    /// Sets whether to show all entries or respect gitignore files
    ///
    /// ## Arguments
    ///
    /// * `show_all` - Whether to show all entries or respect gitignore files
    ///
    /// ## Returns
    ///
    /// A new directory walker
    #[must_use]
    pub fn all(mut self, show_all: bool) -> Self {
        self.show_all = show_all;
        self
    }

    /// Sets the threshold at which it starts parallelizing the walk
    ///
    /// ## Arguments
    ///
    /// * `threshold` - The threshold at which it starts parallelizing the walk
    ///
    /// ## Returns
    ///
    /// A new directory walker
    #[must_use]
    pub fn threshold<T>(self, threshold: T) -> Self
    where
        T: Into<Option<usize>>,
    {
        fn inner(mut walker: Walker, threshold: Option<usize>) -> Walker {
            walker.threshold = match threshold {
                Some(0) | None => 100,
                Some(t) => t,
            };
            walker
        }

        inner(self, threshold.into())
    }

    // END OF BUILDERS

    /// Walks a directory recursively using a hybrid approach. It starts with a single thread/sequential walk using [std], then as soon as it hits the threshold, it starts parallelizing the walk using [`std::thread`].
    ///
    /// ## Returns
    ///
    /// A vector of directory entries
    #[allow(clippy::too_many_lines)]
    pub fn walk(&self) -> Vec<DirEntry> {
        let root_path = &self.root;
        let root_gitignore = root_path.join(".gitignore");
        let respect_gitignore = !self.show_all;

        let mut initial_stack = Vec::new();
        if root_gitignore.is_file() {
            let mut builder = GitignoreBuilder::new(root_path);
            builder.add(&root_gitignore);
            if let Ok(built) = builder.build() {
                initial_stack.push(built);
            }
        }

        let queue = Arc::new(Mutex::new(VecDeque::from([(
            root_path.clone(),
            initial_stack,
        )])));
        let results: Arc<Mutex<Vec<DirEntry>>> = Arc::new(Mutex::new(Vec::new()));

        let active_workers = Arc::new(AtomicUsize::new(0));
        let mut worker_handles = Vec::new();

        let mut items_seen = 0;
        let mut parallelized = false;

        loop {
            let work = {
                let mut q = queue.lock();
                q.pop_front()
            };

            let Some((path, ignore_stack)) = work else {
                if active_workers.load(Ordering::Acquire) == 0 {
                    break;
                }
                yield_now();
                continue;
            };

            items_seen += 1;

            if !parallelized && items_seen >= self.threshold {
                parallelized = true;
                let num_threads = available_parallelism().map_or(4, NonZero::get);

                for _ in 0..num_threads {
                    let q = queue.clone();
                    let r = results.clone();
                    let active = active_workers.clone();

                    active.fetch_add(1, Ordering::Relaxed);

                    let handle = spawn(move || {
                        worker_loop(&q, &r, &active, respect_gitignore);
                    });
                    worker_handles.push(handle);
                }
            }

            let Ok(entries) = read_dir(&path) else {
                continue;
            };

            let mut local_results = vec![DirEntry::new(path)];
            let mut subdirs = Vec::new();

            for entry in entries.flatten() {
                if respect_gitignore {
                    let name = entry.file_name();
                    if name == ".git" || name == ".gitignore" {
                        continue;
                    }
                }

                let p = entry.path();
                let file_type = entry.to_file_type();
                let is_dir = file_type.is_some_and(|ft| matches!(ft, FileType::Dir));

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
                    local_results.push(DirEntry { path: p, file_type });
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

        for handle in worker_handles {
            let _ = handle.join();
        }

        match Arc::try_unwrap(results) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().clone(),
        }
    }
}

/// Trait for converting a [`std::fs::DirEntry`] to a [`FileType`]
pub trait ToFileType {
    /// Converts a [`std::fs::DirEntry`] to a [`FileType`]
    ///
    /// ## Returns
    ///
    /// An [Option] of [`FileType`]
    fn to_file_type(&self) -> Option<FileType>;
}

impl ToFileType for std::fs::DirEntry {
    fn to_file_type(&self) -> Option<FileType> {
        match self.file_type() {
            Ok(ft) if ft.is_dir() => Some(FileType::Dir),
            Ok(ft) if ft.is_file() => Some(FileType::File),
            Ok(ft) if ft.is_symlink() => Some(FileType::Symlink),
            _ => None,
        }
    }
}
