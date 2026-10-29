// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use divan::{Bencher, black_box, counter::ItemsCount};
use rustree::Walker;
use std::fs::{create_dir, write};
use tempfile::TempDir;

const M: usize = 10;
const N: &[usize] = &[100, 500, 1000, 10000];

fn main() {
    divan::main();
}

#[divan::bench(name = "Walker::walk", args = N)]
fn bench_walker_walk(b: Bencher, total_files: usize) {
    let dir = setup_temp_tree(M, total_files);
    let walker = Walker::new(dir.path());

    b.counter(ItemsCount::new(total_files))
        .bench(|| black_box(walker.walk()));
}

fn setup_temp_tree(max_depth: usize, total_files: usize) -> TempDir {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let mut files_created = 0;
    let mut queue = vec![(temp_dir.path().to_path_buf(), 0)];

    while files_created < total_files && !queue.is_empty() {
        let (current_path, depth) = queue.remove(0);

        let remaining = total_files - files_created;
        let files_in_dir = std::cmp::min(remaining, 10);

        for _ in 0..files_in_dir {
            let file_path = current_path.join(format!("file_{files_created}.txt"));
            write(&file_path, b"rustree benchmark payload").unwrap();
            files_created += 1;
            if files_created >= total_files {
                break;
            }
        }

        if depth < max_depth && files_created < total_files {
            for i in 0..3 {
                let sub_dir = current_path.join(format!("dir_{i}"));
                if create_dir(&sub_dir).is_ok() {
                    queue.push((sub_dir, depth + 1));
                }
            }
        }
    }

    temp_dir
}
