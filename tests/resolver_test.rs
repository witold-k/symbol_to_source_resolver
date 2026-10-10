// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use symbol_to_source_resolver::{Backend, Resolver};

#[test]
fn cargo_backend_resolves_unknown_symbol_to_empty_string() {
    let resolver = Resolver::new(Backend::CargoRust);
    assert_eq!(resolver.resolve("some::qualified::Symbol"), "");
}

#[test]
fn clang_backend_requires_compilation_database() {
    let root = std::env::temp_dir().join(format!("clang-missing-db-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let result = Resolver::with_project_dir(Backend::CompileCommandsClang, &root);
    assert!(result.is_err());
    std::fs::remove_dir_all(root).unwrap();
}
