// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use std::fs;
use symbol_to_source_resolver::cargo_rust::CargoRustResolver;

#[test]
fn resolves_named_import_and_reexport() {
    let dir = std::env::temp_dir().join(format!("resolver-alias-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("Cargo.toml"), "[package]\nname = \"alias_fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n").unwrap();
    fs::write(dir.join("src/lib.rs"), "mod model;\npub use crate::model::Thing as PublicThing;\n").unwrap();
    fs::write(dir.join("src/model.rs"), "pub struct Thing;\n").unwrap();
    let resolver = CargoRustResolver::new(&dir).unwrap();
    let files = resolver.resolve_files("alias_fixture::PublicThing").unwrap();
    assert_eq!(files.len(), 1, "{files:?}");
    assert!(files[0].ends_with("src/model.rs"));
    fs::remove_dir_all(dir).unwrap();
}
