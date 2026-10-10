// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

//! Cargo-based Rust source discovery (syntactic, not compiler name resolution).

use crate::SymbolResolver;
use fsscanner::fsscanner_base::collect_files_fast;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use syn::{Item, Type, UseTree};
use std::process::Command;

#[derive(Default)]
struct ScanContext {
    source_files: BTreeSet<PathBuf>,
    visited: BTreeSet<(PathBuf, Vec<String>)>,
    definitions: BTreeMap<Vec<String>, BTreeSet<PathBuf>>,
    implementations: Vec<(Vec<String>, PathBuf)>,
    aliases: BTreeMap<Vec<String>, Vec<String>>,
}

pub struct CargoRustResolver {
    project_dir: PathBuf,
    expand_macros: bool,
}

impl CargoRustResolver {
    pub fn new(project_dir: impl AsRef<Path>) -> Result<Self, cargo_metadata::Error> {
        let project_dir = project_dir.as_ref().to_path_buf();
        cargo_metadata::MetadataCommand::new().current_dir(&project_dir).no_deps().exec()?;
        Ok(Self { project_dir, expand_macros: false })
    }

    /// Enables optional macro expansion. Disabled by default to avoid launching
    /// additional builds for ordinary symbol misses.
    pub fn with_macro_expansion(mut self, enabled: bool) -> Self {
        self.expand_macros = enabled;
        self
    }

    /// Returns sorted, unique source files defining or implementing the requested symbol.
    /// Qualified queries are matched by suffix (e.g. `mod::Thing`).
    pub fn resolve_files(&self, symbol: &str) -> Result<Vec<PathBuf>, cargo_metadata::Error> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .current_dir(&self.project_dir).no_deps().exec()?;
        let query: Vec<&str> = symbol.split("::").collect();
        if query.iter().any(|part| part.is_empty()) {
            return Ok(Vec::new());
        }
        let mut context = ScanContext::default();
        for package in &metadata.packages {
            if !metadata.workspace_members.contains(&package.id) { continue; }
            for target in &package.targets {
                let root = target.src_path.as_std_path();
                // Discover Rust sources through fsscanner; module traversal below
                // determines which discovered files belong to each module.
                let mut discovered = Vec::new();
                if let Some(parent) = root.parent() {
                    collect_files_fast(parent, "rs", &mut discovered);
                }
                context.source_files.extend(discovered.into_iter().filter_map(|path| path.canonicalize().ok()));
                let crate_name = target.name.replace('-', "_");
                let prefix = vec![crate_name];
                scan_file(root, &prefix, &mut context);
            }
        }

        // Resolve imported names (including public re-exports) to their defining types.
        for _ in 0..context.aliases.len().saturating_add(1) {
            let mut changed = false;
            for (alias, target) in &context.aliases {
                if let Some(files) = context.definitions.get(target).cloned() {
                    let entry = context.definitions.entry(alias.clone()).or_default();
                    let before = entry.len();
                    entry.extend(files);
                    changed |= entry.len() != before;
                }
            }
            if !changed { break; }
        }
        let mut selected = BTreeSet::new();
        let matching_types: BTreeSet<Vec<String>> = context.definitions.keys()
            .filter(|name| suffix_matches(name, &query))
            .cloned().collect();
        for name in &matching_types {
            if let Some(files) = context.definitions.get(name) {
                selected.extend(files.iter().cloned());
            }
        }
        for (name, path) in &context.implementations {
            if matching_types.contains(name) || (matching_types.is_empty() && suffix_matches(name, &query)) || context.aliases.iter().any(|(alias, target)| matching_types.contains(alias) && target == name) {
                selected.insert(path.clone());
            }
        }
        // cargo-expand can expose macro-generated declarations, but its output
        // does not preserve reliable source locations. Use the target root as a
        // conservative candidate only when no source match was found.
        if self.expand_macros && selected.is_empty() {
            for package in &metadata.packages {
                if !metadata.workspace_members.contains(&package.id) { continue; }
                for target in &package.targets {
                    let output = Command::new("cargo")
                        .arg("expand").arg("--lib")
                        .current_dir(package.manifest_path.as_std_path().parent().unwrap_or(Path::new(".")))
                        .output();
                    if let Ok(output) = output
                        && output.status.success()
                        && let Ok(expanded) = syn::parse_file(&String::from_utf8_lossy(&output.stdout)) {
                                let mut expanded_context = ScanContext::default();
                                scan_items(&expanded.items, target.src_path.as_std_path(), Path::new("."),
                                    &[target.name.replace('-', "_")], &mut expanded_context);
                                if expanded_context.definitions.keys().any(|p| suffix_matches(p, &query))
                                    || expanded_context.implementations.iter().any(|(p, _)| suffix_matches(p, &query)) {
                                    selected.insert(target.src_path.as_std_path().to_path_buf());
                                }
                        }
                }
            }
        }
        Ok(selected.into_iter().collect())
    }
}

fn suffix_matches(path: &[String], query: &[&str]) -> bool {
    path.len() >= query.len()
        && path[path.len() - query.len()..].iter().zip(query).all(|(a, b)| a == b)
}

