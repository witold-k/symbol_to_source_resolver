// SPDX-License-Identifier: Apache-2.0

use crate::SymbolResolver;

/// Placeholder for Java symbol resolution using Maven project metadata.
pub(crate) struct MavenJavaResolver;

impl SymbolResolver for MavenJavaResolver {
    fn resolve(&self, _symbol: &str) -> String {
        String::new()
    }
}
