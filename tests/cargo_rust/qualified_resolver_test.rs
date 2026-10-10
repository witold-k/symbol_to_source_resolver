// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use std::fs;
use symbol_to_source_resolver::cargo_rust::CargoRustResolver;

#[test]
fn qualified_names_find_definition_and_impls() {
    let dir = std::env::temp_dir().join(format!("qualified-resolver-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src/mymod")).unwrap();
    fs::write(dir.join("Cargo.toml"), "[package]\nname = \"mycrate\"\nversion = \"0.1.0\"\nedition = \"2024\"\n").unwrap();
    fs::write(dir.join("src/lib.rs"), "pub mod mymod;\npub mod other;\n").unwrap();
    fs::write(dir.join("src/mymod.rs"), "pub struct MyStruct;\nmod extra;\n").unwrap();
    fs::write(dir.join("src/mymod/extra.rs"), "impl super::MyStruct { pub fn f(&self) {} }\n").unwrap();
    fs::write(dir.join("src/other.rs"), "pub struct MyStruct;\n").unwrap();
    let resolver = CargoRustResolver::new(&dir).unwrap();
    for query in ["mymod::MyStruct", "mycrate::mymod::MyStruct"] {
        let paths = resolver.resolve_files(query).unwrap();
        assert_eq!(paths.len(), 2, "{query}: {paths:?}");
        assert!(paths.iter().any(|p| p.ends_with("mymod.rs")));
        assert!(paths.iter().any(|p| p.ends_with("extra.rs")));
    }
    assert_eq!(resolver.resolve_files("MyStruct").unwrap().len(), 3);
    fs::remove_dir_all(dir).unwrap();
}
