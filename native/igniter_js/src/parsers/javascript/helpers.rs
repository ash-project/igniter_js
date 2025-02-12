use swc_ecma_ast::{ImportSpecifier, Module, ModuleDecl, ModuleItem};
use swc_ecma_codegen::{text_writer::JsWriter, Config, Emitter};
use swc_ecma_visit::{VisitMut, VisitMutWith};

use swc_common::{
    comments::SingleThreadedComments,
    errors::{ColorConfig, Handler},
    sync::Lrc,
    FileName, SourceMap,
};

use swc_ecma_parser::{lexer::Lexer, Capturing, Parser, StringInput, Syntax};

pub fn parse(
    file_content: &str,
) -> Result<(Module, SingleThreadedComments, Lrc<SourceMap>), String> {
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let fm = cm.new_source_file(
        FileName::Custom("virtual_file.js".into()).into(),
        file_content.into(),
    );

    let comments = SingleThreadedComments::default();

    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        StringInput::from(&*fm),
        Some(&comments),
    );

    let capturing = Capturing::new(lexer);

    let mut parser = Parser::new_from(capturing);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    // let module = parser.parse_module().expect("Failed to parse module");
    let module = match parser.parse_module() {
        Ok(m) => m,
        Err(_e) => {
            return Err("Failed to parse module".to_string());
        }
    };

    Ok((module, comments, cm))
}

pub fn code_gen_from_ast_vist<T>(file_content: &str, mut visitor: T) -> Result<String, String>
where
    T: VisitMut,
{
    let (mut module, comments, cm) = match parse(file_content) {
        Ok(result) => result,
        Err(_) => return Err("Failed to parse JavaScript content".to_string()),
    };

    module.visit_mut_with(&mut visitor);
    let mut buf = vec![];

    let mut emitter = Emitter {
        cfg: Config::default().with_minify(false),
        cm: cm.clone(),
        comments: Some(&comments),
        wr: JsWriter::new(cm.clone(), "\n", &mut buf, None),
    };

    if emitter.emit_module(&module).is_err() {
        return Err("Failed to emit module".to_string());
    }

    String::from_utf8(buf).map_err(|_| "Invalid UTF-8".to_string())
}

pub fn code_gen_from_ast_module(
    module: &mut Module,
    comments: SingleThreadedComments,
    cm: Lrc<SourceMap>,
) -> String {
    let mut buf = vec![];

    let mut emitter = Emitter {
        cfg: Config::default().with_minify(false),
        cm: cm.clone(),
        comments: Some(&comments),
        wr: JsWriter::new(cm.clone(), "\n", &mut buf, None),
    };

    emitter.emit_module(module).expect("Failed to emit module");
    String::from_utf8(buf).expect("Invalid UTF-8")
}

pub fn is_duplicate_import(new_import: &ModuleItem, body: &[ModuleItem]) -> bool {
    if let ModuleItem::ModuleDecl(ModuleDecl::Import(new_import_decl)) = new_import {
        for item in body {
            if let ModuleItem::ModuleDecl(ModuleDecl::Import(existing_import_decl)) = item {
                if new_import_decl.src.value == existing_import_decl.src.value {
                    for new_spec in &new_import_decl.specifiers {
                        if !existing_import_decl
                            .specifiers
                            .iter()
                            .any(|existing_spec| specifier_equals(new_spec, existing_spec))
                        {
                            return false;
                        }
                    }
                    return true;
                }
            }
        }
    }
    false
}

fn specifier_equals(new_spec: &ImportSpecifier, existing_spec: &ImportSpecifier) -> bool {
    match (new_spec, existing_spec) {
        (ImportSpecifier::Named(new_named), ImportSpecifier::Named(existing_named)) => {
            new_named.local.sym == existing_named.local.sym
        }
        (ImportSpecifier::Default(new_named), ImportSpecifier::Default(existing_named)) => {
            new_named.local.sym == existing_named.local.sym
        }
        (ImportSpecifier::Namespace(new_named), ImportSpecifier::Namespace(existing_named)) => {
            new_named.local.sym == existing_named.local.sym
        }
        _ => false,
    }
}

pub fn replace_four_spaces_with_tab(input: &str) -> String {
    input.replace("    ", "\t")
}
