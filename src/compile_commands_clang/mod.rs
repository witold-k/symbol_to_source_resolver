// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

//! Error-tolerant C/C++ source discovery from compile_commands.json.
//! Does not execute the compiler or require a successful build.

use crate::SymbolResolver;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum CompileCommandsClangError {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidDatabase,
}
impl std::fmt::Display for CompileCommandsClangError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::InvalidDatabase => write!(f, "invalid compile_commands.json: expected an array"),
        }
    }
}
impl std::error::Error for CompileCommandsClangError {}
impl From<std::io::Error> for CompileCommandsClangError {
    fn from(e: std::io::Error) -> Self { Self::Io(e) }
}
impl From<serde_json::Error> for CompileCommandsClangError {
    fn from(e: serde_json::Error) -> Self { Self::Json(e) }
}

pub struct CompileCommandsClangResolver {
    sources: BTreeSet<PathBuf>,
}
impl CompileCommandsClangResolver {
    pub fn new(project_dir: impl AsRef<Path>) -> Result<Self, CompileCommandsClangError> {
        let root = project_dir.as_ref();
        let db = if root.is_file() { root.to_path_buf() } else { root.join("compile_commands.json") };
        let entries: serde_json::Value = serde_json::from_slice(&std::fs::read(&db)?)?;
        let entries = entries.as_array().ok_or(CompileCommandsClangError::InvalidDatabase)?;
        let mut sources = BTreeSet::new();
        for entry in entries {
            let (Some(file), Some(directory)) = (
                entry.get("file").and_then(|v| v.as_str()),
                entry.get("directory").and_then(|v| v.as_str()),
            ) else { continue };
            let base = Path::new(directory);
            let base = if base.is_absolute() { base.to_path_buf() } else {
                db.parent().unwrap_or(Path::new(".")).join(base)
            };
            let path = Path::new(file);
            let path = if path.is_absolute() { path.to_path_buf() } else { base.join(path) };
            if let Ok(path) = path.canonicalize() { sources.insert(path); }
        }
        // Headers are absent from most compilation databases. Discover project-local
        // headers independently, without preprocessing or invoking the compiler.
        let project_root = if root.is_file() {
            root.parent().unwrap_or(Path::new("."))
        } else { root };
        collect_headers(project_root, &mut sources);
        Ok(Self { sources })
    }

    /// Returns sorted source files containing matching named declarations.
    /// Matching is syntactic; compiler name lookup is not performed.
    pub fn resolve_files(&self, symbol: &str) -> Vec<PathBuf> {
        let query: Vec<&str> = symbol.split("::").collect();
        if query.iter().any(|s| s.is_empty()) { return Vec::new(); }
        self.sources.iter().filter(|file| {
            let Ok(source) = std::fs::read_to_string(file) else { return false };
            let cpp = matches!(file.extension().and_then(|s| s.to_str()), Some("cpp" | "cxx" | "cc" | "hpp" | "hxx" | "hh" | "h" | "h++"));
            let mut parser = tree_sitter::Parser::new();
            let language = if cpp { tree_sitter_cpp::LANGUAGE.into() } else { tree_sitter_c::LANGUAGE.into() };
            if parser.set_language(&language).is_err() { return false; }
            let Some(tree) = parser.parse(&source, None) else { return false };
            contains_symbol(tree.root_node(), source.as_bytes(), &[], &query)
        }).cloned().collect()
    }
}
fn collect_headers(root: &Path, sources: &mut BTreeSet<PathBuf>) {
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(kind) = entry.file_type() else { continue };
            if kind.is_symlink() { continue; }
            if kind.is_dir() {
                let excluded = path.file_name().and_then(|s| s.to_str())
                    .is_some_and(|name| matches!(name, ".git" | "target" | "build" | "out" | ".cache"));
                if !excluded { pending.push(path); }
            } else if kind.is_file()
                && matches!(path.extension().and_then(|s| s.to_str()), Some("h" | "hpp" | "hh" | "hxx" | "h++"))
                && let Ok(path) = path.canonicalize()
            {
                sources.insert(path);
            }
        }
    }
}

fn contains_symbol(node: tree_sitter::Node<'_>, source: &[u8], scope: &[String], query: &[&str]) -> bool {
    let kind = node.kind();
    let mut nested = scope.to_vec();
    if matches!(kind, "namespace_definition" | "class_specifier" | "struct_specifier" | "union_specifier" | "enum_specifier")
        && let Some(name) = node.child_by_field_name("name")
        && let Ok(name) = name.utf8_text(source)
    {
        let parts: Vec<String> = name.split("::").map(str::to_owned).collect();
        nested.extend(parts);
        if suffix_matches(&nested, query) { return true; }
    }
    if matches!(kind, "function_definition" | "declaration" | "type_definition" | "alias_declaration")
        && let Some(declarator) = node.child_by_field_name("declarator").or_else(|| node.child_by_field_name("name"))
        && let Some(name) = declarator_name(declarator, source)
    {
        let mut full = scope.to_vec();
        full.extend(name.split("::").map(str::to_owned));
        if suffix_matches(&full, query) { return true; }
    }
    let child_scope = if matches!(kind, "namespace_definition" | "class_specifier" | "struct_specifier" | "union_specifier") { &nested } else { scope };
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if contains_symbol(child, source, child_scope, query) { return true; }
    }
    false
}
fn declarator_name(node: tree_sitter::Node<'_>, source: &[u8]) -> Option<String> {
    if matches!(node.kind(), "identifier" | "field_identifier" | "type_identifier" | "qualified_identifier" | "scoped_identifier") {
        return node.utf8_text(source).ok().map(str::to_owned);
    }
    if let Some(inner) = node.child_by_field_name("declarator").or_else(|| node.child_by_field_name("name")) {
        return declarator_name(inner, source);
    }
    None
}
fn suffix_matches(path: &[String], query: &[&str]) -> bool {
    path.len() >= query.len() && path[path.len()-query.len()..].iter().map(String::as_str).eq(query.iter().copied())
}
impl SymbolResolver for CompileCommandsClangResolver {
    fn resolve(&self, symbol: &str) -> String {
        self.resolve_files(symbol).iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("\n")
    }
}
