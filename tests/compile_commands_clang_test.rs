// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use std::fs;
use symbol_to_source_resolver::compile_commands_clang::CompileCommandsClangResolver;

#[test]
fn resolves_cpp_declarations_despite_syntax_errors() {
    let root = std::env::temp_dir().join(format!("clang-resolver-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("sample.cpp"), "namespace demo { class Widget {}; int execute() { return 1; } }\nint broken( {\n").unwrap();
    let db = serde_json::json!([{"directory":root,"file":"sample.cpp","command":"clang++ -c sample.cpp"}]);
    fs::write(root.join("compile_commands.json"), db.to_string()).unwrap();
    let resolver = CompileCommandsClangResolver::new(&root).unwrap();
    assert_eq!(resolver.resolve_files("demo::Widget").len(), 1);
    assert_eq!(resolver.resolve_files("execute").len(), 1);
    assert!(resolver.resolve_files("Missing").is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn discovers_project_headers_not_listed_in_compilation_database() {
    let root = std::env::temp_dir().join(format!("clang-headers-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("include/nested")).unwrap();
    fs::write(root.join("sample.cpp"), "#include \"nested/widget.hpp\"\n").unwrap();
    fs::write(root.join("include/nested/widget.hpp"), "namespace demo { struct HeaderWidget {}; }\n").unwrap();
    let db = serde_json::json!([{"directory":root,"file":"sample.cpp","command":"clang++ -c sample.cpp"}]);
    fs::write(root.join("compile_commands.json"), db.to_string()).unwrap();
    let resolver = CompileCommandsClangResolver::new(&root).unwrap();
    let found = resolver.resolve_files("demo::HeaderWidget");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].ends_with("widget.hpp"));
    fs::remove_dir_all(root).unwrap();
}
