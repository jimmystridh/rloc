use phf::phf_map;
use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommentStyle {
    C,
    CShell,
    Shell,
    Html,
    Haskell,
    Fortran,
    Pascal,
    Ada,
    Sql,
    Lua,
    Lisp,
    Erlang,
    Prolog,
    Matlab,
    Ruby,
    Perl,
    Python,
    R,
    Vim,
    Assembly,
    Batch,
    Ini,
    Tex,
    Forth,
    Clojure,
    Julia,
    Nim,
    Crystal,
    Elm,
    Zig,
    Odin,
    V,
    Cobol,
    Ocaml,
    Fsharp,
    None,
}

#[derive(Debug, Clone)]
pub struct Language {
    pub name: &'static str,
    pub line_comments: &'static [&'static str],
    pub block_comment_start: Option<&'static str>,
    pub block_comment_end: Option<&'static str>,
    pub nested_comments: bool,
    pub string_delimiters: &'static [&'static str],
    #[allow(dead_code)]
    pub raw_string_start: Option<&'static str>,
    #[allow(dead_code)]
    pub raw_string_end: Option<&'static str>,
}

impl Language {
    const fn new(name: &'static str) -> Self {
        Self {
            name,
            line_comments: &[],
            block_comment_start: None,
            block_comment_end: None,
            nested_comments: false,
            string_delimiters: &["\"", "'"],
            raw_string_start: None,
            raw_string_end: None,
        }
    }

    const fn c_style(name: &'static str) -> Self {
        Self {
            name,
            line_comments: &["//"],
            block_comment_start: Some("/*"),
            block_comment_end: Some("*/"),
            nested_comments: false,
            string_delimiters: &["\"", "'"],
            raw_string_start: None,
            raw_string_end: None,
        }
    }

    const fn shell_style(name: &'static str) -> Self {
        Self {
            name,
            line_comments: &["#"],
            block_comment_start: None,
            block_comment_end: None,
            nested_comments: false,
            string_delimiters: &["\"", "'"],
            raw_string_start: None,
            raw_string_end: None,
        }
    }

    const fn html_style(name: &'static str) -> Self {
        Self {
            name,
            line_comments: &[],
            block_comment_start: Some("<!--"),
            block_comment_end: Some("-->"),
            nested_comments: false,
            string_delimiters: &["\"", "'"],
            raw_string_start: None,
            raw_string_end: None,
        }
    }

    const fn with_line_comments(mut self, comments: &'static [&'static str]) -> Self {
        self.line_comments = comments;
        self
    }

    const fn with_block_comments(mut self, start: &'static str, end: &'static str) -> Self {
        self.block_comment_start = Some(start);
        self.block_comment_end = Some(end);
        self
    }

    const fn with_nested_comments(mut self) -> Self {
        self.nested_comments = true;
        self
    }

    #[allow(dead_code)]
    const fn with_string_delimiters(mut self, delims: &'static [&'static str]) -> Self {
        self.string_delimiters = delims;
        self
    }
}

