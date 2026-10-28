use ignore::DirEntry;
use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};

#[derive(Debug)]
pub enum Node {
    File {
        path: PathBuf,
    },
    Directory {
        path: PathBuf,
        children: BTreeMap<String, Node>,
    },
}

impl Node {
    pub fn new_dir(path: PathBuf) -> Self {
        Self::Directory {
            path,
            children: BTreeMap::new(),
        }
    }

    pub fn new_file(path: PathBuf) -> Self {
        Self::File { path }
    }
}

pub fn insert_entry(root: &mut BTreeMap<String, Node>, root_path: &Path, entry: &DirEntry) {
    let path = entry.path();
    if path == root_path {
        return;
    }

    let relative = match path.strip_prefix(root_path) {
        Ok(rel) => rel,
        Err(_) => return,
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

    let is_dir = entry.file_type().is_some_and(|ft| ft.is_dir());
    let mut current_map = root;

    for (i, component) in components.iter().enumerate() {
        let is_last = i == components.len() - 1;
        if is_last {
            if is_dir {
                current_map
                    .entry(component.clone())
                    .or_insert_with(|| Node::new_dir(path.to_path_buf()));
            } else {
                current_map.insert(component.clone(), Node::new_file(path.to_path_buf()));
            }
        } else {
            let node = current_map
                .entry(component.clone())
                .or_insert_with(|| Node::new_dir(path.parent().unwrap().to_path_buf()));

            if let Node::Directory { children, .. } = node {
                current_map = children;
            } else {
                break;
            }
        }
    }
}
