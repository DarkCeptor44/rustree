// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{DirEntry, FileType};
use std::{
    collections::BTreeMap,
    fs::Metadata,
    path::{Component, Path},
};

/// Represents a file or directory
#[derive(Debug)]
pub enum Node {
    /// Represents a file
    File {
        /// Potential size of the file, if available
        size: Option<u64>,
    },

    /// Represents a directory
    Directory {
        /// Children of the directory
        children: BTreeMap<String, Node>,
    },
}

impl Node {
    /// Creates a new directory
    ///
    /// ## Returns
    ///
    /// The new node
    #[must_use]
    pub fn new_dir() -> Self {
        Self::Directory {
            children: BTreeMap::new(),
        }
    }

    //// Creates a new file
    ///
    /// ## Arguments
    ///
    /// * `size` - The size of the file, if available
    ///
    /// ## Returns
    ///
    /// The new node
    pub fn new_file<T>(size: T) -> Self
    where
        T: Into<Option<u64>>,
    {
        fn inner(size: Option<u64>) -> Node {
            Node::File { size }
        }

        inner(size.into())
    }
}

/// Inserts an entry into the tree
///
/// ## Arguments
///
/// * `root` - The root tree object
/// * `root_path` - The root path
/// * `entry` - The entry to insert
/// * `calculate_size` - Whether to calculate the size of the files
pub fn insert_entry(
    root: &mut BTreeMap<String, Node>,
    root_path: &Path,
    entry: &DirEntry,
    calculate_size: bool,
) {
    let path = entry.path();
    if path == root_path {
        return;
    }

    let Ok(relative) = path.strip_prefix(root_path) else {
        return;
    };

    let components: Vec<String> = relative
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    if components.is_empty() {
        return;
    }

    let is_dir = entry
        .file_type
        .is_some_and(|ft| matches!(ft, FileType::Dir));
    let size = if calculate_size {
        entry
            .path
            .metadata()
            .ok()
            .filter(Metadata::is_file)
            .map(|m| m.len())
    } else {
        None
    };
    let mut current_map = root;

    for (i, component) in components.iter().enumerate() {
        let is_last = i == components.len() - 1;
        if is_last {
            if is_dir {
                current_map
                    .entry(component.clone())
                    .or_insert_with(Node::new_dir);
            } else {
                current_map.insert(component.clone(), Node::new_file(size));
            }
        } else {
            let node = current_map
                .entry(component.clone())
                .or_insert_with(Node::new_dir);

            if let Node::Directory { children, .. } = node {
                current_map = children;
            } else {
                break;
            }
        }
    }
}
