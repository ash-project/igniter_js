use crate::atoms;
use crate::helpers::encode_response;
use crate::parsers::javascript::ast::*;
use crate::parsers::javascript::ast_statistics::{source_visitor, ASTStatisticsResultType};
use oxc::allocator::Allocator;
use rustler::{Env, NifResult, Term};

// TODO: We are not going to use it for now, until the OXC supports Json ast
// #[rustler::nif]
// fn source_to_ast_nif(env: Env, file_content: String) -> NifResult<Term> {
//     let allocator = Allocator::default(); // Create an OXC allocator
//     match source_to_ast(&file_content, &allocator) {
//         Ok(ast) => {
//             // Attempt to serialize the AST into JSON
//             let ast_json_result: Result<Value, String> =
//                 to_value(&ast).map_err(|e| format!("Serialization error: {}", e));

//             let (status, message) = match ast_json_result {
//                 // If serialization is successful, modify the JSON object and return it
//                 Ok(mut ast_json) => {
//                     if let Value::Object(ref mut map) = ast_json {
//                         map.insert(
//                             "source_text".to_string(),
//                             Value::String(ast.source_text.to_string()),
//                         );
//                     }

//                     (atoms::ok(), ast_json.to_string())
//                 }
//                 // If serialization fails, send the error message to Elixir
//                 Err(serialization_error) => (atoms::error(), serialization_error),
//             };

//             encode_response(env, status, atoms::source_to_ast_nif(), message)
//         }
//         // If the source_to_ast function fails, send the error message to Elixir
//         Err(msg) => encode_response(env, atoms::error(), atoms::source_to_ast_nif(), msg),
//     }
// }

#[rustler::nif]
pub fn is_module_imported_from_ast_nif(
    env: Env,
    file_content: String,
    module_name: String,
) -> NifResult<Term> {
    let allocator = Allocator::default(); // Create an OXC allocator
    let fn_atom = atoms::is_module_imported_from_ast_nif();
    let (status, result) =
        match is_module_imported_from_ast(&file_content, &module_name, &allocator) {
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
    let allocator = Allocator::default(); // Create an OXC allocator
    let (status, result) = match insert_import_to_ast(&file_content, &import_lines, &allocator) {
        Ok(updated_code) => (atoms::ok(), updated_code),
        Err(error_msg) => (atoms::error(), error_msg),
    };

    encode_response(env, status, atoms::insert_import_to_ast_nif(), result)
}

#[rustler::nif]
fn remove_import_from_ast_nif(
    env: Env,
    file_content: String,
    modules: Vec<String>,
) -> NifResult<Term> {
    let allocator = Allocator::default(); // Create an OXC allocator
    let names_iter = modules.iter().map(|s| s.as_str());
    let (status, result) = match remove_import_from_ast(&file_content, names_iter, &allocator) {
        Ok(updated_code) => (atoms::ok(), updated_code),
        Err(error_msg) => (atoms::error(), error_msg),
    };

    encode_response(env, status, atoms::remove_import_from_ast_nif(), result)
}

#[rustler::nif]
pub fn find_live_socket_node_from_ast_nif(env: Env, file_content: String) -> NifResult<Term> {
    let allocator = Allocator::default(); // Create an OXC allocator
    let ast = source_to_ast(&file_content, &allocator);
    let fn_atom = atoms::find_live_socket_node_from_ast();
    if ast.is_err() {
        let msg = "Invalid JS file.";
        return encode_response(env, atoms::error(), fn_atom, msg);
    }

    let (status, result) = match find_live_socket_node_from_ast(&ast.unwrap()) {
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
    let allocator = Allocator::default(); // Create an OXC allocator
    let names_iter = names.iter().map(|s| s.as_str());
    let (status, result) = match extend_hook_object_to_ast(&file_content, names_iter, &allocator) {
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
    let allocator = Allocator::default(); // Create an OXC allocator
    let fn_atom = atoms::remove_objects_of_hooks_from_ast_nif();
    let names_iter = object_names.iter().map(|s| s.as_str());
    let (status, result) =
        match remove_objects_of_hooks_from_ast(&file_content, names_iter, &allocator) {
            Ok(updated_code) => (atoms::ok(), updated_code),
            Err(error_msg) => (atoms::error(), error_msg),
        };

    encode_response(env, status, fn_atom, result)
}

#[rustler::nif]
fn statistics_from_ast_nif(env: Env, file_content: String) -> NifResult<Term> {
    let allocator = Allocator::default(); // Create an OXC allocator
    let fn_atom = atoms::statistics_from_ast_nif();

    let (status, result) = match source_visitor(&file_content, &allocator) {
        Ok(updated_code) => (
            atoms::ok(),
            ASTStatisticsResultType::Statistics(updated_code),
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
    let allocator = Allocator::default(); // Create an OXC allocator
    let names_iter = object_names.iter().map(|s| s.as_str());
    let (status, result) = match extend_var_object_property_by_names_to_ast(
        &file_content,
        &var_name,
        names_iter,
        &allocator,
    ) {
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
