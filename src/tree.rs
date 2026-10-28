use ignore::DirEntry;
use std::{
    collections::BTreeMap,
    path::{Component, Path},
};

#[derive(Debug)]
pub enum Node {
    File { size: Option<u64> },
    Directory { children: BTreeMap<String, Node> },
}

impl Node {
    pub fn new_dir() -> Self {
        Self::Directory {
            children: BTreeMap::new(),
        }
    }

    pub fn new_file(size: Option<u64>) -> Self {
        Self::File { size }
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
    let size = entry
        .metadata()
        .map_or(None, |m| if m.is_file() { Some(m.len()) } else { None });
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
