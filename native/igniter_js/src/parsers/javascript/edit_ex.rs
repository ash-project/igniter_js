// SPDX-FileCopyrightText: 2024 Shahryar Tavakkoli
// SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs.contributors>
//
// SPDX-License-Identifier: MIT

//! The NIF entry points of [`super::edit`].
//!
//! Two functions rather than one per edit: the request travels as JSON, so a new kind of edit is a
//! new variant in [`super::edit::Request`] and a function in the Elixir module, with no new NIF to
//! register, precompile and keep in step across ten targets.

use crate::atoms;
use crate::helpers::encode_response;
use crate::parsers::javascript::edit::{edit, query, Query, Request};
use rustler::{Env, NifResult, Term};

#[rustler::nif]
pub fn edit_source_nif(
    env: Env,
    file_content: String,
    dialect: String,
    request: String,
) -> NifResult<Term> {
    let (status, result) = match serde_json::from_str::<Request>(&request)
        .map_err(|e| format!("invalid edit request: {e}"))
        .and_then(|request| edit(&file_content, &dialect, &request))
    {
        Ok(updated_code) => (atoms::ok(), updated_code),
        Err(error_msg) => (atoms::error(), error_msg),
    };

    encode_response(env, status, atoms::edit_source_nif(), result)
}

#[rustler::nif]
pub fn query_source_nif(
    env: Env,
    file_content: String,
    dialect: String,
    request: String,
) -> NifResult<Term> {
    let (status, result) = match serde_json::from_str::<Query>(&request)
        .map_err(|e| format!("invalid query: {e}"))
        .and_then(|request| query(&file_content, &dialect, &request))
    {
        Ok(answer) => (atoms::ok(), answer),
        Err(error_msg) => (atoms::error(), error_msg),
    };

    encode_response(env, status, atoms::query_source_nif(), result)
}
