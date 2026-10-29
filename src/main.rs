// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_debug_implementations)]

use clap::Parser;
use colored::Colorize;
use rustree::{Error, Node, Walker, insert_entry, render_tree};
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

    #[arg(
        short,
        long,
        help = "Threshold at for parallelizing",
        default_value_t = 100
    )]
    threshold: usize,
}

fn main() {
    if let Err(e) = main_impl() {
        eprintln!("{}", format!("{NAME}: {e:?}").red());
        exit(1);
    }
}

fn main_impl() -> Result<(), Error> {
    let args = App::parse();
    let App {
        path: root_path,
        all,
        ascii,
        color,
        show_sizes,
        threshold,
    } = args;

    if !root_path.is_dir() {
        return Err(Error::NotDir(root_path));
    }

    let mut root_map: BTreeMap<String, Node> = BTreeMap::new();
    let entries = Walker::new(&root_path).threshold(threshold).all(all).walk();

    for entry in entries {
        insert_entry(&mut root_map, &root_path, &entry, show_sizes);
    }

    render_tree(&root_path, &root_map, ascii, &color, show_sizes);
    Ok(())
}
