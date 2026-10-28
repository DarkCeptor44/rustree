# rustree

A Rust implementation of the `tree` command-line utility. It's not blazingly fast (yet), but at least it respects `.gitignore` files by default.

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
