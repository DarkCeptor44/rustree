use crate::tree::Node;
use std::{collections::BTreeMap, path::Path};

pub const ASCII_STYLE: TreeStyle = TreeStyle {
    tee: "|-- ",
    last: "\\-- ",
    vertical: "|   ",
    space: "    ",
};
pub const UNICODE_STYLE: TreeStyle = TreeStyle {
    tee: "├── ",
    last: "└── ",
    vertical: "│   ",
    space: "    ",
};

#[derive(Debug, Clone, Copy)]
pub struct TreeStyle {
    tee: &'static str,
    last: &'static str,
    vertical: &'static str,
    space: &'static str,
}

impl TreeStyle {
    pub fn print_tree(&self, children: &BTreeMap<String, Node>, prefix: &str) {
        let count = children.len();
        for (i, (name, node)) in children.iter().enumerate() {
            let is_last = i == count - 1;
            let connector = if is_last { self.last } else { self.tee };

            println!("{prefix}{connector}{name}");

            if let Node::Directory { children, .. } = node {
                let extension = if is_last { self.space } else { self.vertical };
                let new_prefix = format!("{prefix}{extension}");
                self.print_tree(children, &new_prefix);
            }
        }
    }
}

pub fn render_tree(root_path: &Path, root_map: &BTreeMap<String, Node>, use_ascii: bool) {
    let style = if use_ascii {
        ASCII_STYLE
    } else {
        UNICODE_STYLE
    };
    let root_name = root_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(".");

    println!("{root_name}");
    style.print_tree(root_map, "");
}
