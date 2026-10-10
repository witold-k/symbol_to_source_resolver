// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use std::fs;
use symbol_to_source_resolver::cargo_rust::CargoRustResolver;

#[test]
fn recovers_alias_and_impl_from_broken_source() {
    let root = std::env::temp_dir().join(format!("rust-tolerant-alias-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("Cargo.toml"), "[package]\nname = \"tolerant_alias_fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n").unwrap();
    fs::write(root.join("src/lib.rs"), "mod model;\nmod extra;\npub use crate::model::Widget as Exported;\nfn broken( {\n").unwrap();
    fs::write(root.join("src/model.rs"), "pub struct Widget;\n").unwrap();
    fs::write(root.join("src/extra.rs"), "use crate::model::Widget;\nimpl Widget { pub fn work(&self) {} }\nfn broken( {\n").unwrap();
    let resolver = CargoRustResolver::new(&root).unwrap();
    let found = resolver.resolve_files("Exported").unwrap();
    assert!(found.iter().any(|p| p.ends_with("model.rs")), "{found:?}");
    let found = resolver.resolve_files("Widget").unwrap();
    assert!(found.iter().any(|p| p.ends_with("extra.rs")), "{found:?}");
    fs::remove_dir_all(root).unwrap();
}
