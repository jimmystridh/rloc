# rloc

A blazingly fast line-of-code counter written in Rust. Inspired by [cloc](https://github.com/AlDanial/cloc).

## Quick Start

```bash
# Count lines in current directory
rloc

# Count lines in specific paths
rloc src/ lib/ tests/

# Output as JSON
rloc --json

# Show per-file breakdown
rloc --by-file

# Include all files (disable .gitignore and default exclusions)
rloc --no-ignore --skip-gitignore
```

## Example Output

```
$ rloc src/
7 files processed in 0.002s (3241 files/s, 1282457 lines/s)

┌──────────┬───────┬───────┬─────────┬──────┐
│ Language ┆ Files ┆ Blank ┆ Comment ┆ Code │
╞══════════╪═══════╪═══════╪═════════╪══════╡
│ Rust     ┆ 7     ┆ 334   ┆ 91      ┆ 2345 │
│ SUM      ┆ 7     ┆ 334   ┆ 91      ┆ 2345 │
└──────────┴───────┴───────┴─────────┴──────┘
```

## Installation

### From source

```bash
git clone https://github.com/your-username/rloc
cd rloc
cargo build --release
# Binary is at ./target/release/rloc
```

### Cargo

```bash
cargo install rloc
```

## Why rloc?

- **Fast**: Parallel processing with [rayon](https://github.com/rayon-rs/rayon). Typically 50-100x faster than cloc.
- **Accurate**: Language-aware comment parsing handles nested comments, string literals, and edge cases.
- **Modern defaults**: Respects `.gitignore` and excludes common non-source directories (`node_modules`, `target`, `vendor`, etc.) by default.
- **Multiple output formats**: Table, JSON, CSV, YAML, and Markdown.
- **Single binary**: No runtime dependencies. Easy to install and distribute.
- **130+ languages**: Comprehensive language support with accurate comment syntax definitions.

## Performance

Benchmark on a large mixed-language monorepo (~40k files, ~6M lines of code):

| Tool | Time | Files/s | Lines/s | Speedup |
|------|------|---------|---------|---------|
| **rloc** | **3.2s** | 15,333 | 2,498,635 | **125x** |
| cloc | 398s | 97 | 18,427 | 1x |

rloc achieves this through parallel file processing with [rayon](https://github.com/rayon-rs/rayon) and efficient memory-mapped I/O.

Note: cloc performs MD5-based duplicate file detection by default, which adds overhead but deduplicates identical files. Use `cloc --skip-uniqueness` for a more direct comparison.

## Output Formats

```bash
rloc --format table   # Default: Unicode table
rloc --format json    # JSON (compatible with cloc JSON output)
rloc --format csv     # CSV
rloc --format yaml    # YAML
rloc --format md      # Markdown table
```

Or use shorthand flags: `--json`, `--csv`, `--yaml`, `--md`

## Filtering

### By language

```bash
rloc --include-lang Rust,Python    # Only count these languages
rloc --exclude-lang JavaScript     # Exclude these languages
```

### By extension

```bash
rloc --include-ext rs,py           # Only these extensions
rloc --exclude-ext min.js,bundle.js
```

### By path pattern

```bash
rloc --match-f '\.test\.'          # Files matching regex
rloc --not-match-f '_test\.go$'    # Files not matching regex
rloc --match-d 'src|lib'           # Directories matching regex
rloc --not-match-d 'vendor|third_party'
```

### By directory

```bash
rloc --exclude-dir vendor,generated
rloc --max-depth 3                 # Limit directory traversal depth
```

## Options Reference

| Option | Description |
|--------|-------------|
| `--by-file` | Report results for every source file |
| `--by-file-by-lang` | Group per-file results by language |
| `--format <FORMAT>` | Output format: table, json, csv, yaml, md |
| `--sort <FIELD>` | Sort by: language, files, code, comments, blanks, total |
| `--exclude-dir <DIR>` | Exclude directories by name |
| `--exclude-ext <EXT>` | Exclude files by extension |
| `--exclude-lang <LANG>` | Exclude languages |
| `--include-ext <EXT>` | Only count files with these extensions |
| `--include-lang <LANG>` | Only count these languages |
| `--match-f <REGEX>` | Only count files matching regex |
| `--not-match-f <REGEX>` | Exclude files matching regex |
| `--match-d <REGEX>` | Only count in directories matching regex |
| `--not-match-d <REGEX>` | Exclude directories matching regex |
| `--fullpath` | Use full path for regex matching |
| `--follow-symlinks` | Follow symbolic links |
| `--hidden` | Include hidden files and directories |
| `--no-ignore` | Disable default directory exclusions |
| `--skip-gitignore` | Don't respect .gitignore files |
| `--max-depth <N>` | Maximum directory depth |
| `--show-total` | Add column with total lines (blank + comment + code) |
| `--hide-rate` | Don't show processing rate |
| `--quiet` | Suppress progress output |
| `--out <FILE>` | Write output to file |
| `--threads <N>` | Number of threads (0 = auto) |
| `--show-lang` | List all supported languages |
| `--show-ext` | List all recognized file extensions |

## Supported Languages

rloc recognizes 130+ programming languages including:

Ada, Assembly, Bash, C, C#, C++, Clojure, COBOL, CoffeeScript, Crystal, CSS, D, Dart, Elixir, Elm, Erlang, F#, Forth, Fortran, Go, GraphQL, Groovy, Haskell, HCL, HTML, Java, JavaScript, JSON, Julia, Kotlin, Less, Lisp, Lua, Makefile, Markdown, MATLAB, Nim, Nix, Objective-C, OCaml, Odin, Pascal, Perl, PHP, PowerShell, Prolog, Protocol Buffers, Python, R, Ruby, Rust, Sass, Scala, Scheme, Shell, SQL, Swift, Tcl, Terraform, TOML, TypeScript, V, Verilog, VHDL, Vim Script, Vue, XML, YAML, Zig, and many more.

Run `rloc --show-lang` for the complete list.

## Default Exclusions

By default, rloc excludes common non-source directories:

`.git`, `.hg`, `.svn`, `node_modules`, `target`, `vendor`, `dist`, `build`, `__pycache__`, `.venv`, `venv`, `.tox`, `.mypy_cache`, `.pytest_cache`, `.cache`, `coverage`, `.coverage`, `.nyc_output`, `bower_components`, `jspm_packages`, `.next`, `.nuxt`, `.output`, `.gradle`, `.idea`, `.vs`, `.vscode`, `Pods`, `DerivedData`

Use `--no-ignore` to disable these exclusions.

## Differences from cloc

| Feature | rloc | cloc |
|---------|------|------|
| Speed | ~100x faster | Baseline |
| Respects .gitignore | Yes (default) | No |
| Default exclusions | Yes | No |
| Duplicate detection | No | Yes |
| Archive support | No | Yes |
| Diff mode | No | Yes |
| Custom language defs | No | Yes |

rloc is designed for fast, everyday use. For advanced features like diff mode or archive analysis, use cloc.

## How It Works

1. **Walk**: Traverse directories using the [ignore](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore) crate, respecting `.gitignore` patterns.
2. **Detect**: Identify language by file extension or special filenames (e.g., `Makefile`, `Dockerfile`).
3. **Parse**: Process each file with a state machine that tracks:
   - Whether we're inside a string literal
   - Whether we're inside a block comment (including nesting for languages that support it)
   - Line comment prefixes
4. **Classify**: Each line is classified as blank, comment, or code.
5. **Aggregate**: Results are collected in parallel and merged by language.

## License

MIT License. See [LICENSE](LICENSE) for details.

This is a clean-room reimplementation inspired by cloc. No code from cloc was used.

## Acknowledgments

- [cloc](https://github.com/AlDanial/cloc) by Al Danial - the original inspiration
- [tokei](https://github.com/XAMPPRocky/tokei) - another excellent Rust code counter
- [ripgrep](https://github.com/BurntSushi/ripgrep) - for the `ignore` crate
