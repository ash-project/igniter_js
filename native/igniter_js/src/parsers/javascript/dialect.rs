// SPDX-FileCopyrightText: 2024 Shahryar Tavakkoli
// SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs/contributors>
//
// SPDX-License-Identifier: MIT

//! Which flavour of JavaScript a source is.
//!
//! Every parser in this crate used to hardcode plain ECMAScript: the codemods parsed
//! `Syntax::Es(Default::default())` under a virtual filename of `virtual_file.js`, the ESTree
//! dump asked oxc for `SourceType::from_path("example.js")`, and the formatter used
//! `JsFileSource::default()`. Three separate places, all saying "this is a .js file".
//!
//! That made every TypeScript and every JSX source unparseable — not badly parsed, but rejected
//! outright, so a caller could not so much as insert an import into a `.tsx` file.
//!
//! All three underlying libraries already understand the other dialects. This type is the single
//! place that says which one, and converts it for each of them.
//!
//! ## Compatibility
//!
//! [`Dialect::Js`] is the default everywhere the caller does not say otherwise, and it produces
//! exactly the configuration this crate used before. Anything that parsed before parses the same
//! way now. TypeScript is opt-in, either by naming the dialect or by passing a path whose
//! extension implies it.

use biome_js_syntax::{JsFileSource, ModuleKind};
use oxc_span::SourceType;
use swc_ecma_parser::{EsSyntax, Syntax, TsSyntax};

/// A JavaScript dialect: plain ES, ES with JSX, TypeScript, or TypeScript with JSX.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Dialect {
    /// Plain ECMAScript. The default, and what every caller got before dialects existed.
    #[default]
    Js,
    /// ECMAScript with JSX elements.
    Jsx,
    /// TypeScript without JSX. In this mode `<T>expr` is a type assertion.
    Ts,
    /// TypeScript with JSX. In this mode `<T>expr` is a JSX element, not an assertion — which is
    /// why `Ts` and `Tsx` cannot be collapsed into one.
    Tsx,
}

impl Dialect {
    /// Parse a dialect name.
    ///
    /// Accepts `"js"`, `"jsx"`, `"ts"` and `"tsx"`, case-insensitively, with or without a leading
    /// dot. Anything else is an error rather than a silent fallback: guessing on a name we do not
    /// recognise is how a `.mts` file would quietly get parsed as plain JS and fail confusingly.
    pub fn from_name(name: &str) -> Result<Self, String> {
        match name.trim().trim_start_matches('.').to_ascii_lowercase().as_str() {
            "js" | "mjs" | "cjs" => Ok(Dialect::Js),
            "jsx" => Ok(Dialect::Jsx),
            "ts" | "mts" | "cts" => Ok(Dialect::Ts),
            "tsx" => Ok(Dialect::Tsx),
            other => Err(format!(
                "unknown dialect {other:?}: expected one of js, jsx, ts, tsx"
            )),
        }
    }

    /// The dialect implied by a file path's extension, defaulting to [`Dialect::Js`].
    ///
    /// A path is the most natural way to ask, and it is what the `:path` mode of the Elixir API
    /// uses — reading `app.tsx` and then parsing it as `.js` would be absurd.
    pub fn from_path(path: &str) -> Self {
        match path.rsplit('.').next() {
            Some(ext) => Dialect::from_name(ext).unwrap_or(Dialect::Js),
            None => Dialect::Js,
        }
    }

    /// Is this dialect TypeScript?
    pub fn typescript(self) -> bool {
        matches!(self, Dialect::Ts | Dialect::Tsx)
    }

    /// Does this dialect allow JSX elements?
    pub fn jsx(self) -> bool {
        matches!(self, Dialect::Jsx | Dialect::Tsx)
    }

    /// The swc parser configuration — used by every codemod in this crate.
    pub fn swc_syntax(self) -> Syntax {
        if self.typescript() {
            Syntax::Typescript(TsSyntax {
                tsx: self.jsx(),
                ..Default::default()
            })
        } else {
            Syntax::Es(EsSyntax {
                jsx: self.jsx(),
                ..Default::default()
            })
        }
    }

