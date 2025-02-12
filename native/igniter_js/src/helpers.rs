//! Helper functions for encoding and formatting responses.
//!
//! This module provides utility functions for encoding consistent responses
//! in Elixir NIFs using Rust. It leverages the Rustler library for seamless
//! integration with the Erlang VM.

use rustler::{Encoder, Env, NifResult, Term};

/// Encodes a response into an Erlang term.
///
/// This function takes a status, a source, and a message, and encodes them into
/// a tuple format that can be passed to the Erlang/Elixir runtime.
///
/// # Arguments
///
/// - `env`: The environment in which the term is created. This is required to interact
///   with the Erlang/Elixir runtime.
/// - `status`: An atom representing the status of the response (e.g., `:ok` or `:error`).
/// - `source`: An atom representing the source or context of the response, which can help
///   identify where the response originated.
/// - `message`: A generic type `T` representing the message or payload of the response.
///   This must implement the `Encoder` trait to allow encoding into an Erlang term.
///
/// # Returns
///
/// Returns a `NifResult` containing the encoded tuple `(status, source, message)`
/// as a `Term` that can be sent to the Erlang/Elixir runtime.
///
/// # Example
///
/// ```rust
/// use rustler::{Atom, Env, Encoder, NifResult};
///
/// fn example(env: Env) -> NifResult<Term> {
///     let status = Atom::from_str(env, "ok").unwrap();
///     let source = Atom::from_str(env, "parser").unwrap();
///     let message = "Parsing completed successfully";
///
///     encode_response(env, status, source, message)
/// }
/// ```
///
/// This function is useful for building consistent response formats
/// when integrating Rust code with Elixir applications.
pub fn encode_response<T>(
    env: Env<'_>,
    status: rustler::types::atom::Atom,
    source: rustler::types::atom::Atom,
    message: T,
) -> NifResult<Term<'_>>
where
    T: Encoder,
{
    Ok((status, source, message).encode(env))
}
