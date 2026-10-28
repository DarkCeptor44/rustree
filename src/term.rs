use crate::tree::Node;
use colored::Colorize;
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
    pub fn print_tree(
        &self,
        children: &BTreeMap<String, Node>,
        prefix: &str,
        color: &str,
        show_size: bool,
    ) {
        let count = children.len();
        for (i, (name, node)) in children.iter().enumerate() {
            let is_last = i == count - 1;
            let connector = if is_last { self.last } else { self.tee };
            let display_name = match node {
                Node::Directory { .. } => name.bold().color(color),
                Node::File { .. } => name.normal(),
            };

            if show_size {
                if let Node::File { size } = node
                    && let Some(size) = size
                {
                    let size_str = format!(" ({})", human_bytes(*size)).dimmed();
                    println!("{prefix}{connector}{display_name}{size_str}");
                } else {
                    println!("{prefix}{connector}{display_name}");
                }
            } else {
                println!("{prefix}{connector}{display_name}");
            }

            if let Node::Directory { children, .. } = node {
                let extension = if is_last { self.space } else { self.vertical };
                let new_prefix = format!("{prefix}{extension}");
                self.print_tree(children, &new_prefix, color, show_size);
            }
        }
    }
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    let mut bytes = bytes as f64;
    let mut idx = 0;

    while bytes >= 1024.0 && idx < UNITS.len() - 1 {
        bytes /= 1024.0;
        idx += 1;
    }

    let n = if idx == 0 { 0 } else { 1 };
    format!("{bytes:.n$} {}", UNITS[idx])
}

pub fn render_tree(
    root_path: &Path,
    root_map: &BTreeMap<String, Node>,
    use_ascii: bool,
    color: &str,
    show_size: bool,
) {
    let style = if use_ascii {
        ASCII_STYLE
    } else {
        UNICODE_STYLE
    };
    let root_name = root_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(".");

    println!("{}", root_name.bold().color(color));
    style.print_tree(root_map, "", color, show_size);
}
