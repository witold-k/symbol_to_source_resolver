// SPDX-License-Identifier: Apache-2.0

use crate::SymbolResolver;

/// Placeholder for Rust symbol resolution using Cargo project metadata.
pub(crate) struct CargoRustResolver;

impl SymbolResolver for CargoRustResolver {
    fn resolve(&self, _symbol: &str) -> String {
        String::new()
    }
}
