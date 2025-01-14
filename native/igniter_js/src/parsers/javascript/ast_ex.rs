use std::collections::HashSet;

use crate::atoms;
use crate::helpers::encode_response;
use crate::parsers::javascript::ast::*;
use crate::parsers::javascript::phoenix::*;
use rustler::{Env, NifResult, NifStruct, NifTaggedEnum, Term};

#[rustler::nif]
pub fn is_module_imported_from_ast_nif(
    env: Env,
    file_content: String,
    module_name: String,
) -> NifResult<Term> {
    let fn_atom = atoms::is_module_imported_from_ast_nif();
    let (status, result) = match is_module_imported_from_ast(&file_content, &module_name) {
        Ok(true) => (atoms::ok(), true),
        _ => (atoms::error(), false),
    };

    encode_response(env, status, fn_atom, result)
}

#[rustler::nif]
pub fn insert_import_to_ast_nif(
    env: Env,
    file_content: String,
    import_lines: String,
) -> NifResult<Term> {
    let (status, result) = match insert_import_to_ast(&file_content, &import_lines) {
        Ok(updated_code) => (atoms::ok(), updated_code),
        Err(error_msg) => (atoms::error(), error_msg),
    };

    encode_response(env, status, atoms::insert_import_to_ast_nif(), result)
}

#[rustler::nif]
fn remove_import_from_ast_nif(env: Env, file_content: String, modules: String) -> NifResult<Term> {
    let (status, result) = match remove_import_from_ast(&file_content, &modules) {
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
) -> NifResult<Term> {
    let fn_atom = atoms::contains_variable_from_ast_nif();

    let (status, result) = match contains_variable_from_ast(&file_content, &variable_name) {
        Ok(true) => (atoms::ok(), true),
        _ => (atoms::error(), false),
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
fn statistics_from_ast_nif(env: Env, file_content: String) -> NifResult<Term> {
    let fn_atom = atoms::statistics_from_ast_nif();

    let (status, result) = match statistics_from_ast(&file_content) {
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
) -> NifResult<Term> {
    let unique_names: HashSet<String> = object_names.into_iter().collect();
    let mut vec_of_strs: Vec<&str> = unique_names.iter().map(|s| s.as_str()).collect();
    vec_of_strs.sort();

    let (status, result) =
        match extend_var_object_property_by_names_to_ast(&file_content, &var_name, vec_of_strs) {
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
