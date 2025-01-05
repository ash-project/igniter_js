//! # ASTStatistics Module
//!
//! This module provides functionality to parse JavaScript source code and
//! extract statistics about specific AST nodes,
//! such as functions, classes, debugger statements, imports, try-catch blocks, and throw statements.
//! It is designed to work with Elixir via the `rustler` library.

#![allow(clippy::print_stdout)]
use crate::parsers::javascript::ast::source_to_ast;
use oxc::{
    allocator::Allocator,
    ast::{
        ast::{
            Class, DebuggerStatement, Function, ImportDeclaration, ThrowStatement, TryStatement,
        },
        visit::walk,
        Visit,
    },
    syntax::scope::ScopeFlags,
};

use rustler::{NifStruct, NifTaggedEnum};

#[derive(NifTaggedEnum)]
pub enum ASTStatisticsResultType {
    Statistics(ASTStatistics),
    Error(String),
}

/// Represents the statistics gathered from JavaScript source code.
/// Each field corresponds to a specific type of AST node.
#[derive(Debug, Default, NifStruct)]
#[module = "IgniterJs.Native.Parsers.Javascript.ASTStatistics"]
pub struct ASTStatistics {
    functions: usize,
    classes: usize,
    debuggers: usize,
    imports: usize,
    trys: usize,
    throws: usize,
}

/// Parses the given JavaScript source code and collects statistics about the AST nodes.
///
/// # Arguments
/// - `file_content`: A string slice containing the JavaScript source code.
/// - `allocator`: A reference to the allocator used for AST node allocation.
///
/// # Returns
/// A result containing `ASTStatistics` with statistics about the parsed source code or an error message if parsing fails.
///
/// # Example
/// ```rust
/// let allocator = Allocator::default();
/// let file_content = "function foo() { console.log('Hello'); }";
/// let result = source_visitor(file_content, &allocator);
/// assert!(result.is_ok());
/// ```
pub fn source_visitor(file_content: &str, allocator: &Allocator) -> Result<ASTStatistics, String> {
    let parsed = source_to_ast(file_content, allocator)?;

    if let Some(errors) = parsed.errors.first() {
        return Err(format!("Failed to parse source: {:?}", errors));
    }

    let mut ast_pass = ASTStatistics::default();
    ast_pass.visit_program(&parsed.program);
    Ok(ast_pass)
}

impl<'a> Visit<'a> for ASTStatistics {
    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        self.functions += 1;
        walk::walk_function(self, func, flags);
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        self.classes += 1;
        walk::walk_class(self, class);
    }

    fn visit_debugger_statement(&mut self, it: &DebuggerStatement) {
        self.debuggers += 1;
        walk::walk_debugger_statement(self, it);
    }

    fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
        self.imports += 1;
        walk::walk_import_declaration(self, it);
    }

    fn visit_try_statement(&mut self, it: &TryStatement<'a>) {
        self.trys += 1;
        walk::walk_try_statement(self, it);
    }

    fn visit_throw_statement(&mut self, it: &ThrowStatement<'a>) {
        self.throws += 1;
        walk::walk_throw_statement(self, it);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc::allocator::Allocator;

    fn create_allocator<'a>() -> &'a Allocator {
        let allocator = Box::new(Allocator::default());
        Box::leak(allocator)
    }

    #[test]
    fn test_source_visitor() {
        let allocator = create_allocator();
        let file_content = r#"
            import { foo } from 'bar';
            import * as jar from 'jar';
            console.log('Start JS file');
            class Foo {
                constructor() {
                    debugger;
                    console.log('Hello');
                }
            }
            function bar() {
                console.log('World');
                debugger;
            }
        "#;
        let parsed = source_visitor(file_content, &allocator).unwrap();
        println!("{:#?}", parsed);
        assert_eq!(parsed.functions, 2);
        assert_eq!(parsed.classes, 1);
        assert_eq!(parsed.debuggers, 2);
        assert_eq!(parsed.imports, 2);
        assert_eq!(parsed.trys, 0);
        assert_eq!(parsed.throws, 0);
    }
}
