// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use std::fs;
use symbol_to_source_resolver::maven_java::MavenJavaResolver;

#[test]
fn resolves_types_without_pom_or_compilation() {
    let root = std::env::temp_dir().join(format!("maven-tree-sitter-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let src = root.join("module/src/main/java/org/example");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("Foo.java"), r#"package org.example;
        // class Fake {}
        public class Foo { class Inner {} }
        interface Bar {}
        String text = "class Imaginary {}";
        class Broken {
    "#).unwrap();
    fs::write(src.join("Other.java"), "package org.example; public record Other(int x) {}").unwrap();
    let resolver = MavenJavaResolver::new(&root).unwrap();
    for symbol in ["Foo", "org.example.Foo", "Foo.Inner", "org.example.Foo.Inner", "Bar", "Other"] {
        assert!(!resolver.resolve_files(symbol).unwrap().is_empty(), "{symbol}");
    }
    for symbol in ["Fake", "Imaginary"] {
        assert!(resolver.resolve_files(symbol).unwrap().is_empty(), "{symbol}");
    }
    fs::remove_dir_all(root).unwrap();
}
