#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_debug_implementations, missing_docs)]

//! # rustree
//!
//! A Rust implementation of the `tree` command-line utility.
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use ignore::WalkBuilder;
//! use rustree::{Node, insert_entry, render_tree};
//! use std::{collections::BTreeMap, path::Path};
//!
//! let root = Path::new(".");
//! let mut root_map: BTreeMap<String, Node> = BTreeMap::new();
//!
//! let walker = WalkBuilder::new(&root)
//!     .git_ignore(true)
//!     .ignore(true)
//!     .build();
//!
//! for entry in walker.flatten() {
//!     insert_entry(&mut root_map, &root, &entry, true);
//! }
//!
//! render_tree(&root, &root_map, false, "blue", true);
//! ```

mod term;
mod tree;

pub use crate::{
    term::{ASCII_STYLE, TreeStyle, UNICODE_STYLE, human_bytes, render_tree},
    tree::{Node, insert_entry},
};
