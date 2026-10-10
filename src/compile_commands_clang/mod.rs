// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use crate::SymbolResolver;

/// Placeholder for C/C++ symbol resolution using compile_commands.json and Clang.
pub(crate) struct CompileCommandsClangResolver;

impl SymbolResolver for CompileCommandsClangResolver {
    fn resolve(&self, _symbol: &str) -> String {
        String::new()
    }
}
