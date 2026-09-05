// SPDX-FileCopyrightText: 2024 Shahryar Tavakkoli
// SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs.contributors>
//
// SPDX-License-Identifier: MIT

use std::collections::HashSet;

use crate::atoms;
use crate::helpers::encode_response;
use crate::parsers::javascript::ast::*;
use crate::parsers::javascript::ast_json::convert_ast_to_estree_as;
use crate::parsers::javascript::dialect::Dialect;
use crate::parsers::javascript::phoenix::*;
use rustler::{Env, NifResult, NifStruct, NifTaggedEnum, Term};

/// Resolve a dialect name coming from Elixir.
///
/// The Elixir side always sends one of `"js"`, `"jsx"`, `"ts"`, `"tsx"`, defaulting to `"js"`, so
/// an unrecognised value means the caller asked for something this crate does not support. Saying
/// so beats parsing their TypeScript as JavaScript and reporting a syntax error they cannot
/// explain.
fn dialect_or_error(name: &str) -> Result<Dialect, String> {
    Dialect::from_name(name)
}

#[rustler::nif]
pub fn is_module_imported_from_ast_nif(
    env: Env,
    file_content: String,
    module_name: String,
    dialect: String,
) -> NifResult<Term> {
    let fn_atom = atoms::is_module_imported_from_ast_nif();

    let (status, result) = match dialect_or_error(&dialect) {
        Err(_) => (atoms::error(), false),
        Ok(dialect) => match is_module_imported_from_ast(&file_content, &module_name, dialect) {
            Ok(true) => (atoms::ok(), true),
            _ => (atoms::error(), false),
        },
    };

    encode_response(env, status, fn_atom, result)
}

#[rustler::nif]
pub fn insert_import_to_ast_nif(
    env: Env,
    file_content: String,
    import_lines: String,
    dialect: String,
) -> NifResult<Term> {
    let (status, result) = match dialect_or_error(&dialect)
        .and_then(|dialect| insert_import_to_ast(&file_content, &import_lines, dialect))
    {
        Ok(updated_code) => (atoms::ok(), updated_code),
        Err(error_msg) => (atoms::error(), error_msg),
    };

    encode_response(env, status, atoms::insert_import_to_ast_nif(), result)
}

#[rustler::nif]
fn remove_import_from_ast_nif(
    env: Env,
    file_content: String,
    modules: String,
    dialect: String,
) -> NifResult<Term> {
    let (status, result) = match dialect_or_error(&dialect)
        .and_then(|dialect| remove_import_from_ast(&file_content, &modules, dialect))
    {
        Ok(updated_code) => (atoms::ok(), updated_code),
        Err(error_msg) => (atoms::error(), error_msg),
    };

    encode_response(env, status, atoms::remove_import_from_ast_nif(), result)
}

#[rustler::nif]
pub fn find_live_socket_node_from_ast_nif(env: Env, file_content: String) -> NifResult<Term> {
    let fn_atom = atoms::find_live_socket_node_from_ast();

    let (status, result) = match find_live_socket_node_from_ast(&file_content) {
        Ok(true) => (atoms::ok(), true),
        _ => (atoms::error(), false),
    };

    encode_response(env, status, fn_atom, result)
}

#[rustler::nif]
pub fn contains_variable_from_ast_nif(
    env: Env,
    file_content: String,
    variable_name: String,
    dialect: String,
) -> NifResult<Term> {
    let fn_atom = atoms::contains_variable_from_ast_nif();

    let (status, result) = match dialect_or_error(&dialect) {
        Err(_) => (atoms::error(), false),
        Ok(dialect) => match contains_variable_from_ast(&file_content, &variable_name, dialect) {
            Ok(true) => (atoms::ok(), true),
            _ => (atoms::error(), false),
        },
    };

    encode_response(env, status, fn_atom, result)
}

#[rustler::nif]
pub fn extend_hook_object_to_ast_nif(
    env: Env,
    file_content: String,
    names: Vec<String>,
) -> NifResult<Term> {
    let unique_names: HashSet<String> = names.into_iter().collect();
    let mut vec_of_strs: Vec<&str> = unique_names.iter().map(|s| s.as_str()).collect();
    vec_of_strs.sort();
    let (status, result) = match extend_hook_object_to_ast(&file_content, vec_of_strs) {
        Ok(updated_code) => (atoms::ok(), updated_code),
        Err(error_msg) => (atoms::error(), error_msg),
    };

    encode_response(env, status, atoms::extend_hook_object_to_ast_nif(), result)
}

