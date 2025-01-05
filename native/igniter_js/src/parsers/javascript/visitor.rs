#![allow(clippy::print_stdout)]
use crate::parsers::javascript::ast::source_to_ast;
use oxc::{
    allocator::Allocator,
    ast::{
        ast::{Class, Function, TSImportType},
        visit::walk,
        Visit,
    },
    syntax::scope::ScopeFlags,
};

use rustler::NifStruct;

#[derive(Debug, Default, NifStruct)]
#[module = "Elixir.IgniterJs.Native.Parsers.Javascript.Visitor.CountASTNodes"]
pub struct CountASTNodes {
    functions: usize,
    classes: usize,
    ts_import_types: usize,
}

pub fn source_visitor<'a>(
    file_content: &str,
    allocator: &Allocator,
) -> Result<CountASTNodes, Box<dyn std::error::Error>> {
    let parsed = source_to_ast(file_content, allocator)?;

    let mut ast_pass = CountASTNodes::default();
    ast_pass.visit_program(&parsed.program);
    println!("{ast_pass:?}");
    Ok(ast_pass)
}

impl<'a> Visit<'a> for CountASTNodes {
    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        self.functions += 1;
        walk::walk_function(self, func, flags);
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        self.classes += 1;
        walk::walk_class(self, class);
    }

    fn visit_ts_import_type(&mut self, ty: &TSImportType<'a>) {
        self.ts_import_types += 1;
        walk::walk_ts_import_type(self, ty);
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
            class Foo {
                constructor() {
                    console.log('Hello');
                }
            }
            function bar() {
                console.log('World');
            }
        "#;
        assert!(source_visitor(file_content, &allocator).is_ok());
    }
}
