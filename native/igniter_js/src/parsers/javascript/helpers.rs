// SPDX-FileCopyrightText: 2024 Shahryar Tavakkoli
// SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs.contributors>
//
// SPDX-License-Identifier: MIT

use swc_ecma_ast::{ImportSpecifier, Module, ModuleDecl, ModuleItem};
use swc_ecma_codegen::{text_writer::JsWriter, Config, Emitter};
use swc_ecma_visit::{VisitMut, VisitMutWith};

use swc_common::{
    comments::SingleThreadedComments,
    errors::{ColorConfig, Handler},
    sync::Lrc,
    FileName, SourceMap,
};

use swc_ecma_parser::{lexer::Lexer, Parser, StringInput};

use super::dialect::Dialect;

/// Parse `file_content` as plain JavaScript.
///
/// Kept for callers that predate dialects and for the many places inside this crate where the
/// source is a fragment we generated ourselves. Everything reachable from a NIF should call
/// [`parse_as`] and pass the dialect through.
pub fn parse(
    file_content: &str,
) -> Result<(Module, SingleThreadedComments, Lrc<SourceMap>), String> {
    parse_as(file_content, Dialect::Js)
}

/// Parse `file_content` in `dialect`.
///
/// The dialect decides both the swc syntax configuration and the filename swc reports in
/// diagnostics, so a TypeScript error no longer claims to come from `virtual_file.js`.
pub fn parse_as(
    file_content: &str,
    dialect: Dialect,
) -> Result<(Module, SingleThreadedComments, Lrc<SourceMap>), String> {
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let fm = cm.new_source_file(
        FileName::Custom(dialect.virtual_file_name().into()).into(),
        file_content.to_string(),
    );

    let comments = SingleThreadedComments::default();

    let lexer = Lexer::new(
        dialect.swc_syntax(),
        Default::default(),
        StringInput::from(&*fm),
        Some(&comments),
    );

    let mut parser = Parser::new_from(lexer);

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

pub fn code_gen_from_ast_vist<T>(file_content: &str, visitor: T) -> Result<String, String>
where
    T: VisitMut,
{
    code_gen_from_ast_vist_as(file_content, visitor, Dialect::Js)
}

/// As [`code_gen_from_ast_vist`], in a given dialect.
pub fn code_gen_from_ast_vist_as<T>(
    file_content: &str,
    mut visitor: T,
    dialect: Dialect,
) -> Result<String, String>
where
    T: VisitMut,
{
    let (mut module, comments, cm) = match parse_as(file_content, dialect) {
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

/// Emit source for a module that has already been parsed and modified.
///
/// Returns a `Result`. It used to `.expect()` on both the emit and the UTF-8 conversion, which
/// panics the NIF — and a panic in a NIF takes down the calling BEAM process with a message about
/// Rust rather than an `{:error, _}` the caller can act on. Neither failure is impossible: a
/// visitor can build a module swc declines to print.
pub fn code_gen_from_ast_module(
    module: &mut Module,
    comments: SingleThreadedComments,
    cm: Lrc<SourceMap>,
) -> Result<String, String> {
    let mut buf = vec![];

    let mut emitter = Emitter {
        cfg: Config::default().with_minify(false),
        cm: cm.clone(),
        comments: Some(&comments),
        wr: JsWriter::new(cm.clone(), "\n", &mut buf, None),
    };

    emitter
        .emit_module(module)
        .map_err(|_| "Failed to emit module".to_string())?;

    String::from_utf8(buf).map_err(|_| "Invalid UTF-8 in generated source".to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn items(code: &str) -> Vec<ModuleItem> {
        parse(code).expect("test fixture must parse").0.body
    }

    fn first_item(code: &str) -> ModuleItem {
        items(code).into_iter().next().expect("expected one item")
    }

    #[test]
    fn parse_accepts_valid_source() {
        assert!(parse("const a = 1;").is_ok());
        assert!(parse("").is_ok());
    }

    #[test]
    fn parse_rejects_invalid_source() {
        match parse("const a = ;") {
            Err(reason) => assert_eq!(reason, "Failed to parse module"),
            Ok(_) => panic!("expected a parse error"),
        }

        assert!(parse("import * from;").is_err());
    }

    #[test]
    fn code_gen_reports_unparseable_source() {
        struct Noop;
        impl VisitMut for Noop {}

        assert!(code_gen_from_ast_vist("const a = ;", Noop).is_err());
        assert!(code_gen_from_ast_vist("const a = 1;", Noop).is_ok());
    }

    #[test]
    fn code_gen_preserves_comments() {
        struct Noop;
        impl VisitMut for Noop {}

        let output = code_gen_from_ast_vist("// keep me\nconst a = 1;", Noop).unwrap();
        assert!(output.contains("// keep me"), "got: {output}");
    }

    #[test]
    fn duplicate_named_import_is_detected() {
        let body = items(r#"import { Socket } from "phoenix";"#);
        let candidate = first_item(r#"import { Socket } from "phoenix";"#);

        assert!(is_duplicate_import(&candidate, &body));
    }

    #[test]
    fn duplicate_default_import_is_detected() {
        let body = items(r#"import topbar from "topbar";"#);
        let candidate = first_item(r#"import topbar from "topbar";"#);

        assert!(is_duplicate_import(&candidate, &body));
    }

    #[test]
    fn duplicate_namespace_import_is_detected() {
        let body = items(r#"import * as All from "mod";"#);
        let candidate = first_item(r#"import * as All from "mod";"#);

        assert!(is_duplicate_import(&candidate, &body));
    }

    #[test]
    fn same_source_with_a_new_specifier_is_not_duplicate() {
        let body = items(r#"import { Socket } from "phoenix";"#);
        let candidate = first_item(r#"import { Socket, LiveSocket } from "phoenix";"#);

        assert!(!is_duplicate_import(&candidate, &body));
    }

    #[test]
    fn different_source_is_not_duplicate() {
        let body = items(r#"import { Socket } from "phoenix";"#);
        let candidate = first_item(r#"import { Socket } from "other";"#);

        assert!(!is_duplicate_import(&candidate, &body));
    }

    /// Specifier kinds must match, so a default and a named import that share a
    /// local name are still distinct.
    #[test]
    fn default_and_named_specifiers_are_distinct() {
        let body = items(r#"import { Socket } from "phoenix";"#);
        let candidate = first_item(r#"import Socket from "phoenix";"#);

        assert!(!is_duplicate_import(&candidate, &body));
    }

    #[test]
    fn non_import_items_are_never_duplicates() {
        let body = items(r#"import { Socket } from "phoenix";"#);
        let candidate = first_item("const a = 1;");

        assert!(!is_duplicate_import(&candidate, &body));
    }

    #[test]
    fn empty_body_has_no_duplicates() {
        let candidate = first_item(r#"import { Socket } from "phoenix";"#);

        assert!(!is_duplicate_import(&candidate, &[]));
    }

    #[test]
    fn replace_four_spaces_with_tab_converts_indentation() {
        assert_eq!(replace_four_spaces_with_tab("    a"), "\ta");
        assert_eq!(replace_four_spaces_with_tab("        a"), "\t\ta");
        assert_eq!(replace_four_spaces_with_tab("  a"), "  a");
        assert_eq!(replace_four_spaces_with_tab(""), "");
    }
}
