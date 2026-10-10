// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use std::fs;
use symbol_to_source_resolver::cargo_rust::CargoRustResolver;

#[test]
fn resolves_all_impl_files() {
    let temp = std::env::temp_dir().join(format!("resolver-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(temp.join("src")).unwrap();
    fs::write(temp.join("Cargo.toml"), "[package]\nname = \"resolver_fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n").unwrap();
    fs::write(temp.join("src/lib.rs"), "pub struct Example;\nmod extra;\n").unwrap();
    fs::write(temp.join("src/extra.rs"), "impl crate::Example { pub fn run(&self) {} }\n").unwrap();

    let resolver = CargoRustResolver::new(&temp).unwrap();
    let files = resolver.resolve_files("Example").unwrap();
    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|p| p.ends_with("src/lib.rs")));
    assert!(files.iter().any(|p| p.ends_with("src/extra.rs")));
    fs::remove_dir_all(temp).unwrap();
}
