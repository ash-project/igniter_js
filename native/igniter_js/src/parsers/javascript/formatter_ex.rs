use crate::atoms;
use crate::helpers::encode_response;
use crate::parsers::javascript::formatter::*;

use rustler::{Env, NifResult, Term};

#[rustler::nif]
pub fn format_js_nif(env: Env, file_content: String) -> NifResult<Term> {
    let fn_atom = atoms::format_js_nif();
    let (status, result) = match format(&file_content) {
        Ok(updated_code) => (atoms::ok(), updated_code),
        Err(error_msg) => (atoms::error(), error_msg),
    };

    encode_response(env, status, fn_atom, result)
}

#[rustler::nif]
pub fn is_js_formatted_nif(env: Env, file_content: String) -> NifResult<Term> {
    let fn_atom = atoms::is_js_formatted_nif();
    let (status, result) = match is_formatted(&file_content) {
        Ok(true) => (atoms::ok(), true),
        _ => (atoms::error(), false),
    };

    encode_response(env, status, fn_atom, result)
}
