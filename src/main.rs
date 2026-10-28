#![forbid(unsafe_code)]
// #![warn(clippy::pedantic, missing_debug_implementations)]

mod tree;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::process::exit;

const NAME: &str = env!("CARGO_BIN_NAME");

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct App {}

fn main() {
    if let Err(e) = main_impl() {
        eprintln!("{}", format!("{NAME}: {e:?}").red());
        exit(1);
    }
}

fn main_impl() -> Result<()> {
    let args = App::parse();

    Ok(())
}
