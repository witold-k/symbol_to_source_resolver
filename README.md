# symbol_to_source_resolver

Resolve source symbols to source files. Rust/Cargo has an initial
implementation; Maven/Java has initial type discovery; compile_commands/Clang
has an experimental C/C++ implementation.

## Rust usage

```rust
use symbol_to_source_resolver::cargo_rust::CargoRustResolver;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = CargoRustResolver::new("/path/to/mycrate")?;

    // All matching definitions and impl blocks across the Cargo workspace:
    for path in resolver.resolve_files("MyStruct")? {
        println!("{}", path.display());
    }

    // Partially qualified: matches the end of a module path.
    let files = resolver.resolve_files("mymod::MyStruct")?;

    // Fully qualified: crate name is included.
    let files = resolver.resolve_files("mycrate::mymod::MyStruct")?;
    Ok(())
}
```

For example, a declaration in `src/mymod.rs` and separate `impl MyStruct` blocks
in `src/mymod/extra.rs` are returned together, provided the files are part of
the Rust module tree.

## Optional macro expansion

Install [cargo-expand](https://github.com/dtolnay/cargo-expand):

```sh
cargo install cargo-expand
cargo expand --version
```

Some toolchains require a nightly Rust toolchain for expansion; install one with
`rustup toolchain install nightly` if needed. The resolver first scans source
files with `syn`. If macro expansion is explicitly enabled and no match is
found, it tries `cargo expand --lib` for workspace packages. If cargo-expand is
unavailable or expansion fails, this fallback is skipped. Enable expansion
explicitly with
`CargoRustResolver::new(project_dir)?.with_macro_expansion(true)`. Expansion is
disabled by default to prevent additional builds and long-running processes
during normal symbol searches.

**Important:** Expanded code does not reliably retain original source-file
locations. A macro-generated match therefore returns the Cargo target root (e.g.
`src/lib.rs`) as a *candidate*, not the exact macro definition or invocation
file. Expansion may also be slow.

## Aliases and re-exports

Simple named imports and re-exports are followed, including:

```rust
use crate::model::MyStruct as Alias;
pub use crate::model::MyStruct;
```

Queries such as `Alias` or `mycrate::Alias` can return the original definition
file. Glob imports and more complex semantic name resolution are not yet
supported.

## File discovery

Rust source files are discovered using
[`fsscanner`](https://github.com/witold-k/fsscanner) (`collect_files_fast` with
the `rs` extension). Cargo metadata locates the workspace targets; module
parsing continues to determine Rust module ownership. The dependency is fetched
from GitHub by Cargo and pinned to a specific commit on `fsscanner`'s `master`
branch for reproducible builds.

## Current limitations

- Uses `cargo metadata` to discover workspace targets, then `syn` to inspect reachable source files. Cargo itself does not index symbol definitions.
- Qualified names match **path suffixes**. An unqualified name can match multiple types; results are sorted and deduplicated.
- Basic `mod`, inline modules, `#[path = "..."]`, and simple `crate::`, `self::`, `super::` impl paths are supported.
- Named `use` aliases and re-exports are handled syntactically; glob imports, cross-crate resolution, conditional compilation, type aliases, and macro source provenance remain limited.
- This returns file candidates, not exact line ranges. Java and C/C++ provide initial syntactic symbol discovery.

## Maven / Java

`MavenJavaResolver::new(project_dir)?` accepts a project directory even without
`pom.xml`. Call `resolve_files("org.example.MyClass")?` to locate Java classes,
interfaces, enums and records. Uses `fsscanner` to find `src/main/java` and
`src/test/java` files, including nested modules, excluding `target`. No Maven
subprocess is launched. This is syntactic matching, not Java compiler name
resolution; imports, methods, inheritance and nested type qualification are not
yet supported.

### Broken / incomplete Maven projects

Java declarations are parsed using Tree-sitter, which produces partial syntax
trees for malformed source files. Neither Maven nor javac is executed, and a
missing `pom.xml` does not prevent lookup. Unreadable individual files are
skipped. Maven-specific source-root configuration is not yet evaluated;
conventional `src/main/java` and `src/test/java` paths are scanned.

## compile_commands.json / C and C++

Use `CompileCommandsClangResolver::new(project_dir)?` with a directory
containing `compile_commands.json` (or pass the database path directly). Call
`resolve_files("Widget")` or `resolve_files("demo::Widget")`. The backend reads
translation-unit paths from the compilation database and uses Tree-sitter to
recover named declarations even in malformed C/C++ files. It does not execute
Clang or compile sources. Project-local headers (`.h`, `.hpp`, `.hh`, `.hxx`,
`.h++`) are also scanned recursively from the supplied project directory;
`.git`, `target`, `build`, `out` and `.cache` directories are skipped. This is a
project-wide scan, not a compiler-accurate include graph; headers outside the
project (including system headers and external `-I` directories) are not yet
indexed. Initial limitations: no macro expansion, compiler-level name resolution
or exact source ranges. The compilation database itself must be valid JSON.

### Implementation status (C/C++)

**Not yet tested or validated:** The current compile_commands/Clang
implementation, including header discovery, has been committed but has not yet
been compiled or executed by the assistant. The included tests are proposed
checks, not evidence of a successful test run. Run `cargo build`, `cargo test`,
and `cargo clippy -- -D warnings` before relying on this backend.
