// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

//! Error-tolerant Java source discovery; does not invoke Maven or javac.
use crate::SymbolResolver;
use fsscanner::fsscanner_base::collect_files_fast;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Parser};

#[derive(Debug)]
pub enum MavenJavaError {
    Io(std::io::Error),
    ParserLanguage(tree_sitter::LanguageError),
    ParserUnavailable,
}
impl std::fmt::Display for MavenJavaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::ParserLanguage(error) => write!(f, "{error}"),
            Self::ParserUnavailable => write!(f, "Java parser failed to produce a syntax tree"),
        }
    }
}
impl std::error::Error for MavenJavaError {}
impl From<std::io::Error> for MavenJavaError {
    fn from(error: std::io::Error) -> Self { Self::Io(error) }
}
impl From<tree_sitter::LanguageError> for MavenJavaError {
    fn from(error: tree_sitter::LanguageError) -> Self { Self::ParserLanguage(error) }
}

pub struct MavenJavaResolver {
    project_dir: PathBuf,
}
impl MavenJavaResolver {
    /// A pom.xml is optional: source lookup works with incomplete Maven projects.
    pub fn new(project_dir: impl AsRef<Path>) -> Result<Self, MavenJavaError> {
        let project_dir = project_dir.as_ref().to_path_buf();
        if !project_dir.is_dir() {
            return Err(MavenJavaError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound, format!("project directory missing: {}", project_dir.display())
            )));
        }
        Ok(Self { project_dir })
    }

    /// Find type declarations, even if unrelated Java files contain syntax errors.
    pub fn resolve_files(&self, symbol: &str) -> Result<Vec<PathBuf>, MavenJavaError> {
        let query: Vec<&str> = symbol.split('.').collect();
        if query.iter().any(|part| !is_identifier(part)) { return Ok(Vec::new()); }
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_java::LANGUAGE.into())?;
        let mut paths = Vec::new();
        collect_files_fast(&self.project_dir, "java", &mut paths);
        let mut result = BTreeSet::new();
        for path in paths {
            if !is_source_path(&self.project_dir, &path) { continue; }
            // A broken or unreadable source file must not block other matches.
            let Ok(source) = std::fs::read(&path) else { continue };
            let Some(tree) = parser.parse(&source, None) else { continue };
            let root = tree.root_node();
            let package = package_name(root, &source);
            if has_type(root, &source, &package, &query) {
                result.insert(path);
            }
        }
        Ok(result.into_iter().collect())
    }
}
fn is_source_path(root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else { return false };
    let parts: Vec<String> = relative.components().map(|part| part.as_os_str().to_string_lossy().into_owned()).collect();
    !parts.iter().any(|part| matches!(part.as_str(), "target" | ".git"))
        && parts.windows(3).any(|window| window == ["src", "main", "java"] || window == ["src", "test", "java"])
}
fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars.next().is_some_and(|ch| ch.is_alphabetic() || ch == '_' || ch == '$')
        && chars.all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '$')
}
fn package_name(root: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut cursor = root.walk();
    for node in root.named_children(&mut cursor) {
        if node.kind() == "package_declaration" {
            let text = String::from_utf8_lossy(&source[node.byte_range()]);
            return text.trim_start_matches("package").trim().trim_end_matches(';')
                .split('.').map(|part| part.trim().to_string()).collect();
        }
    }
    Vec::new()
}
fn has_type(root: Node<'_>, source: &[u8], package: &[String], query: &[&str]) -> bool {
    let mut stack = vec![(root, Vec::<String>::new())];
    while let Some((node, enclosing)) = stack.pop() {
        let declaration = matches!(node.kind(),
            "class_declaration" | "interface_declaration" | "enum_declaration" |
            "record_declaration" | "annotation_type_declaration");
        let mut nested = enclosing;
        if declaration
            && let Some(name_node) = node.child_by_field_name("name")
            && let Ok(name) = name_node.utf8_text(source)
        {
            nested.push(name.to_owned());
            let qualified: Vec<&str> = package.iter().map(String::as_str)
                .chain(nested.iter().map(String::as_str)).collect();
            if qualified.ends_with(query) { return true; }
        }
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            stack.push((child, nested.clone()));
        }
    }
    false
}
impl SymbolResolver for MavenJavaResolver {
    fn resolve(&self, symbol: &str) -> String {
        self.resolve_files(symbol)
            .map(|paths| paths.iter().map(|path| path.display().to_string()).collect::<Vec<_>>().join("\n"))
            .unwrap_or_default()
    }
}
