#![forbid(unsafe_code)]
// #![warn(clippy::pedantic, missing_debug_implementations)]

mod tree;

use crate::tree::{Node, insert_entry};
use anyhow::{Result, anyhow};
use clap::Parser;
use colored::Colorize;
use crossbeam::channel::bounded;
use ignore::{WalkBuilder, WalkState};
use std::{collections::BTreeMap, path::PathBuf, process::exit};

const NAME: &str = env!("CARGO_BIN_NAME");

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct App {
    #[arg(help = "Root path", default_value = ".")]
    path: PathBuf,
}

fn main() {
    if let Err(e) = main_impl() {
        eprintln!("{}", format!("{NAME}: {e:?}").red());
        exit(1);
    }
}

fn main_impl() -> Result<()> {
    let args = App::parse();
    let App { path: root_path } = args;

    if !root_path.is_dir() {
        return Err(anyhow!("path is not a directory: {}", root_path.display()));
    }

    let walker = WalkBuilder::new(&root_path)
        .git_ignore(true)
        .ignore(true)
        .build_parallel();
    let (tx, rx) = bounded(100);

    walker.run(|| {
        let tx = tx.clone();
        Box::new(move |result| {
            if let Ok(entry) = result {
                let _ = tx.send(entry);
            }
            WalkState::Continue
        })
    });

    drop(tx);

    let mut root_map: BTreeMap<String, Node> = BTreeMap::new();
    for entry in rx {
        insert_entry(&mut root_map, &root_path, &entry);
    }

    for name in root_map.keys() {
        println!("{name}");
    }

    Ok(())
}