#[rustler::nif]
fn remove_objects_of_hooks_from_ast_nif(
    env: Env,
    file_content: String,
    object_names: Vec<String>,
) -> NifResult<Term> {
    let fn_atom = atoms::remove_objects_of_hooks_from_ast_nif();
    let vec_of_strs: Vec<&str> = object_names.iter().map(|s| s.as_str()).collect();
    let (status, result) = match remove_objects_of_hooks_from_ast(&file_content, vec_of_strs) {
        Ok(updated_code) => (atoms::ok(), updated_code),
        Err(error_msg) => (atoms::error(), error_msg),
    };

    encode_response(env, status, fn_atom, result)
}

#[derive(Debug, NifStruct)]
#[module = "IgniterJs.Native.Parsers.Javascript.ASTStatisticsResult"]
pub struct ASTStatisticsResult {
    pub functions: usize,
    pub classes: usize,
    pub debuggers: usize,
    pub imports: usize,
    pub trys: usize,
    pub throws: usize,
}

#[derive(Debug, NifTaggedEnum)]
pub enum ASTStatisticsResultType {
    Statistics(ASTStatisticsResult),
    Error(String),
}

#[rustler::nif]
fn statistics_from_ast_nif(env: Env, file_content: String, dialect: String) -> NifResult<Term> {
    let fn_atom = atoms::statistics_from_ast_nif();

    let (status, result) = match dialect_or_error(&dialect)
        .and_then(|dialect| statistics_from_ast(&file_content, dialect))
    {
        Ok(updated_code) => (
            atoms::ok(),
            ASTStatisticsResultType::Statistics(ASTStatisticsResult {
                imports: updated_code.imports,
                classes: updated_code.classes,
                debuggers: updated_code.debuggers,
                functions: updated_code.functions,
                throws: updated_code.throws,
                trys: updated_code.trys,
            }),
        ),
        Err(error_msg) => (atoms::error(), ASTStatisticsResultType::Error(error_msg)),
    };

    encode_response(env, status, fn_atom, result)
}

#[rustler::nif]
pub fn extend_var_object_property_by_names_to_ast_nif(
    env: Env,
    file_content: String,
    var_name: String,
    object_names: Vec<String>,
    dialect: String,
) -> NifResult<Term> {
    let unique_names: HashSet<String> = object_names.into_iter().collect();
    let mut vec_of_strs: Vec<&str> = unique_names.iter().map(|s| s.as_str()).collect();
    vec_of_strs.sort();

    let (status, result) = match dialect_or_error(&dialect).and_then(|dialect| {
        extend_var_object_property_by_names_to_ast(&file_content, &var_name, vec_of_strs, dialect)
    }) {
        Ok(updated_code) => (atoms::ok(), updated_code),
        Err(error_msg) => (atoms::error(), error_msg),
    };

    encode_response(
        env,
        status,
        atoms::extend_var_object_property_by_names_to_ast_nif(),
        result,
    )
}

#[rustler::nif]
pub fn convert_ast_to_estree_nif(
    env: Env,
    file_content: String,
    dialect: String,
) -> NifResult<Term> {
    let (status, result) = match dialect_or_error(&dialect)
        .and_then(|dialect| convert_ast_to_estree_as(&file_content, dialect))
    {
        Ok(updated_code) => (atoms::ok(), updated_code),
        Err(error_msg) => (atoms::error(), error_msg),
    };

    encode_response(env, status, atoms::convert_ast_to_estree_nif(), result)
}

#[rustler::nif]
pub fn insert_ast_at_index_nif(
    env: Env,
    file_content: String,
    insert_code: String,
    index: usize,
    dialect: String,
) -> NifResult<Term> {
    let (status, result) = match dialect_or_error(&dialect)
        .and_then(|dialect| insert_ast_at_index(&file_content, &insert_code, index, dialect))
    {
        Ok(updated_code) => (atoms::ok(), updated_code),
        Err(error_msg) => (atoms::error(), error_msg),
    };

    encode_response(env, status, atoms::insert_ast_at_index_nif(), result)
}

#[rustler::nif]
pub fn replace_ast_at_index_nif(
    env: Env,
    file_content: String,
    replace_code: String,
    index: usize,
    dialect: String,
) -> NifResult<Term> {
    let (status, result) = match dialect_or_error(&dialect)
        .and_then(|dialect| replace_ast_at_index(&file_content, &replace_code, index, dialect))
    {
        Ok(updated_code) => (atoms::ok(), updated_code),
        Err(error_msg) => (atoms::error(), error_msg),
    };

    encode_response(env, status, atoms::replace_ast_at_index_nif(), result)
}
