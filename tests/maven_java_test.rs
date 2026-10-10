// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use std::fs;
use symbol_to_source_resolver::maven_java::MavenJavaResolver;

#[test]
fn finds_java_types_in_nested_maven_module() {
    let dir = std::env::temp_dir().join(format!("maven-resolver-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("module/src/main/java/org/example")).unwrap();
    fs::create_dir_all(dir.join("module/target/src/main/java/org/example")).unwrap();
    fs::write(dir.join("pom.xml"), "<project/>").unwrap();
    fs::write(dir.join("module/src/main/java/org/example/Foo.java"), "package org.example; // class Fake {}\n public class Foo {} interface Bar {}").unwrap();
    fs::write(dir.join("module/target/src/main/java/org/example/Bad.java"), "package org.example; class Foo {}").unwrap();
    let resolver = MavenJavaResolver::new(&dir).unwrap();
    for symbol in ["Foo", "example.Foo", "org.example.Foo", "Bar"] {
        let files = resolver.resolve_files(symbol).unwrap();
        assert_eq!(files.len(), 1, "{symbol}: {files:?}");
        assert!(files[0].ends_with("Foo.java"));
    }
    assert!(resolver.resolve_files("Fake").unwrap().is_empty());
    fs::remove_dir_all(dir).unwrap();
}
