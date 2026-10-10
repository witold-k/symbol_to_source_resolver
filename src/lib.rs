// SPDX-License-Identifier: Apache-2.0
//! Resolve source-code symbols using a selectable language/build-system backend.

mod cargo_rust;
mod compile_commands_clang;
mod maven_java;

use cargo_rust::CargoRustResolver;
use compile_commands_clang::CompileCommandsClangResolver;
use maven_java::MavenJavaResolver;

/// Available symbol resolution backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Backend {
    CargoRust,
    MavenJava,
    CompileCommandsClang,
}

/// Common interface for resolving a symbol to source code.
pub trait SymbolResolver {
    fn resolve(&self, symbol: &str) -> String;
}

/// A symbol resolver configured for a particular backend.
pub struct Resolver {
    implementation: ResolverImpl,
}

enum ResolverImpl {
    CargoRust(CargoRustResolver),
    MavenJava(MavenJavaResolver),
    CompileCommandsClang(CompileCommandsClangResolver),
}

impl Resolver {
    pub fn new(backend: Backend) -> Self {
        let implementation = match backend {
            Backend::CargoRust => ResolverImpl::CargoRust(CargoRustResolver),
            Backend::MavenJava => ResolverImpl::MavenJava(MavenJavaResolver),
            Backend::CompileCommandsClang => {
                ResolverImpl::CompileCommandsClang(CompileCommandsClangResolver)
            }
        };
        Self { implementation }
    }

    pub fn resolve(&self, symbol: &str) -> String {
        match &self.implementation {
            ResolverImpl::CargoRust(resolver) => resolver.resolve(symbol),
            ResolverImpl::MavenJava(resolver) => resolver.resolve(symbol),
            ResolverImpl::CompileCommandsClang(resolver) => resolver.resolve(symbol),
        }
    }
}