fn scan_file(file: &Path, module: &[String], context: &mut ScanContext) {
    let Ok(file) = file.canonicalize() else { return };
    if !context.source_files.contains(&file) { return; }
    if !context.visited.insert((file.clone(), module.to_vec())) { return; }
    let Ok(source) = std::fs::read_to_string(&file) else { return };
    let Ok(ast) = syn::parse_file(&source) else { return };
    // Rust's module lookup base differs between lib.rs/mod.rs and foo.rs.
    let parent = file.parent().unwrap_or(Path::new("."));
    let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let child_dir = if matches!(stem, "lib" | "main" | "mod") {
        parent.to_path_buf()
    } else {
        parent.join(stem)
    };
    scan_items(&ast.items, &file, &child_dir, module, context);
}

fn scan_items(items: &[Item], file: &Path, child_dir: &Path, module: &[String], context: &mut ScanContext) {
    for item in items {
        match item {
            Item::Use(u) => collect_use(&u.tree, module, &[], &mut context.aliases),
            Item::Struct(s) => add_def(module, &s.ident.to_string(), file, &mut context.definitions),
            Item::Enum(s) => add_def(module, &s.ident.to_string(), file, &mut context.definitions),
            Item::Union(s) => add_def(module, &s.ident.to_string(), file, &mut context.definitions),
            Item::Trait(s) => add_def(module, &s.ident.to_string(), file, &mut context.definitions),
            Item::Fn(s) => add_def(module, &s.sig.ident.to_string(), file, &mut context.definitions),
            Item::Mod(m) => {
                let mut nested = module.to_vec();
                nested.push(m.ident.to_string());
                if let Some((_, items)) = &m.content {
                    scan_items(items, file, &child_dir.join(m.ident.to_string()), &nested, context);
                } else {
                    let explicit = m.attrs.iter().find_map(|a| {
                        if a.path().is_ident("path") {
                            match &a.meta {
                                syn::Meta::NameValue(v) => {
                                    if let syn::Expr::Lit(e) = &v.value
                                        && let syn::Lit::Str(s) = &e.lit { return Some(s.value()); }
                                    None
                                }
                                _ => None,
                            }
                        } else { None }
                    });
                    let candidate = if let Some(path) = explicit {
                        file.parent().unwrap_or(Path::new(".")).join(path)
                    } else {
                        let a = child_dir.join(format!("{}.rs", m.ident));
                        if a.exists() { a } else { child_dir.join(m.ident.to_string()).join("mod.rs") }
                    };
                    scan_file(&candidate, &nested, context);
                }
            }
            Item::Impl(imp) => {
                if let Type::Path(ty) = imp.self_ty.as_ref()
                    && ty.qself.is_none() {
                        let parts: Vec<String> = ty.path.segments.iter().map(|s| s.ident.to_string()).collect();
                        let resolved = if parts.first().is_some_and(|s| s == "crate") {
                            [module[..1].to_vec(), parts[1..].to_vec()].concat()
                        } else if parts.first().is_some_and(|s| s == "self") {
                            [module.to_vec(), parts[1..].to_vec()].concat()
                        } else if parts.first().is_some_and(|s| s == "super") {
                            [module[..module.len().saturating_sub(1)].to_vec(), parts[1..].to_vec()].concat()
                        } else {
                            [module.to_vec(), parts].concat()
                        };
                        let resolved = context.aliases.get(&resolved).cloned().unwrap_or(resolved);
                        context.implementations.push((resolved, file.to_path_buf()));
                }
            }
            _ => {}
        }
    }
}

fn collect_use(
    tree: &UseTree, module: &[String], prefix: &[String],
    aliases: &mut BTreeMap<Vec<String>, Vec<String>>,
) {
    match tree {
        UseTree::Path(p) => {
            let mut next = prefix.to_vec();
            next.push(p.ident.to_string());
            collect_use(&p.tree, module, &next, aliases);
        }
        UseTree::Name(n) => {
            let mut path = prefix.to_vec();
            path.push(n.ident.to_string());
            register_alias(module, &path, &n.ident.to_string(), aliases);
        }
        UseTree::Rename(r) => {
            let mut path = prefix.to_vec();
            path.push(r.ident.to_string());
            register_alias(module, &path, &r.rename.to_string(), aliases);
        }
        UseTree::Group(g) => for item in &g.items { collect_use(item, module, prefix, aliases); },
        UseTree::Glob(_) => {}
    }
}

fn register_alias(
    module: &[String], path: &[String], name: &str,
    aliases: &mut BTreeMap<Vec<String>, Vec<String>>,
) {
    let mut target = module.to_vec();
    if path.first().is_some_and(|p| p == "crate") {
        target = module[..1].to_vec();
        target.extend_from_slice(&path[1..]);
    } else if path.first().is_some_and(|p| p == "self") {
        target.extend_from_slice(&path[1..]);
    } else if path.first().is_some_and(|p| p == "super") {
        target.pop();
        target.extend_from_slice(&path[1..]);
    } else {
        target.extend_from_slice(path);
    }
    let mut alias = module.to_vec();
    alias.push(name.to_owned());
    aliases.insert(alias, target);
}

fn add_def(module: &[String], name: &str, file: &Path, defs: &mut BTreeMap<Vec<String>, BTreeSet<PathBuf>>) {
    let mut full = module.to_vec();
    full.push(name.to_owned());
    defs.entry(full).or_default().insert(file.to_path_buf());
}

impl SymbolResolver for CargoRustResolver {
    fn resolve(&self, symbol: &str) -> String {
        self.resolve_files(symbol)
            .map(|files| files.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("\n"))
            .unwrap_or_default()
    }
}