    /// The filename swc reports in diagnostics.
    ///
    /// swc does not infer syntax from this — [`Dialect::swc_syntax`] does that — but the name
    /// appears in error messages, and `virtual_file.js` on a TypeScript error is a small lie that
    /// costs somebody ten minutes.
    pub fn virtual_file_name(self) -> &'static str {
        match self {
            Dialect::Js => "virtual_file.js",
            Dialect::Jsx => "virtual_file.jsx",
            Dialect::Ts => "virtual_file.ts",
            Dialect::Tsx => "virtual_file.tsx",
        }
    }

    /// The oxc source type — used by the ESTree conversion.
    pub fn oxc_source_type(self) -> SourceType {
        match self {
            Dialect::Js => SourceType::mjs(),
            Dialect::Jsx => SourceType::jsx(),
            Dialect::Ts => SourceType::ts(),
            Dialect::Tsx => SourceType::tsx(),
        }
    }

    /// The biome file source — used by the formatter.
    pub fn biome_source(self) -> JsFileSource {
        let source = match self {
            Dialect::Js => JsFileSource::js_module(),
            Dialect::Jsx => JsFileSource::jsx(),
            Dialect::Ts => JsFileSource::ts(),
            Dialect::Tsx => JsFileSource::tsx(),
        };

        source.with_module_kind(ModuleKind::Module)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        assert_eq!(Dialect::from_name("js"), Ok(Dialect::Js));
        assert_eq!(Dialect::from_name("jsx"), Ok(Dialect::Jsx));
        assert_eq!(Dialect::from_name("ts"), Ok(Dialect::Ts));
        assert_eq!(Dialect::from_name("tsx"), Ok(Dialect::Tsx));
    }

    #[test]
    fn names_are_forgiving_about_case_and_leading_dots() {
        assert_eq!(Dialect::from_name(".TSX"), Ok(Dialect::Tsx));
        assert_eq!(Dialect::from_name("  ts  "), Ok(Dialect::Ts));
    }

    #[test]
    fn module_variants_map_to_their_base_dialect() {
        assert_eq!(Dialect::from_name("mjs"), Ok(Dialect::Js));
        assert_eq!(Dialect::from_name("cjs"), Ok(Dialect::Js));
        assert_eq!(Dialect::from_name("mts"), Ok(Dialect::Ts));
        assert_eq!(Dialect::from_name("cts"), Ok(Dialect::Ts));
    }

    #[test]
    fn an_unknown_name_is_an_error_not_a_guess() {
        assert!(Dialect::from_name("coffee").is_err());
        assert!(Dialect::from_name("").is_err());
    }

    #[test]
    fn paths_are_read_from_their_extension() {
        assert_eq!(Dialect::from_path("vite.config.ts"), Dialect::Ts);
        assert_eq!(Dialect::from_path("src/main.tsx"), Dialect::Tsx);
        assert_eq!(Dialect::from_path("app.js"), Dialect::Js);
        assert_eq!(Dialect::from_path("Component.jsx"), Dialect::Jsx);
    }

    // A path is a convenience, so it falls back rather than failing — unlike an explicit name,
    // where the caller said something specific and deserves to be told it was not understood.
    #[test]
    fn an_unknown_extension_falls_back_to_js() {
        assert_eq!(Dialect::from_path("Makefile"), Dialect::Js);
        assert_eq!(Dialect::from_path("styles.css"), Dialect::Js);
    }

    #[test]
    fn the_default_is_plain_javascript() {
        assert_eq!(Dialect::default(), Dialect::Js);
        assert_eq!(Dialect::default().virtual_file_name(), "virtual_file.js");
    }

    #[test]
    fn ts_and_tsx_are_distinct_because_angle_brackets_mean_different_things() {
        assert!(Dialect::Ts.typescript() && !Dialect::Ts.jsx());
        assert!(Dialect::Tsx.typescript() && Dialect::Tsx.jsx());
    }

    #[test]
    fn each_dialect_names_its_own_virtual_file() {
        for d in [Dialect::Js, Dialect::Jsx, Dialect::Ts, Dialect::Tsx] {
            assert!(d.virtual_file_name().starts_with("virtual_file."));
        }
    }
}
