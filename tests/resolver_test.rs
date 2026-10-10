// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use symbol_to_source_resolver::{Backend, Resolver};

#[test]
fn placeholder_backends_return_empty_strings() {
    for backend in [
        Backend::CargoRust,
        Backend::MavenJava,
        Backend::CompileCommandsClang,
    ] {
        let resolver = Resolver::new(backend);
        assert_eq!(resolver.resolve("some::qualified::Symbol"), "");
    }
}
