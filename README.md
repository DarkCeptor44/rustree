# rustree

A Rust implementation of the `tree` command-line utility. It's not blazingly fast (yet), but at least it respects `.gitignore` files by default.

## Installation

For the CLI:

```bash
cargo install rustree
```

For the library:

```toml
[dependencies]
rustree = { version = "0.1.0", default-features = false }
```

## Usage Example

```rust
use rustree::{Node, Walker, insert_entry, render_tree};
use std::{collections::BTreeMap, path::Path};

let root = Path::new(".");
let entries = Walker::new(root).threshold(100).all(false).walk();

// if you want to use ignore, this crate already provides
// a From<ignore::DirEntry> for DirEntry.
// if you want to use another crate for directory traversal,
// it's pretty easy to convert a DirEntry to a rustree::DirEntry,
// all you need is a PathBuf.

let mut root_map: BTreeMap<String, Node> = BTreeMap::new();
for entry in entries {
    insert_entry(&mut root_map, root, &entry, true);
}

render_tree(root, &root_map, false, "blue", true);
```

## Todo

- [ ] Make it faster
- [ ] Depth limiting
- [ ] Directories-only mode

## Benchmarks

- **Command:**:

    ```bash
    hyperfine --warmup 10 "tree" --command-name "Windows tree" "rustree -a" --command-name "rustree (all)" "rustree -l" --command-name "rustree (with sizes)" "rustree" -S --export-markdown=windows.md
    ```

### Rust Codebase

- **2.6M lines of code**

| Command | Mean Wall Time [ms] | Change | Factor |
| --- | --- | --- | --- |
| `Windows tree` | 428.7 ± 4.5 | | |
| `rustree (all)` | 1096.3 ± 5.9 | +155.7% | (2.6x slower) |
| `rustree` | 1645.7 ± 7.1 | +283.9% | (3.8x slower) |
| `rustree (with sizes)` | 1675.7 ± 5.6 | +290.9% | (3.9x slower) |

### Rustree Codebase

- **170 lines of code**

| Command | Mean Wall Time [ms] | Change | Factor |
| --- | --- | --- | --- |
| `Windows tree` | 14.0 ± 1.8 | | |
| `rustree (with sizes)` | 17.8 ± 1.3 | +27.4% | |
| `rustree (all)` | 18.4 ± 1.9 | +31.5% | (1.3x slower) |
| `rustree` | 18.5 ± 2.2 | +32.1% | (1.3x slower) |

### Library Benchmarks

```text
Timer precision: 100 ns
fs               fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ Walker::walk                │               │               │               │         │
   ├─ 100        9.086 ms      │ 13.09 ms      │ 9.794 ms      │ 10.01 ms      │ 100     │ 100
   │             11 Kitem/s    │ 7.637 Kitem/s │ 10.2 Kitem/s  │ 9.983 Kitem/s │         │
   ├─ 500        45.89 ms      │ 63.79 ms      │ 47.85 ms      │ 48.48 ms      │ 100     │ 100
   │             10.89 Kitem/s │ 7.838 Kitem/s │ 10.44 Kitem/s │ 10.31 Kitem/s │         │
   ├─ 1000       84.57 ms      │ 116.2 ms      │ 87.87 ms      │ 89.13 ms      │ 100     │ 100
   │             11.82 Kitem/s │ 8.605 Kitem/s │ 11.37 Kitem/s │ 11.21 Kitem/s │         │
   ╰─ 10000      233.8 ms      │ 313.1 ms      │ 246 ms        │ 252.4 ms      │ 100     │ 100
                 42.75 Kitem/s │ 31.93 Kitem/s │ 40.64 Kitem/s │ 39.6 Kitem/s  │         │
```
