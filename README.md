# symbol_to_source_resolver

> **Status: Planning only — not implemented yet.**
>
> This repository currently documents an idea for a reusable Rust library. There is no working resolver, public API, or implementation. The examples below describe intended behavior, not existing functionality.

## Goal

Resolve a source-code symbol (such as a fully qualified class, struct, method, or module name) to **one or more source files** in a project.

The library is intended for reuse by tools such as [aiagents](https://github.com/witold-k/aiagents), without depending on aiagents itself.

## Examples of intended mappings

| Language | Symbol | Potential source files |
| --- | --- | --- |
| C++ | `foo::bar::MyClass` | `include/foo/bar/MyClass.hpp`, `src/MyClass.cpp` |
| Rust | `foo::bar::MyStruct` | `src/foo/bar.rs`, `src/foo/bar/mod.rs` |
| Java | `com.example.MyClass` | `src/main/java/com/example/MyClass.java` |

These are illustrative guesses. In particular, C++ namespaces and class names do not imply a unique directory structure or filename.

## Intended requirements

- Support C and C++, Rust, and Java, with room for further languages.
- Accept qualified symbol names and return **multiple candidates** where appropriate.
- Distinguish successful matches from guesses and missing results.
- Keep filesystem discovery and language-specific matching logic modular.
- Remain a small, reusable Rust library rather than a component tied to a single application.

## Possible approach (not decided)

A first version could combine filesystem path/name heuristics with language-specific conventions. Later versions might use parsers or a source-symbol index to locate declarations and definitions reliably. The exact API, indexing strategy, dependencies, and performance requirements are **still to be designed**.

## Current state

**No implementation exists.** This README is a planning note, not a promise of supported functionality. Contributions to the design can begin before any code is added.
