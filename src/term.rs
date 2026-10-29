use crate::tree::Node;
use colored::Colorize;
use std::{
    collections::BTreeMap,
    io::{BufWriter, Write},
    path::Path,
};

/// ASCII Tree Style
pub const ASCII_STYLE: TreeStyle = TreeStyle {
    tee: "|-- ",
    last: "\\-- ",
    vertical: "|   ",
    space: "    ",
};

/// Unicode Tree Style
pub const UNICODE_STYLE: TreeStyle = TreeStyle {
    tee: "├── ",
    last: "└── ",
    vertical: "│   ",
    space: "    ",
};

/// Tree style helper struct
#[derive(Debug, Clone, Copy)]
pub struct TreeStyle {
    tee: &'static str,
    last: &'static str,
    vertical: &'static str,
    space: &'static str,
}

impl TreeStyle {
    /// Prints a tree's node
    ///
    /// ## Arguments
    ///
    /// * `writer` - The writer to write to
    /// * `children` - The tree or map with nodes to print
    /// * `prefix` - The prefix to add to the node
    /// * `color` - The color to use for the node, can be `off`, `blue` for blue, or hexcode (e.g. `#ff0000` for red)
    /// * `show_size` - Whether to show the size of the node
    ///
    /// ## Panics
    ///
    /// Panics if the writer fails
    pub fn print_tree<W>(
        &self,
        writer: &mut W,
        children: &BTreeMap<String, Node>,
        prefix: &str,
        color: &str,
        show_size: bool,
    ) where
        W: Write,
    {
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
                    let _ = writeln!(writer, "{prefix}{connector}{display_name}{size_str}");
                } else {
                    let _ = writeln!(writer, "{prefix}{connector}{display_name}");
                }
            } else {
                let _ = writeln!(writer, "{prefix}{connector}{display_name}");
            }

            if let Node::Directory { children, .. } = node {
                let extension = if is_last { self.space } else { self.vertical };
                let new_prefix = format!("{prefix}{extension}");
                self.print_tree(writer, children, &new_prefix, color, show_size);
            }
        }
    }
}

/// Converts bytes to a human readable string
///
/// ## Arguments
///
/// * `bytes` - The bytes to convert
///
/// ## Returns
///
/// The human readable string
#[must_use]
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    #[allow(clippy::cast_precision_loss)]
    let mut bytes = bytes as f64;
    let mut idx = 0;

    while bytes >= 1024.0 && idx < UNITS.len() - 1 {
        bytes /= 1024.0;
        idx += 1;
    }

    let n = usize::from(idx != 0);
    format!("{bytes:.n$} {}", UNITS[idx])
}

/// Renders a tree to the terminal
///
/// ## Arguments
///
/// * `root_path` - The root path
/// * `root_map` - The root tree
/// * `use_ascii` - Whether to use ASCII characters
/// * `color` - The color to use for the tree, can be `off`, `blue` for blue, or hexcode (e.g. `#ff0000` for red)
/// * `show_size` - Whether to show the size of the files
///
/// ## Panics
///
/// Panics if the writer fails
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

    let stdout = std::io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    let _ = writeln!(writer, "{}", root_name.bold().color(color));
    style.print_tree(&mut writer, root_map, "", color, show_size);
}
