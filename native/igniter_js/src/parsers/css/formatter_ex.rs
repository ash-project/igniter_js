use crate::atoms;
use crate::helpers::encode_response;
use crate::parsers::css::formatter::*;

use rustler::{Env, NifResult, Term};

#[rustler::nif]
pub fn format_css_nif(env: Env, file_content: String) -> NifResult<Term> {
    let fn_atom = atoms::format_css_nif();
    let (status, result) = match format(&file_content) {
        Ok(updated_code) => (atoms::ok(), updated_code),
        Err(error_msg) => (atoms::error(), error_msg),
    };

    encode_response(env, status, fn_atom, result)
}

#[rustler::nif]
pub fn is_css_formatted_nif(env: Env, file_content: String) -> NifResult<Term> {
    let fn_atom = atoms::is_css_formatted_nif();
    let (status, result) = match is_formatted(&file_content) {
        Ok(true) => (atoms::ok(), true),
        _ => (atoms::error(), false),
    };

    encode_response(env, status, fn_atom, result)
}
