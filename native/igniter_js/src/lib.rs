// SPDX-FileCopyrightText: 2024 Shahryar Tavakkoli
//
// SPDX-License-Identifier: MIT

pub mod atoms;
pub mod helpers;
pub mod parsers {
    pub mod css;
    pub mod javascript;
}

rustler::init!("Elixir.IgniterJs.Native");
