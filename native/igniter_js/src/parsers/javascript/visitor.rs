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

use rustler::NifStruct;

#[derive(Debug, Default, NifStruct)]
#[module = "Elixir.IgniterJs.Native.Parsers.Javascript.Visitor.ASTNodesInfo"]
pub struct ASTNodesInfo {
    functions: usize,
    classes: usize,
    debuggers: usize,
    imports: usize,
    trys: usize,
    throws: usize,
}

pub fn source_visitor<'a>(
    file_content: &str,
    allocator: &Allocator,
) -> Result<ASTNodesInfo, String> {
    let parsed = source_to_ast(file_content, allocator)?;

    if let Some(errors) = parsed.errors.first() {
        return Err(format!("Failed to parse source: {:?}", errors));
    }

    let mut ast_pass = ASTNodesInfo::default();
    ast_pass.visit_program(&parsed.program);
    println!("{ast_pass:?}");
    Ok(ast_pass)
}

impl<'a> Visit<'a> for ASTNodesInfo {
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
        assert!(source_visitor(file_content, &allocator).is_ok());
    }
}