pub static LANGUAGES: phf::Map<&'static str, Language> = phf_map! {
    // Systems Programming
    "Rust" => Language::c_style("Rust").with_nested_comments(),
    "C" => Language::c_style("C"),
    "C Header" => Language::c_style("C Header"),
    "C++" => Language::c_style("C++"),
    "C++ Header" => Language::c_style("C++ Header"),
    "Objective-C" => Language::c_style("Objective-C"),
    "Objective-C++" => Language::c_style("Objective-C++"),
    "D" => Language::c_style("D").with_nested_comments(),
    "Zig" => Language {
        name: "Zig",
        line_comments: &["//"],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Odin" => Language::c_style("Odin").with_nested_comments(),
    "V" => Language::c_style("V"),
    "Nim" => Language {
        name: "Nim",
        line_comments: &["#"],
        block_comment_start: Some("#["),
        block_comment_end: Some("]#"),
        nested_comments: true,
        string_delimiters: &["\"", "'"],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Crystal" => Language {
        name: "Crystal",
        line_comments: &["#"],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &["\"", "'"],
        raw_string_start: None,
        raw_string_end: None,
    },

    // JVM Languages
    "Java" => Language::c_style("Java"),
    "Kotlin" => Language::c_style("Kotlin").with_nested_comments(),
    "Scala" => Language::c_style("Scala").with_nested_comments(),
    "Groovy" => Language::c_style("Groovy"),
    "Clojure" => Language {
        name: "Clojure",
        line_comments: &[";"],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },

    // .NET Languages
    "C#" => Language::c_style("C#"),
    "F#" => Language {
        name: "F#",
        line_comments: &["//"],
        block_comment_start: Some("(*"),
        block_comment_end: Some("*)"),
        nested_comments: true,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Visual Basic" => Language {
        name: "Visual Basic",
        line_comments: &["'"],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },

    // Web Languages
    "JavaScript" => Language::c_style("JavaScript"),
    "TypeScript" => Language::c_style("TypeScript"),
    "JSX" => Language::c_style("JSX"),
    "TSX" => Language::c_style("TSX"),
    "CoffeeScript" => Language::shell_style("CoffeeScript").with_block_comments("###", "###"),
    "HTML" => Language::html_style("HTML"),
    "CSS" => Language {
        name: "CSS",
        line_comments: &[],
        block_comment_start: Some("/*"),
        block_comment_end: Some("*/"),
        nested_comments: false,
        string_delimiters: &["\"", "'"],
        raw_string_start: None,
        raw_string_end: None,
    },
    "SCSS" => Language::c_style("SCSS"),
    "Sass" => Language::c_style("Sass"),
    "Less" => Language::c_style("Less"),
    "Vue" => Language::html_style("Vue"),
    "Svelte" => Language::html_style("Svelte"),

    // Scripting Languages
    "Python" => Language {
        name: "Python",
        line_comments: &["#"],
        block_comment_start: Some("\"\"\""),
        block_comment_end: Some("\"\"\""),
        nested_comments: false,
        string_delimiters: &["\"", "'"],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Ruby" => Language {
        name: "Ruby",
        line_comments: &["#"],
        block_comment_start: Some("=begin"),
        block_comment_end: Some("=end"),
        nested_comments: false,
        string_delimiters: &["\"", "'"],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Perl" => Language::shell_style("Perl").with_block_comments("=pod", "=cut"),
    "PHP" => Language::c_style("PHP").with_line_comments(&["//", "#"]),
    "Lua" => Language {
        name: "Lua",
        line_comments: &["--"],
        block_comment_start: Some("--[["),
        block_comment_end: Some("]]"),
        nested_comments: false,
        string_delimiters: &["\"", "'"],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Tcl" => Language::shell_style("Tcl"),
    "Awk" => Language::shell_style("Awk"),

    // Shell Languages
    "Shell" => Language::shell_style("Shell"),
    "Bash" => Language::shell_style("Bash"),
    "Zsh" => Language::shell_style("Zsh"),
    "Fish" => Language::shell_style("Fish"),
    "PowerShell" => Language {
        name: "PowerShell",
        line_comments: &["#"],
        block_comment_start: Some("<#"),
        block_comment_end: Some("#>"),
        nested_comments: false,
        string_delimiters: &["\"", "'"],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Batch" => Language {
        name: "Batch",
        line_comments: &["REM", "rem", "::"],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },

    // Functional Languages
    "Haskell" => Language {
        name: "Haskell",
        line_comments: &["--"],
        block_comment_start: Some("{-"),
        block_comment_end: Some("-}"),
        nested_comments: true,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },
    "OCaml" => Language {
        name: "OCaml",
        line_comments: &[],
        block_comment_start: Some("(*"),
        block_comment_end: Some("*)"),
        nested_comments: true,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Standard ML" => Language {
        name: "Standard ML",
        line_comments: &[],
        block_comment_start: Some("(*"),
        block_comment_end: Some("*)"),
        nested_comments: true,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Elm" => Language {
        name: "Elm",
        line_comments: &["--"],
        block_comment_start: Some("{-"),
        block_comment_end: Some("-}"),
        nested_comments: true,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Erlang" => Language {
        name: "Erlang",
        line_comments: &["%"],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Elixir" => Language::shell_style("Elixir").with_block_comments("@doc \"\"\"", "\"\"\""),
    "Lisp" => Language {
        name: "Lisp",
        line_comments: &[";"],
        block_comment_start: Some("#|"),
        block_comment_end: Some("|#"),
        nested_comments: true,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Scheme" => Language {
        name: "Scheme",
        line_comments: &[";"],
        block_comment_start: Some("#|"),
        block_comment_end: Some("|#"),
        nested_comments: true,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Racket" => Language {
        name: "Racket",
        line_comments: &[";"],
        block_comment_start: Some("#|"),
        block_comment_end: Some("|#"),
        nested_comments: true,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },

    // Go and friends
    "Go" => Language::c_style("Go"),

    // Swift and Apple ecosystem
    "Swift" => Language::c_style("Swift").with_nested_comments(),

    // Data/Config Languages
    "JSON" => Language::new("JSON"),
    "JSON5" => Language::c_style("JSON5"),
    "YAML" => Language::shell_style("YAML"),
    "TOML" => Language::shell_style("TOML"),
    "XML" => Language::html_style("XML"),
    "INI" => Language {
        name: "INI",
        line_comments: &[";", "#"],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &["\"", "'"],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Properties" => Language {
        name: "Properties",
        line_comments: &["#", "!"],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &[],
        raw_string_start: None,
        raw_string_end: None,
    },

    // Query Languages
    "SQL" => Language {
        name: "SQL",
        line_comments: &["--"],
        block_comment_start: Some("/*"),
        block_comment_end: Some("*/"),
        nested_comments: false,
        string_delimiters: &["'"],
        raw_string_start: None,
        raw_string_end: None,
    },
    "GraphQL" => Language::shell_style("GraphQL"),

    // Build/Config
    "Makefile" => Language::shell_style("Makefile"),
    "CMake" => Language::shell_style("CMake"),
    "Meson" => Language::shell_style("Meson"),
    "Dockerfile" => Language::shell_style("Dockerfile"),
    "Docker Compose" => Language::shell_style("Docker Compose"),
    "Terraform" => Language::c_style("Terraform").with_line_comments(&["//", "#"]),
    "HCL" => Language::c_style("HCL").with_line_comments(&["//", "#"]),
    "Nix" => Language::shell_style("Nix").with_block_comments("/*", "*/"),
    "Bazel" => Language::shell_style("Bazel"),
    "Just" => Language::shell_style("Just"),

    // Documentation
    "Markdown" => Language::html_style("Markdown"),
    "reStructuredText" => Language {
        name: "reStructuredText",
        line_comments: &[".."],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &[],
        raw_string_start: None,
        raw_string_end: None,
    },
    "AsciiDoc" => Language {
        name: "AsciiDoc",
        line_comments: &["//"],
        block_comment_start: Some("////"),
        block_comment_end: Some("////"),
        nested_comments: false,
        string_delimiters: &[],
        raw_string_start: None,
        raw_string_end: None,
    },
    "LaTeX" => Language {
        name: "LaTeX",
        line_comments: &["%"],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &[],
        raw_string_start: None,
        raw_string_end: None,
    },
    "TeX" => Language {
        name: "TeX",
        line_comments: &["%"],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &[],
        raw_string_start: None,
        raw_string_end: None,
    },

    // Assembly
    "Assembly" => Language {
        name: "Assembly",
        line_comments: &[";", "#", "//"],
        block_comment_start: Some("/*"),
        block_comment_end: Some("*/"),
        nested_comments: false,
        string_delimiters: &["\"", "'"],
        raw_string_start: None,
        raw_string_end: None,
    },
    "ARM Assembly" => Language {
        name: "ARM Assembly",
        line_comments: &[";", "@", "//"],
        block_comment_start: Some("/*"),
        block_comment_end: Some("*/"),
        nested_comments: false,
        string_delimiters: &["\"", "'"],
        raw_string_start: None,
        raw_string_end: None,
    },

    // Hardware Description
    "Verilog" => Language::c_style("Verilog"),
    "SystemVerilog" => Language::c_style("SystemVerilog"),
    "VHDL" => Language {
        name: "VHDL",
        line_comments: &["--"],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },

    // Scientific/Math
    "R" => Language::shell_style("R"),
    "Julia" => Language::shell_style("Julia").with_block_comments("#=", "=#"),
    "MATLAB" => Language {
        name: "MATLAB",
        line_comments: &["%"],
        block_comment_start: Some("%{"),
        block_comment_end: Some("%}"),
        nested_comments: false,
        string_delimiters: &["'", "\""],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Octave" => Language {
        name: "Octave",
        line_comments: &["%", "#"],
        block_comment_start: Some("%{"),
        block_comment_end: Some("%}"),
        nested_comments: false,
        string_delimiters: &["'", "\""],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Fortran" => Language {
        name: "Fortran",
        line_comments: &["!", "C", "c", "*"],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &["'", "\""],
        raw_string_start: None,
        raw_string_end: None,
    },

    // Legacy
    "COBOL" => Language {
        name: "COBOL",
        line_comments: &["*"],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &["\"", "'"],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Pascal" => Language {
        name: "Pascal",
        line_comments: &["//"],
        block_comment_start: Some("{"),
        block_comment_end: Some("}"),
        nested_comments: false,
        string_delimiters: &["'"],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Delphi" => Language {
        name: "Delphi",
        line_comments: &["//"],
        block_comment_start: Some("{"),
        block_comment_end: Some("}"),
        nested_comments: false,
        string_delimiters: &["'"],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Ada" => Language {
        name: "Ada",
        line_comments: &["--"],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },

    // Mobile
    "Dart" => Language::c_style("Dart"),

    // Misc
    "Prolog" => Language {
        name: "Prolog",
        line_comments: &["%"],
        block_comment_start: Some("/*"),
        block_comment_end: Some("*/"),
        nested_comments: false,
        string_delimiters: &["\"", "'"],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Forth" => Language {
        name: "Forth",
        line_comments: &["\\"],
        block_comment_start: Some("("),
        block_comment_end: Some(")"),
        nested_comments: false,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },
    "ActionScript" => Language::c_style("ActionScript"),
    "Vim Script" => Language {
        name: "Vim Script",
        line_comments: &["\""],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &["'"],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Emacs Lisp" => Language {
        name: "Emacs Lisp",
        line_comments: &[";"],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },

    // Wasm
    "WebAssembly" => Language {
        name: "WebAssembly",
        line_comments: &[";;"],
        block_comment_start: Some("(;"),
        block_comment_end: Some(";)"),
        nested_comments: true,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },

    // Protocol/Schema
    "Protocol Buffers" => Language::c_style("Protocol Buffers"),
    "Thrift" => Language::c_style("Thrift"),
    "Cap'n Proto" => Language::shell_style("Cap'n Proto"),
    "FlatBuffers" => Language::c_style("FlatBuffers"),

    // Templating
    "Jinja2" => Language {
        name: "Jinja2",
        line_comments: &[],
        block_comment_start: Some("{#"),
        block_comment_end: Some("#}"),
        nested_comments: false,
        string_delimiters: &["\"", "'"],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Handlebars" => Language {
        name: "Handlebars",
        line_comments: &[],
        block_comment_start: Some("{{!--"),
        block_comment_end: Some("--}}"),
        nested_comments: false,
        string_delimiters: &["\"", "'"],
        raw_string_start: None,
        raw_string_end: None,
    },
    "EJS" => Language {
        name: "EJS",
        line_comments: &[],
        block_comment_start: Some("<%#"),
        block_comment_end: Some("%>"),
        nested_comments: false,
        string_delimiters: &["\"", "'"],
        raw_string_start: None,
        raw_string_end: None,
    },
    "ERB" => Language {
        name: "ERB",
        line_comments: &[],
        block_comment_start: Some("<%#"),
        block_comment_end: Some("%>"),
        nested_comments: false,
        string_delimiters: &["\"", "'"],
        raw_string_start: None,
        raw_string_end: None,
    },

    // Solidity / Smart Contracts
    "Solidity" => Language::c_style("Solidity"),
    "Vyper" => Language::shell_style("Vyper").with_block_comments("\"\"\"", "\"\"\""),

    // Modern config
    "Jsonnet" => Language::c_style("Jsonnet"),
    "Dhall" => Language {
        name: "Dhall",
        line_comments: &["--"],
        block_comment_start: Some("{-"),
        block_comment_end: Some("-}"),
        nested_comments: true,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },
    "CUE" => Language::c_style("CUE"),
    "KDL" => Language::c_style("KDL"),

    // Gleam
    "Gleam" => Language::c_style("Gleam"),

    // Roc
    "Roc" => Language::shell_style("Roc"),

    // Grain
    "Grain" => Language::c_style("Grain"),

    // Move
    "Move" => Language::c_style("Move"),

    // Windows/Visual Studio
    "Windows Resource" => Language::c_style("Windows Resource"),
    "MSBuild" => Language::html_style("MSBuild"),
    "Visual Studio Solution" => Language::shell_style("Visual Studio Solution"),
    "XSD" => Language::html_style("XSD"),
    "Windows Module Definition" => Language {
        name: "Windows Module Definition",
        line_comments: &[";"],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },
    "C# Generated" => Language::c_style("C# Generated"),
    "InstallShield" => Language {
        name: "InstallShield",
        line_comments: &["//"],
        block_comment_start: Some("/*"),
        block_comment_end: Some("*/"),
        nested_comments: false,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Civet" => Language::c_style("Civet"),

    // Org Mode
    "Org" => Language::shell_style("Org"),

    // Infrastructure & DevOps
    "Aria" => Language::shell_style("Aria"),
    "AXAML" => Language::html_style("AXAML"),
    "Bicep" => Language::c_style("Bicep"),
    "BitBake" => Language::shell_style("BitBake"),
    "Clarity" => Language {
        name: "Clarity",
        line_comments: &[";;"],
        block_comment_start: None,
        block_comment_end: None,
        nested_comments: false,
        string_delimiters: &["\""],
        raw_string_start: None,
        raw_string_end: None,
    },
    "Magik" => Language::shell_style("Magik"),
    "Rego" => Language::shell_style("Rego"),
    "USS" => Language::c_style("USS"),
    "UXML" => Language::html_style("UXML"),
    "VSCode Workspace" => Language::new("VSCode Workspace"),
    "Yarn" => Language::shell_style("Yarn"),

    // Plain Text
    "Text" => Language::new("Text"),

    // SVG
    "SVG" => Language::html_style("SVG"),
};

pub static EXTENSION_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    // Rust
    "rs" => "Rust",

    // C/C++
    "c" => "C",
    "h" => "C Header",
    "cc" => "C++",
    "cpp" => "C++",
    "cxx" => "C++",
    "c++" => "C++",
    "hh" => "C++ Header",
    "hpp" => "C++ Header",
    "hxx" => "C++ Header",
    "h++" => "C++ Header",
    "ipp" => "C++ Header",
    "inl" => "C++ Header",

    // Objective-C
    "m" => "Objective-C",
    "mm" => "Objective-C++",

    // D
    "d" => "D",
    "di" => "D",

    // Zig
    "zig" => "Zig",

    // Odin
    "odin" => "Odin",

    // V (note: .v conflicts with Verilog, V uses .v and .vv)
    "vv" => "V",

    // Nim
    "nim" => "Nim",
    "nims" => "Nim",
    "nimble" => "Nim",

    // Crystal
    "cr" => "Crystal",

    // Java
    "java" => "Java",

    // Kotlin
    "kt" => "Kotlin",
    "kts" => "Kotlin",

    // Scala
    "scala" => "Scala",
    "sc" => "Scala",

    // Groovy
    "groovy" => "Groovy",
    "gradle" => "Groovy",
    "gvy" => "Groovy",
    "gy" => "Groovy",

    // Clojure
    "clj" => "Clojure",
    "cljs" => "Clojure",
    "cljc" => "Clojure",
    "edn" => "Clojure",

    // C#
    "cs" => "C#",
    "csx" => "C#",

    // F#
    "fs" => "F#",
    "fsi" => "F#",
    "fsx" => "F#",
    "fsscript" => "F#",

    // Visual Basic
    "vb" => "Visual Basic",
    "vbs" => "Visual Basic",

    // JavaScript/TypeScript
    "js" => "JavaScript",
    "mjs" => "JavaScript",
    "cjs" => "JavaScript",
    "ts" => "TypeScript",
    "mts" => "TypeScript",
    "cts" => "TypeScript",
    "jsx" => "JSX",
    "tsx" => "TSX",
    "coffee" => "CoffeeScript",

    // Web
    "html" => "HTML",
    "htm" => "HTML",
    "xhtml" => "HTML",
    "css" => "CSS",
    "scss" => "SCSS",
    "sass" => "Sass",
    "less" => "Less",
    "vue" => "Vue",
    "svelte" => "Svelte",

    // Python
    "py" => "Python",
    "pyi" => "Python",
    "pyw" => "Python",
    "pyx" => "Python",
    "pxd" => "Python",

    // Ruby
    "rb" => "Ruby",
    "rake" => "Ruby",
    "gemspec" => "Ruby",
    "erb" => "ERB",

    // Perl
    "pl" => "Perl",
    "pm" => "Perl",
    "pod" => "Perl",
    "t" => "Perl",

    // PHP
    "php" => "PHP",
    "php3" => "PHP",
    "php4" => "PHP",
    "php5" => "PHP",
    "php7" => "PHP",
    "phtml" => "PHP",

    // Lua
    "lua" => "Lua",

    // Tcl
    "tcl" => "Tcl",
    "tk" => "Tcl",

    // Awk
    "awk" => "Awk",
    "gawk" => "Awk",

    // Shell
    "sh" => "Shell",
    "bash" => "Bash",
    "zsh" => "Zsh",
    "fish" => "Fish",
    "ps1" => "PowerShell",
    "psm1" => "PowerShell",
    "psd1" => "PowerShell",
    "bat" => "Batch",
    "cmd" => "Batch",

    // Haskell
    "hs" => "Haskell",
    "lhs" => "Haskell",

    // OCaml
    "ml" => "OCaml",
    "mli" => "OCaml",

    // Standard ML
    "sml" => "Standard ML",
    "sig" => "Standard ML",
    "fun" => "Standard ML",

    // Elm
    "elm" => "Elm",

    // Erlang/Elixir
    "erl" => "Erlang",
    "hrl" => "Erlang",
    "ex" => "Elixir",
    "exs" => "Elixir",
    "heex" => "Elixir",
    "leex" => "Elixir",

    // Lisp family
    "lisp" => "Lisp",
    "lsp" => "Lisp",
    "cl" => "Lisp",
    "scm" => "Scheme",
    "ss" => "Scheme",
    "rkt" => "Racket",
    "el" => "Emacs Lisp",
    "elc" => "Emacs Lisp",

    // Go
    "go" => "Go",

    // Swift
    "swift" => "Swift",

    // Data/Config
    "json" => "JSON",
    "json5" => "JSON5",
    "yaml" => "YAML",
    "yml" => "YAML",
    "toml" => "TOML",
    "xml" => "XML",
    "xsl" => "XML",
    "xslt" => "XML",
    "xsd" => "XML",
    "dtd" => "XML",
    "ini" => "INI",
    "cfg" => "INI",
    "conf" => "INI",
    "properties" => "Properties",

    // Query
    "sql" => "SQL",
    "mysql" => "SQL",
    "pgsql" => "SQL",
    "plsql" => "SQL",
    "graphql" => "GraphQL",
    "gql" => "GraphQL",

    // Build
    "mk" => "Makefile",
    "cmake" => "CMake",
    "meson" => "Meson",
    "dockerfile" => "Dockerfile",
    "tf" => "Terraform",
    "tfvars" => "Terraform",
    "hcl" => "HCL",
    "nix" => "Nix",
    "bzl" => "Bazel",
    "just" => "Just",

    // Documentation
    "md" => "Markdown",
    "markdown" => "Markdown",
    "rst" => "reStructuredText",
    "adoc" => "AsciiDoc",
    "asciidoc" => "AsciiDoc",
    "tex" => "LaTeX",
    "latex" => "LaTeX",
    "sty" => "LaTeX",
    "cls" => "LaTeX",
    "bib" => "LaTeX",

    // Assembly
    "asm" => "Assembly",
    "s" => "Assembly",
    "S" => "ARM Assembly",

    // Hardware
    "v" => "Verilog",
    "sv" => "SystemVerilog",
    "svh" => "SystemVerilog",
    "vhd" => "VHDL",
    "vhdl" => "VHDL",

    // Scientific
    "r" => "R",
    "R" => "R",
    "jl" => "Julia",
    // Note: .m conflicts with Objective-C, MATLAB files typically use .mat or are detected by content
    "mat" => "MATLAB",
    "f" => "Fortran",
    "for" => "Fortran",
    "f77" => "Fortran",
    "f90" => "Fortran",
    "f95" => "Fortran",
    "f03" => "Fortran",
    "f08" => "Fortran",

    // Legacy
    "cob" => "COBOL",
    "cbl" => "COBOL",
    "cpy" => "COBOL",
    "pas" => "Pascal",
    "pp" => "Pascal",
    "dpr" => "Delphi",
    "dpk" => "Delphi",
    "ada" => "Ada",
    "adb" => "Ada",
    "ads" => "Ada",

    // Mobile
    "dart" => "Dart",

    // Misc
    "pro" => "Prolog",
    "P" => "Prolog",
    "4th" => "Forth",
    "fth" => "Forth",
    "forth" => "Forth",
    "as" => "ActionScript",
    "vim" => "Vim Script",
    "vimrc" => "Vim Script",

    // WebAssembly
    "wat" => "WebAssembly",
    "wast" => "WebAssembly",

    // Protocol/Schema
    "proto" => "Protocol Buffers",
    "thrift" => "Thrift",
    "capnp" => "Cap'n Proto",
    "fbs" => "FlatBuffers",

    // Templating
    "j2" => "Jinja2",
    "jinja" => "Jinja2",
    "jinja2" => "Jinja2",
    "hbs" => "Handlebars",
    "handlebars" => "Handlebars",
    "ejs" => "EJS",

    // Smart Contracts
    "sol" => "Solidity",
    "vy" => "Vyper",

    // Modern Config
    "jsonnet" => "Jsonnet",
    "libsonnet" => "Jsonnet",
    "dhall" => "Dhall",
    "cue" => "CUE",
    "kdl" => "KDL",

    // New Languages
    "gleam" => "Gleam",
    "roc" => "Roc",
    "gr" => "Grain",
    "move" => "Move",

    // Windows/Visual Studio
    "rc" => "Windows Resource",
    "rc2" => "Windows Resource",
    "csproj" => "MSBuild",
    "vbproj" => "MSBuild",
    "fsproj" => "MSBuild",
    "vcxproj" => "MSBuild",
    "props" => "MSBuild",
    "targets" => "MSBuild",
    "sln" => "Visual Studio Solution",
    "def" => "Windows Module Definition",

    // InstallShield
    "ism" => "InstallShield",
    "iss" => "InstallShield",

    // Civet
    "civet" => "Civet",

    // Org Mode
    "org" => "Org",

    // Plain Text
    "txt" => "Text",

    // SVG
    "svg" => "SVG",

    // Additional Extensions
    "aria" => "Aria",
    "axaml" => "AXAML",
    "bicep" => "Bicep",
    "bicepparam" => "Bicep",
    "bb" => "BitBake",
    "bbappend" => "BitBake",
    "bbclass" => "BitBake",
    "clar" => "Clarity",
    "magik" => "Magik",
    "rego" => "Rego",
    "uss" => "USS",
    "uxml" => "UXML",
    "code-workspace" => "VSCode Workspace",
    "Dsr" => "Visual Basic",
};

pub static FILENAME_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    "Makefile" => "Makefile",
    "makefile" => "Makefile",
    "GNUmakefile" => "Makefile",
    "CMakeLists.txt" => "CMake",
    "Dockerfile" => "Dockerfile",
    "dockerfile" => "Dockerfile",
    "Containerfile" => "Dockerfile",
    "docker-compose.yml" => "Docker Compose",
    "docker-compose.yaml" => "Docker Compose",
    "compose.yml" => "Docker Compose",
    "compose.yaml" => "Docker Compose",
    "Gemfile" => "Ruby",
    "Rakefile" => "Ruby",
    "Vagrantfile" => "Ruby",
    "Berksfile" => "Ruby",
    "Guardfile" => "Ruby",
    "Podfile" => "Ruby",
    "Fastfile" => "Ruby",
    "Appfile" => "Ruby",
    "Dangerfile" => "Ruby",
    "BUILD" => "Bazel",
    "BUILD.bazel" => "Bazel",
    "WORKSPACE" => "Bazel",
    "WORKSPACE.bazel" => "Bazel",
    "Justfile" => "Just",
    "justfile" => "Just",
    ".justfile" => "Just",
    ".bashrc" => "Bash",
    ".bash_profile" => "Bash",
    ".bash_aliases" => "Bash",
    ".profile" => "Shell",
    ".zshrc" => "Zsh",
    ".zprofile" => "Zsh",
    ".zshenv" => "Zsh",
    "config.fish" => "Fish",
    ".vimrc" => "Vim Script",
    ".gvimrc" => "Vim Script",
    "_vimrc" => "Vim Script",
    ".emacs" => "Emacs Lisp",
    "meson.build" => "Meson",
    "meson_options.txt" => "Meson",
    "go.mod" => "Go",
    "go.sum" => "Go",
    "Cargo.toml" => "TOML",
    "Cargo.lock" => "TOML",
    "pyproject.toml" => "TOML",
    "poetry.lock" => "TOML",
    "package.json" => "JSON",
    "package-lock.json" => "JSON",
    "tsconfig.json" => "JSON",
    "jsconfig.json" => "JSON",
    ".eslintrc" => "JSON",
    ".prettierrc" => "JSON",
    "flake.nix" => "Nix",
    "flake.lock" => "JSON",
    "default.nix" => "Nix",
    "shell.nix" => "Nix",
};

pub fn detect_language(path: &Path) -> Option<&'static Language> {
    // Check custom languages first (if any are loaded)
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if let Some(lang) = crate::custom_langs::CustomLanguages::get_by_extension(ext) {
            return Some(lang);
        }
    }

    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
        if let Some(&lang_name) = FILENAME_MAP.get(filename) {
            return LANGUAGES.get(lang_name);
        }

        // C# Generated files (.g.cs, .designer.cs)
        let lower = filename.to_lowercase();
        if lower.ends_with(".g.cs") || lower.ends_with(".designer.cs") {
            return LANGUAGES.get("C# Generated");
        }
    }

    if let Some(ext) = path.extension().and_then(|e| e.to_str())
        && let Some(&lang_name) = EXTENSION_MAP.get(ext) {
            return LANGUAGES.get(lang_name);
        }

    None
}

#[allow(dead_code)]
pub fn get_language(name: &str) -> Option<&'static Language> {
    LANGUAGES.get(name)
}

pub fn list_languages() -> impl Iterator<Item = (&'static str, &'static Language)> {
    LANGUAGES.entries().map(|(k, v)| (*k, v))
}

pub fn list_extensions() -> impl Iterator<Item = (&'static str, &'static str)> {
    EXTENSION_MAP.entries().map(|(k, v)| (*k, *v))
}
