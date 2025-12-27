# rloc

A blazingly fast line-of-code counter written in Rust. Full-featured [cloc](https://github.com/AlDanial/cloc) replacement.

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

# Compare two directories
rloc old_version/ --diff new_version/
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

- **Fast**: Parallel processing with [rayon](https://github.com/rayon-rs/rayon). Typically 100-125x faster than cloc.
- **Accurate**: Language-aware comment parsing handles nested comments, string literals, and edge cases.
- **Full-featured**: Diff mode, archive support, custom languages, duplicate detection - everything cloc has.
- **Modern defaults**: Respects `.gitignore` and excludes common non-source directories by default.
- **Multiple output formats**: Table, JSON, CSV, YAML, Markdown, SQL, and XML.
- **Single binary**: No runtime dependencies. Easy to install and distribute.

## Performance

Benchmark on a large mixed-language monorepo (~40k files, ~6M lines of code):

| Tool | Time | Files/s | Lines/s | Speedup |
|------|------|---------|---------|---------|
| **rloc** | **3.2s** | 15,333 | 2,498,635 | **125x** |
| cloc | 398s | 97 | 18,427 | 1x |

rloc achieves this through parallel file processing and efficient I/O.

## Output Formats

```bash
rloc --format table   # Default: Unicode table
rloc --format json    # JSON (compatible with cloc)
rloc --format csv     # CSV
rloc --format yaml    # YAML
rloc --format md      # Markdown table
rloc --format sql     # SQL CREATE/INSERT statements
rloc --format xml     # XML
```

Or use shorthand flags: `--json`, `--csv`, `--yaml`, `--md`, `--sql`, `--xml`

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

### By file content

```bash
rloc --include-content 'TODO|FIXME'   # Only files containing pattern
rloc --exclude-content 'generated'     # Exclude files containing pattern
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
rloc --no-recurse                  # Only process top-level directory
```

### By file size

```bash
rloc --max-file-size 10            # Skip files larger than 10 MB
```

## Advanced Features

### Diff Mode

Compare two sets of files to see what changed:

```bash
rloc old_version/ --diff new_version/
```

Output shows lines of code that are same, modified, added, or removed:

```
Language           Same   Modified      Added    Removed
──────────────────────────────────────────────────────────
Rust               1500        200        150         50
JavaScript          800         50         25         10
──────────────────────────────────────────────────────────
SUM                2300        250        175         60
```

### Archive Support

Process files inside archives without extracting manually:

```bash
rloc project.zip --extract-archives
rloc release.tar.gz --extract-archives
```

Supports: `.zip`, `.tar`, `.tar.gz`, `.tgz`

### Custom Language Definitions

Define new languages or override built-in definitions with YAML:

```yaml
# custom_langs.yaml
MyLang:
  extensions: [mylang, ml]
  line_comments: ["#", "//"]
  block_comment_start: "/*"
  block_comment_end: "*/"
  nested_comments: false

DSL:
  extensions: [dsl]
  line_comments: ["--"]
```

```bash
rloc --read-lang-def custom_langs.yaml
```

### Force Language Detection

Treat files with specific extensions as a different language:

```bash
rloc --force-lang=Rust,txt    # Treat .txt files as Rust
rloc --force-lang=Python,inc  # Treat .inc files as Python
```

### Duplicate Detection

By default, rloc detects and skips duplicate files (by content hash):

```bash
rloc                          # Skips duplicates (default)
rloc --skip-uniqueness        # Count duplicates multiple times
```

### Strip Comments

Extract code or comments from source files:

```bash
rloc --strip-comments=stripped src/    # Output: *.stripped (code only)
rloc --strip-code=comments src/        # Output: *.comments (comments only)
```

### Read File List

Process a predefined list of files:

```bash
rloc --list-file files.txt    # One file path per line
```

### Combine Reports

Merge multiple JSON reports:

```bash
rloc project1/ --json > report1.json
rloc project2/ --json > report2.json
rloc --sum-reports report1.json --sum-reports report2.json
```

### Aggregate Small Results

Group languages with few files into "Other":

```bash
rloc --summary-cutoff 5       # Languages with <5 files become "Other"
```

### Percentage Output

Show percentages instead of absolute counts:

```bash
rloc --by-percent
```

## Git Integration

```bash
rloc --vcs git                # Use git ls-files for file discovery
rloc --include-submodules     # Include files in git submodules
```

## Options Reference

| Option | Description |
|--------|-------------|
| `--by-file` | Report results for every source file |
| `--by-file-by-lang` | Group per-file results by language |
| `--format <FMT>` | Output format: table, json, csv, yaml, md, sql, xml |
| `--sort <FIELD>` | Sort by: language, files, code, comments, blanks, total |
| `--diff <PATH>` | Compare against another directory |
| `--exclude-dir <DIR>` | Exclude directories by name |
| `--exclude-ext <EXT>` | Exclude files by extension |
| `--exclude-lang <LANG>` | Exclude languages |
| `--include-ext <EXT>` | Only count files with these extensions |
| `--include-lang <LANG>` | Only count these languages |
| `--include-content <RE>` | Only count files matching content regex |
| `--exclude-content <RE>` | Exclude files matching content regex |
| `--match-f <REGEX>` | Only count files matching regex |
| `--not-match-f <REGEX>` | Exclude files matching regex |
| `--match-d <REGEX>` | Only count in directories matching regex |
| `--not-match-d <REGEX>` | Exclude directories matching regex |
| `--fullpath` | Use full path for regex matching |
| `--force-lang <L,E>` | Treat extension E as language L |
| `--read-lang-def <FILE>` | Load custom language definitions |
| `--list-file <FILE>` | Read file paths from file |
| `--extract-archives` | Process zip/tar/tar.gz files |
| `--follow-symlinks` | Follow symbolic links |
| `--hidden` | Include hidden files and directories |
| `--no-ignore` | Disable default directory exclusions |
| `--skip-gitignore` | Don't respect .gitignore files |
| `--skip-uniqueness` | Don't skip duplicate files |
| `--max-depth <N>` | Maximum directory depth |
| `--no-recurse` | Don't recurse into subdirectories |
| `--max-file-size <MB>` | Skip files larger than N megabytes |
| `--csv-delimiter <C>` | Custom CSV delimiter |
| `--summary-cutoff <N>` | Aggregate languages with <N files |
| `--sum-reports <FILE>` | Combine JSON report files |
| `--strip-comments <EXT>` | Write code-only files with extension |
| `--strip-code <EXT>` | Write comment-only files with extension |
| `--show-total` | Add column with total lines |
| `--by-percent` | Show percentages instead of counts |
| `--hide-rate` | Don't show processing rate |
| `--quiet` | Suppress progress output |
| `--out <FILE>` | Write output to file |
| `--threads <N>` | Number of threads (0 = auto) |
| `--show-lang` | List all supported languages |
| `--show-ext` | List all recognized file extensions |

## Supported Languages

rloc recognizes 100+ programming languages including:

Ada, Aria, Assembly, AXAML, Bash, Bicep, BitBake, C, C#, C++, Clarity, Clojure, COBOL, CoffeeScript, Crystal, CSS, D, Dart, Elixir, Elm, Erlang, F#, Forth, Fortran, Go, GraphQL, Groovy, Haskell, HCL, HTML, Java, JavaScript, JSON, Julia, Kotlin, Less, Lisp, Lua, Magik, Makefile, Markdown, MATLAB, Nim, Nix, Objective-C, OCaml, Odin, Pascal, Perl, PHP, PowerShell, Prolog, Protocol Buffers, Python, R, Rego, Ruby, Rust, Sass, Scala, Scheme, Shell, SQL, Swift, Tcl, Terraform, TOML, TypeScript, USS, UXML, V, Verilog, VHDL, Vim Script, VSCode Workspace, Vue, XML, YAML, Yarn, Zig, and many more.

Run `rloc --show-lang` for the complete list with comment syntax.

## Default Exclusions

By default, rloc excludes common non-source directories:

`.git`, `.hg`, `.svn`, `node_modules`, `target`, `vendor`, `dist`, `build`, `__pycache__`, `.venv`, `venv`, `.tox`, `.env`

Use `--no-ignore` to disable these exclusions.

## Feature Comparison with cloc

| Feature | rloc | cloc |
|---------|------|------|
| Speed | ~125x faster | Baseline |
| Parallel processing | Yes | No |
| Respects .gitignore | Yes (default) | No |
| Default exclusions | Yes | No |
| Duplicate detection | Yes | Yes |
| Archive support | Yes | Yes |
| Diff mode | Yes | Yes |
| Custom language defs | Yes | Yes |
| Strip comments/code | Yes | Yes |
| Content filtering | Yes | No |
| Git submodules | Yes | Yes |
| Sum reports | Yes | Yes |

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

## Acknowledgments

- [cloc](https://github.com/AlDanial/cloc) by Al Danial - the original inspiration
- [tokei](https://github.com/XAMPPRocky/tokei) - another excellent Rust code counter
- [ripgrep](https://github.com/BurntSushi/ripgrep) - for the `ignore` crate
