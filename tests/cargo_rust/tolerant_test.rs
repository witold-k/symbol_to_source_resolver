// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use std::fs;
use symbol_to_source_resolver::cargo_rust::CargoRustResolver;

#[test]
fn recovers_symbols_from_broken_rust_source() {
    let root = std::env::temp_dir().join(format!("rust-tolerant-resolver-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("Cargo.toml"), "[package]\nname = \"tolerant_fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n").unwrap();
    fs::write(root.join("src/lib.rs"), "pub struct Found;\nfn broken( {\n").unwrap();
    let resolver = CargoRustResolver::new(&root).unwrap();
    let found = resolver.resolve_files("Found").unwrap();
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].ends_with("lib.rs"));
    fs::remove_dir_all(root).unwrap();
}
