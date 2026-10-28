#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_debug_implementations)]

mod term;
mod tree;

use crate::{
    term::render_tree,
    tree::{Node, insert_entry},
};
use anyhow::{Result, anyhow};
use clap::Parser;
use colored::Colorize;
use ignore::WalkBuilder;
use std::{collections::BTreeMap, path::PathBuf, process::exit};

const NAME: &str = env!("CARGO_BIN_NAME");

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct App {
    #[arg(help = "Root path", default_value = ".")]
    path: PathBuf,

    #[arg(short, long, help = "Show all entries", default_value_t)]
    all: bool,

    #[arg(short = 'A', long, help = "Use ASCII characters", default_value_t)]
    ascii: bool,

    #[arg(
        short,
        long,
        help = "Color to use for directory names (`off` and hexcode like `#ff4500` both work as well)",
        default_value = "blue"
    )]
    color: String,

    #[arg(short = 'l', long, help = "Show file sizes", default_value_t)]
    show_sizes: bool,
}

fn main() {
    if let Err(e) = main_impl() {
        eprintln!("{}", format!("{NAME}: {e:?}").red());
        exit(1);
    }
}

fn main_impl() -> Result<()> {
    let args = App::parse();
    let App {
        path: root_path,
        all,
        ascii,
        color,
        show_sizes,
    } = args;

    if !root_path.is_dir() {
        return Err(anyhow!("path is not a directory: {}", root_path.display()));
    }

    let mut root_map: BTreeMap<String, Node> = BTreeMap::new();
    let walker = WalkBuilder::new(&root_path)
        .git_ignore(!all)
        .ignore(!all)
        .build();

    for entry in walker.flatten() {
        insert_entry(&mut root_map, &root_path, &entry);
    }

    render_tree(&root_path, &root_map, ascii, &color, show_sizes);
    Ok(())
}
