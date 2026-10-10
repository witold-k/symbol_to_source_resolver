// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

//! Resolve source-code symbols using a selectable language/build-system backend.

pub mod cargo_rust;
mod compile_commands_clang;
mod maven_java;

use cargo_rust::CargoRustResolver;
use compile_commands_clang::CompileCommandsClangResolver;
use maven_java::MavenJavaResolver;
use std::path::Path;

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
    /// Creates a resolver for the current working directory.
    ///
    /// Use `with_project_dir` to select a different Cargo project.
    pub fn new(backend: Backend) -> Self {
        Self::with_project_dir(backend, ".")
            .expect("failed to initialize resolver")
    }

    /// Creates a resolver for a specified project directory.
    pub fn with_project_dir(
        backend: Backend,
        project_dir: impl AsRef<Path>,
    ) -> Result<Self, cargo_metadata::Error> {
        let implementation = match backend {
            Backend::CargoRust => ResolverImpl::CargoRust(CargoRustResolver::new(project_dir)?),
            Backend::MavenJava => ResolverImpl::MavenJava(MavenJavaResolver),
            Backend::CompileCommandsClang => {
                ResolverImpl::CompileCommandsClang(CompileCommandsClangResolver)
            }
        };
        Ok(Self { implementation })
    }

    pub fn resolve(&self, symbol: &str) -> String {
        match &self.implementation {
            ResolverImpl::CargoRust(resolver) => resolver.resolve(symbol),
            ResolverImpl::MavenJava(resolver) => resolver.resolve(symbol),
            ResolverImpl::CompileCommandsClang(resolver) => resolver.resolve(symbol),
        }
    }
}
