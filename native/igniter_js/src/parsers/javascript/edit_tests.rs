// SPDX-FileCopyrightText: 2024 Shahryar Tavakkoli
// SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs.contributors>
//
// SPDX-License-Identifier: MIT

//! Tests for [`super::edit`].
//!
//! Every edit is checked three ways: it produces exactly the text it should, it leaves the rest of
//! the file byte-for-byte alone, and running it again changes nothing.
//!
//! ## Non-ASCII text
//!
//! The text that broke the first version of this feature was not ASCII: a comment in Persian before
//! the edit point put the edit that many bytes early. These tests cover right-to-left text, a
//! zero-width non-joiner, and an astral character (which is two UTF-16 units and four UTF-8 bytes,
//! so it tells the two counts apart). They are written with `\u{…}` escapes so the lib's source
//! stays ASCII; the sentinel strings below stand for "a short word in Persian" and so on.

use super::edit::*;

/// "salam" in Persian: six UTF-8 bytes, three UTF-16 units.
const FA: &str = "\u{0633}\u{0644}\u{0627}\u{0645}";
/// A word with a zero-width non-joiner in the middle, as Persian writes plurals.
const ZWNJ: &str = "\u{0645}\u{06CC}\u{200C}\u{062E}\u{0648}\u{0627}\u{0647}\u{0645}";
/// Outside the BMP: two UTF-16 units, four UTF-8 bytes.
const ASTRAL: &str = "\u{1F9FE}";

// ── helpers ─────────────────────────────────────────────────────────────────

fn member(key: &str, code: &str) -> Member {
    Member {
        key: key.into(),
        code: code.into(),
    }
}

fn prop(name: &str, value: Option<&str>) -> Prop {
    Prop {
        name: name.into(),
        value: value.map(String::from),
    }
}

fn var(name: &str) -> Target {
    Target::Variable { name: name.into() }
}

fn call(callee: &str, index: usize) -> Target {
    Target::CallArgument {
        callee: callee.into(),
        index,
    }
}

fn paths(path: &[&str]) -> Vec<String> {
    path.iter().map(|s| s.to_string()).collect()
}

fn merge_with(
    dialect: &str,
    source: &str,
    target: Target,
    path: &[&str],
    container: Container,
    members: Vec<Member>,
    mode: Mode,
) -> Result<String, String> {
    edit(
        source,
        dialect,
        &Request::MergeObjectMembers {
            target,
            path: paths(path),
            container,
            members,
            mode,
        },
    )
}

fn merge(source: &str, target: Target, path: &[&str], members: Vec<Member>) -> String {
    merge_with(
        "ts",
        source,
        target,
        path,
        Container::Object,
        members,
        Mode::Add,
    )
    .unwrap()
}

fn merge_array(source: &str, target: Target, path: &[&str], items: &[&str]) -> String {
    merge_with(
        "ts",
        source,
        target,
        path,
        Container::Array,
        items
            .iter()
            .map(|i| member(i, &format!("\"{i}\"")))
            .collect(),
        Mode::Add,
    )
    .unwrap()
}

fn tsx(source: &str, request: Request) -> Result<String, String> {
    edit(source, "tsx", &request)
}

fn wrap(source: &str, element: &str, wrapper: &str, props: Vec<Prop>) -> Result<String, String> {
    tsx(
        source,
        Request::WrapJsx {
            element: element.into(),
            wrapper: wrapper.into(),
            props,
        },
    )
}

fn unwrap(source: &str, element: &str) -> Result<String, String> {
    tsx(
        source,
        Request::UnwrapJsx {
            element: element.into(),
        },
    )
}

fn props(source: &str, element: &str, props: Vec<Prop>, mode: Mode) -> Result<String, String> {
    tsx(
        source,
        Request::AddJsxProps {
            element: element.into(),
            props,
            mode,
        },
    )
}

fn ensure(source: &str, module: &str, names: &[&str]) -> String {
    edit(
        source,
        "ts",
        &Request::EnsureImports {
            module: module.into(),
            names: names.iter().map(|s| s.to_string()).collect(),
        },
    )
    .unwrap()
}

fn move_to(source: &str, from: &str, to: &str, names: &[&str]) -> String {
    edit(
        source,
        "ts",
        &Request::MoveImports {
            from: from.into(),
            to: to.into(),
            names: names.iter().map(|s| s.to_string()).collect(),
        },
    )
    .unwrap()
}

fn members_of(
    source: &str,
    dialect: &str,
    target: Target,
    path: &[&str],
) -> Result<Vec<String>, String> {
    let answer = query(
        source,
        dialect,
        &Query::ObjectMembers {
            target,
            path: paths(path),
            container: Container::Object,
        },
    )?;
    Ok(serde_json::from_str(&answer).unwrap())
}

// ── object members ──────────────────────────────────────────────────────────

mod objects {
    use super::*;

    #[test]
    fn into_an_inline_object() {
        let out = merge(
            "export const t = { a: 1 }\n",
            var("t"),
            &[],
            vec![member("b", "b: 2")],
        );
        assert_eq!(out, "export const t = { b: 2, a: 1 }\n");
    }

    #[test]
    fn into_an_empty_object() {
        let out = merge(
            "export const t = {}\n",
            var("t"),
            &[],
            vec![member("b", "b: 2")],
        );
        assert_eq!(out, "export const t = { b: 2 }\n");
    }

    #[test]
    fn several_members_keep_their_order() {
        let out = merge(
            "const t = { a: 1 }\n",
            var("t"),
            &[],
            vec![member("x", "x: 1"), member("y", "y: 2")],
        );
        assert_eq!(out, "const t = { x: 1, y: 2, a: 1 }\n");
    }

    #[test]
    fn into_a_multiline_object_on_its_own_line_with_the_same_indent() {
        let src = "const t = {\n    a: 1,\n    c: 3,\n}\n";
        let out = merge(src, var("t"), &[], vec![member("b", "b: 2")]);
        assert_eq!(out, "const t = {\n    b: 2,\n    a: 1,\n    c: 3,\n}\n");
    }

    #[test]
    fn into_an_empty_multiline_object() {
        let out = merge("const t = {\n}\n", var("t"), &[], vec![member("b", "b: 2")]);
        assert_eq!(out, "const t = {\n  b: 2,\n}\n");
    }

    #[test]
    fn tabs_are_kept() {
        let src = "const t = {\n\ta: 1,\n}\n";
        let out = merge(src, var("t"), &[], vec![member("b", "b: 2")]);
        assert_eq!(out, "const t = {\n\tb: 2,\n\ta: 1,\n}\n");
    }

    #[test]
    fn a_comment_above_a_member_stays_above_it() {
        let src = "export default {\n  // keep me\n  plugins: [],\n}\n";
        let out = merge(
            src,
            Target::DefaultExport,
            &[],
            vec![member("base", "base: \"/\"")],
        );
        assert_eq!(
            out,
            "export default {\n  base: \"/\",\n  // keep me\n  plugins: [],\n}\n"
        );
    }

    #[test]
    fn a_comment_after_the_opening_brace_keeps_its_line() {
        let src = "const t = { // the texts\n  a: 1,\n}\n";
        let out = merge(src, var("t"), &[], vec![member("b", "b: 2")]);
        assert_eq!(out, "const t = { // the texts\n  b: 2,\n  a: 1,\n}\n");
    }

    #[test]
    fn block_comments_and_trailing_comments_survive() {
        let src = "const t = {\n  /* first */ a: 1, // one\n  b: 2, /* two */\n}\n";
        let out = merge(src, var("t"), &[], vec![member("c", "c: 3")]);
        assert!(out.contains("/* first */ a: 1, // one"), "{out}");
        assert!(out.contains("b: 2, /* two */"), "{out}");
        assert!(out.contains("c: 3,"), "{out}");
    }

    #[test]
    fn crlf_files_stay_crlf() {
        let src = "const t = {\r\n  a: 1,\r\n}\r\n";
        let out = merge(src, var("t"), &[], vec![member("b", "b: 2")]);
        assert_eq!(out, "const t = {\r\n  b: 2,\r\n  a: 1,\r\n}\r\n");
        assert!(!out.contains("\n\n") && !out.replace("\r\n", "").contains('\n'));
    }

    #[test]
    fn an_existing_key_is_left_alone() {
        let src = "const t = { a: 1 }\n";
        let out = merge(src, var("t"), &[], vec![member("a", "a: 99")]);
        assert_eq!(out, src);
    }

    #[test]
    fn a_quoted_key_matches_its_bare_name() {
        let src = "const t = { \"a-b\": 1, 'c': 2 }\n";
        assert_eq!(
            merge(src, var("t"), &[], vec![member("a-b", "\"a-b\": 9")]),
            src
        );
        assert_eq!(merge(src, var("t"), &[], vec![member("c", "c: 9")]), src);
    }

    #[test]
    fn a_shorthand_member_counts_as_present() {
        let src = "const fa = {}\nexport const translations = { fa } as const\n";
        let out = merge(
            src,
            var("translations"),
            &[],
            vec![member("fa", "fa: {}"), member("en", "en: {}")],
        );
        assert_eq!(
            out,
            "const fa = {}\nexport const translations = { en: {}, fa } as const\n"
        );
    }

    #[test]
    fn computed_keys_and_spreads_are_not_mistaken_for_names() {
        let src = "const t = { [k]: 1, ...rest }\n";
        let out = merge(src, var("t"), &[], vec![member("k", "k: 2")]);
        assert_eq!(out, "const t = { k: 2, [k]: 1, ...rest }\n");
    }

    #[test]
    fn set_mode_replaces_a_member_and_leaves_an_equal_one() {
        let src = "const t = { a: \"./x\", b: 2 }\n";
        let out = merge_with(
            "ts",
            src,
            var("t"),
            &[],
            Container::Object,
            vec![member("a", "a: \"/abs/x\"")],
            Mode::Set,
        )
        .unwrap();
        assert_eq!(out, "const t = { a: \"/abs/x\", b: 2 }\n");
        let same = merge_with(
            "ts",
            &out,
            var("t"),
            &[],
            Container::Object,
            vec![member("a", "a: \"/abs/x\"")],
            Mode::Set,
        )
        .unwrap();
        assert_eq!(same, out);
    }

    #[test]
    fn set_mode_adds_what_is_missing() {
        let out = merge_with(
            "ts",
            "const t = { a: 1 }\n",
            var("t"),
            &[],
            Container::Object,
            vec![member("a", "a: 2"), member("b", "b: 3")],
            Mode::Set,
        )
        .unwrap();
        assert_eq!(out, "const t = { b: 3, a: 2 }\n");
    }

    #[test]
    fn plain_javascript_works_too() {
        let out = merge_with(
            "js",
            "const t = { a: 1 }\n",
            var("t"),
            &[],
            Container::Object,
            vec![member("b", "b: 2")],
            Mode::Add,
        )
        .unwrap();
        assert_eq!(out, "const t = { b: 2, a: 1 }\n");
    }

    #[test]
    fn a_source_that_does_not_parse_is_an_error_not_a_guess() {
        let err = merge_with(
            "ts",
            "const t = { a: ",
            var("t"),
            &[],
            Container::Object,
            vec![member("b", "b: 2")],
            Mode::Add,
        )
        .unwrap_err();
        assert!(err.contains("could not parse"), "{err}");
    }

    #[test]
    fn an_unknown_dialect_is_an_error() {
        assert!(merge_with(
            "coffee",
            "a = 1",
            var("a"),
            &[],
            Container::Object,
            vec![],
            Mode::Add
        )
        .is_err());
    }
}

// ── paths ───────────────────────────────────────────────────────────────────

mod paths_into_objects {
    use super::*;

    #[test]
    fn into_a_nested_object_that_exists() {
        let src = "export default {\n  resolve: {\n    alias: { a: 1 },\n  },\n}\n";
        let out = merge(
            src,
            Target::DefaultExport,
            &["resolve", "alias"],
            vec![member("b", "b: 2")],
        );
        assert_eq!(
            out,
            "export default {\n  resolve: {\n    alias: { b: 2, a: 1 },\n  },\n}\n"
        );
    }

    #[test]
    fn a_missing_leaf_is_created() {
        let src = "export default {\n  resolve: {\n    dedupe: [],\n  },\n}\n";
        let out = merge(
            src,
            Target::DefaultExport,
            &["resolve", "alias"],
            vec![member("a", "a: 1")],
        );
        assert_eq!(
            out,
            "export default {\n  resolve: {\n    alias: {\n      a: 1,\n    },\n    dedupe: [],\n  },\n}\n"
        );
    }

    #[test]
    fn every_missing_level_is_created() {
        let src = "export default {\n  plugins: [],\n}\n";
        let out = merge(
            src,
            Target::DefaultExport,
            &["resolve", "alias"],
            vec![member("a", "a: 1")],
        );
        assert_eq!(
            out,
            "export default {\n  resolve: {\n    alias: {\n      a: 1,\n    },\n  },\n  plugins: [],\n}\n"
        );
    }

    #[test]
    fn creating_levels_in_a_one_line_object_puts_the_rest_on_its_own_line() {
        let out = merge(
            "export default { plugins: [] }\n",
            Target::DefaultExport,
            &["resolve", "alias"],
            vec![member("a", "a: 1")],
        );
        assert_eq!(
            out,
            "export default {\n  resolve: {\n    alias: {\n      a: 1,\n    },\n  },\n  plugins: [] }\n"
        );
    }

    #[test]
    fn a_created_level_is_idempotent() {
        let once = merge(
            "export default {\n  plugins: [],\n}\n",
            Target::DefaultExport,
            &["resolve", "alias"],
            vec![member("a", "a: 1")],
        );
        let twice = merge(
            &once,
            Target::DefaultExport,
            &["resolve", "alias"],
            vec![member("a", "a: 1")],
        );
        assert_eq!(once, twice);
    }

    #[test]
    fn a_level_that_is_not_a_literal_is_an_error() {
        let src = "const shared = {}\nexport default { resolve: shared }\n";
        let err = merge_with(
            "ts",
            src,
            Target::DefaultExport,
            &["resolve", "alias"],
            Container::Object,
            vec![member("a", "a: 1")],
            Mode::Add,
        )
        .unwrap_err();
        assert!(err.contains("`resolve`"), "{err}");
    }

    #[test]
    fn keys_that_are_not_identifiers_are_quoted_when_created() {
        let out = merge(
            "export default {\n}\n",
            Target::DefaultExport,
            &["@scope/x"],
            vec![member("a", "a: 1")],
        );
        assert!(out.contains("\"@scope/x\": {"), "{out}");
    }
}

// ── targets ─────────────────────────────────────────────────────────────────

mod targets {
    use super::*;

    #[test]
    fn a_variable_seen_through_as_const_satisfies_and_parentheses() {
        for src in [
            "export const t = { a: 1 } as const\n",
            "export const t = { a: 1 } satisfies Record<string, number>\n",
            "export const t = ({ a: 1 })\n",
            "export const t: Record<string, number> = { a: 1 }\n",
        ] {
            let out = merge(src, var("t"), &[], vec![member("b", "b: 2")]);
            assert!(out.contains("b: 2, a: 1"), "{src} -> {out}");
        }
    }

    #[test]
    fn a_missing_variable_is_an_error_naming_it() {
        let err = merge_with(
            "ts",
            "const a = {}\n",
            var("nope"),
            &[],
            Container::Object,
            vec![],
            Mode::Add,
        )
        .unwrap_err();
        assert!(err.contains("nope"), "{err}");
    }

    #[test]
    fn the_first_argument_of_a_call() {
        let src = "export const rsk = createRsk({ adapter: A })\n";
        let out = merge(
            src,
            call("createRsk", 0),
            &[],
            vec![member("i18n", "i18n: x")],
        );
        assert_eq!(
            out,
            "export const rsk = createRsk({ i18n: x, adapter: A })\n"
        );
    }

    #[test]
    fn a_later_argument_and_a_method_call() {
        let src = "const x = lib.make(1, { a: 1 })\n";
        let out = merge(src, call("make", 1), &[], vec![member("b", "b: 2")]);
        assert_eq!(out, "const x = lib.make(1, { b: 2, a: 1 })\n");
    }

    #[test]
    fn a_call_with_no_object_argument_is_an_error() {
        assert!(merge_with(
            "ts",
            "createRsk(1)\n",
            call("createRsk", 0),
            &[],
            Container::Object,
            vec![],
            Mode::Add
        )
        .is_err());
        assert!(merge_with(
            "ts",
            "other({})\n",
            call("createRsk", 0),
            &[],
            Container::Object,
            vec![],
            Mode::Add
        )
        .is_err());
    }

    #[test]
    fn default_export_forms_of_a_vite_config() {
        for src in [
            "export default { a: 1 }\n",
            "export default defineConfig({ a: 1 })\n",
            "export default defineConfig(() => ({ a: 1 }))\n",
            "export default defineConfig(({ mode }) => ({ a: 1 }))\n",
            "export default defineConfig(({ mode }) => {\n  return { a: 1 }\n})\n",
            "export default defineConfig(async function () {\n  return { a: 1 }\n})\n",
            "const config = { a: 1 }\nexport default config\n",
            "export default defineConfig({ a: 1 } as const)\n",
        ] {
            let out = merge(src, Target::DefaultExport, &[], vec![member("b", "b: 2")]);
            assert!(out.contains("b: 2"), "{src} -> {out}");
        }
    }

    #[test]
    fn no_default_export_is_an_error() {
        assert!(merge_with(
            "ts",
            "export const a = {}\n",
            Target::DefaultExport,
            &[],
            Container::Object,
            vec![],
            Mode::Add
        )
        .is_err());
    }

    #[test]
    fn imports_before_the_config_are_left_alone() {
        let src = "import react from '@vitejs/plugin-react'\nimport { defineConfig } from 'vite'\n\n// https://vite.dev/config/\nexport default defineConfig({\n  plugins: [react()],\n})\n";
        let out = merge(src, Target::DefaultExport, &["resolve", "dedupe"], vec![]);
        assert!(
            out.starts_with(
                "import react from '@vitejs/plugin-react'\nimport { defineConfig } from 'vite'\n"
            ),
            "{out}"
        );
        assert!(out.contains("// https://vite.dev/config/"), "{out}");
    }
}

// ── arrays ──────────────────────────────────────────────────────────────────

mod arrays {
    use super::*;

    #[test]
    fn into_an_inline_array() {
        let out = merge_array("const a = [\"react\"]\n", var("a"), &[], &["zod"]);
        assert_eq!(out, "const a = [\"zod\", \"react\"]\n");
    }

    #[test]
    fn into_an_empty_array() {
        assert_eq!(
            merge_array("const a = []\n", var("a"), &[], &["zod"]),
            "const a = [\"zod\"]\n"
        );
        assert_eq!(
            merge_array("const a = [ ]\n", var("a"), &[], &["zod"]),
            "const a = [\"zod\"]\n"
        );
    }

    #[test]
    fn into_a_multiline_array() {
        let src = "const a = [\n  \"react\",\n  \"react-dom\",\n]\n";
        assert_eq!(
            merge_array(src, var("a"), &[], &["zod"]),
            "const a = [\n  \"zod\",\n  \"react\",\n  \"react-dom\",\n]\n"
        );
    }

    #[test]
    fn an_element_already_there_is_not_added_again() {
        let src = "const a = [\"react\", 'zod']\n";
        assert_eq!(merge_array(src, var("a"), &[], &["react", "zod"]), src);
    }

    #[test]
    fn a_nested_array_by_path() {
        let src = "export default {\n  resolve: {\n    dedupe: [\"react\"],\n  },\n}\n";
        let out = merge_array(src, Target::DefaultExport, &["resolve", "dedupe"], &["zod"]);
        assert!(out.contains("dedupe: [\"zod\", \"react\"]"), "{out}");
    }

    #[test]
    fn a_missing_array_is_created() {
        let out = merge_array(
            "export default {\n  a: 1,\n}\n",
            Target::DefaultExport,
            &["resolve", "dedupe"],
            &["react", "zod"],
        );
        assert_eq!(
            out,
            "export default {\n  resolve: {\n    dedupe: [\n      \"react\",\n      \"zod\",\n    ],\n  },\n  a: 1,\n}\n"
        );
    }

    #[test]
    fn asking_for_an_array_where_there_is_an_object_is_an_error() {
        let err = merge_with(
            "ts",
            "const a = {}\n",
            var("a"),
            &[],
            Container::Array,
            vec![member("x", "\"x\"")],
            Mode::Add,
        )
        .unwrap_err();
        assert!(err.contains("array"), "{err}");
    }
}

// ── JSON ────────────────────────────────────────────────────────────────────

mod json {
    use super::*;

    fn doc_merge(
        source: &str,
        path: &[&str],
        container: Container,
        members: Vec<Member>,
    ) -> String {
        merge_with(
            "json",
            source,
            Target::Document,
            path,
            container,
            members,
            Mode::Add,
        )
        .unwrap()
    }

    #[test]
    fn a_new_top_level_key_keeps_every_other_key_where_it_was() {
        let src = "{\n  \"name\": \"app\",\n  \"private\": true,\n  \"dependencies\": {}\n}\n";
        let out = doc_merge(
            src,
            &[],
            Container::Object,
            vec![member("workspaces", "\"workspaces\": [\"vendor/@rsk/*\"]")],
        );
        assert_eq!(
            out,
            "{\n  \"workspaces\": [\"vendor/@rsk/*\"],\n  \"name\": \"app\",\n  \"private\": true,\n  \"dependencies\": {}\n}\n"
        );
    }

    #[test]
    fn an_element_into_an_existing_array() {
        let src = "{\n  \"workspaces\": [\n    \"apps/*\"\n  ]\n}\n";
        let out = doc_merge(
            src,
            &["workspaces"],
            Container::Array,
            vec![member("vendor/@rsk/*", "\"vendor/@rsk/*\"")],
        );
        assert_eq!(
            out,
            "{\n  \"workspaces\": [\n    \"vendor/@rsk/*\",\n    \"apps/*\"\n  ]\n}\n"
        );
    }

    #[test]
    fn the_element_already_there_is_not_added() {
        let src = "{\"workspaces\":[\"vendor/@rsk/*\"]}";
        assert_eq!(
            doc_merge(
                src,
                &["workspaces"],
                Container::Array,
                vec![member("vendor/@rsk/*", "\"vendor/@rsk/*\"")]
            ),
            src
        );
    }

    #[test]
    fn comments_and_trailing_commas_in_tsconfig_survive() {
        let src = "{\n  \"compilerOptions\": {\n    /* Bundler mode */\n    \"moduleResolution\": \"bundler\",\n    \"strict\": true,\n  },\n  // the files\n  \"include\": [\"src\"],\n}\n";
        let out = doc_merge(
            src,
            &["compilerOptions", "paths"],
            Container::Object,
            vec![member(
                "@rsk/core",
                "\"@rsk/core\": [\"./vendor/@rsk/core/src/index.ts\"]",
            )],
        );
        assert!(out.contains("/* Bundler mode */"), "{out}");
        assert!(out.contains("// the files"), "{out}");
        assert!(
            out.contains("\"@rsk/core\": [\"./vendor/@rsk/core/src/index.ts\"]"),
            "{out}"
        );
        assert!(out.contains("\"moduleResolution\": \"bundler\""), "{out}");
    }

    #[test]
    fn four_space_indentation_is_followed() {
        let src = "{\n    \"name\": \"app\"\n}\n";
        let out = doc_merge(
            src,
            &[],
            Container::Object,
            vec![member("type", "\"type\": \"module\"")],
        );
        assert_eq!(
            out,
            "{\n    \"type\": \"module\",\n    \"name\": \"app\"\n}\n"
        );
    }

    #[test]
    fn a_missing_path_in_json_is_created_with_quoted_keys_where_needed() {
        let src = "{\n  \"name\": \"app\"\n}\n";
        let out = doc_merge(
            src,
            &["compilerOptions", "paths"],
            Container::Object,
            vec![member("@rsk/core", "\"@rsk/core\": [\"./x\"]")],
        );
        assert!(out.contains("\"compilerOptions\": {"), "{out}");
        assert!(out.contains("\"paths\": {"), "{out}");
    }

    #[test]
    fn an_edit_twice_changes_nothing() {
        let src = "{\n  \"name\": \"app\"\n}\n";
        let once = doc_merge(
            src,
            &["workspaces"],
            Container::Array,
            vec![member("v", "\"v\"")],
        );
        let twice = doc_merge(
            &once,
            &["workspaces"],
            Container::Array,
            vec![member("v", "\"v\"")],
        );
        assert_eq!(once, twice);
    }

    #[test]
    fn invalid_json_is_an_error() {
        assert!(merge_with(
            "json",
            "{ \"a\": ",
            Target::Document,
            &[],
            Container::Object,
            vec![],
            Mode::Add
        )
        .is_err());
    }

    #[test]
    fn only_the_document_target_applies_to_json() {
        assert!(merge_with(
            "json",
            "{}",
            var("x"),
            &[],
            Container::Object,
            vec![],
            Mode::Add
        )
        .is_err());
    }
}

// ── non-ASCII text ──────────────────────────────────────────────────────────

mod non_ascii {
    use super::*;

    #[test]
    fn right_to_left_comment_before_the_node() {
        let src = format!("// {FA}\nexport const t = {{ fa }}\nconst fa = {{}}\n");
        let out = merge(&src, var("t"), &[], vec![member("en", "en: {}")]);
        assert_eq!(
            out,
            format!("// {FA}\nexport const t = {{ en: {{}}, fa }}\nconst fa = {{}}\n")
        );
    }

    #[test]
    fn zero_width_non_joiner_before_the_node() {
        let src = format!("const x = \"{ZWNJ}\"\nexport const t = {{ a: 1 }}\n");
        let out = merge(&src, var("t"), &[], vec![member("b", "b: 2")]);
        assert_eq!(
            out,
            format!("const x = \"{ZWNJ}\"\nexport const t = {{ b: 2, a: 1 }}\n")
        );
    }

    #[test]
    fn an_astral_character_counts_four_bytes_and_two_units() {
        let src = format!("// {ASTRAL}{ASTRAL}\nexport const t = {{ a: 1 }}\n");
        let out = merge(&src, var("t"), &[], vec![member("b", "b: 2")]);
        assert_eq!(
            out,
            format!("// {ASTRAL}{ASTRAL}\nexport const t = {{ b: 2, a: 1 }}\n")
        );
    }

    #[test]
    fn text_inside_the_object_and_after_the_edit_is_untouched() {
        let src =
            format!("const t = {{\n  a: \"{FA}\", // {ZWNJ}\n  b: \"{ASTRAL}\",\n}}\n// {FA}\n");
        let out = merge(&src, var("t"), &[], vec![member("c", "c: 3")]);
        assert_eq!(out, format!("const t = {{\n  c: 3,\n  a: \"{FA}\", // {ZWNJ}\n  b: \"{ASTRAL}\",\n}}\n// {FA}\n"));
    }

    #[test]
    fn non_ascii_keys_are_matched_by_name() {
        let src = format!("const t = {{ \"{FA}\": 1 }}\n");
        assert_eq!(merge(&src, var("t"), &[], vec![member(FA, "x: 1")]), src);
    }

    #[test]
    fn json_with_non_ascii_values() {
        let src = format!("{{\n  \"title\": \"{FA}\"\n}}\n");
        let out = merge_with(
            "json",
            &src,
            Target::Document,
            &[],
            Container::Object,
            vec![member("a", "\"a\": 1")],
            Mode::Add,
        )
        .unwrap();
        assert_eq!(out, format!("{{\n  \"a\": 1,\n  \"title\": \"{FA}\"\n}}\n"));
    }

    #[test]
    fn jsx_after_non_ascii_text() {
        let src = format!("// {FA} {ASTRAL}\nconst a = <App />\n");
        let out = wrap(&src, "App", "RskApp", vec![]).unwrap();
        assert!(out.starts_with(&format!("// {FA} {ASTRAL}\n")), "{out}");
        assert!(out.contains("<RskApp><App /></RskApp>"), "{out}");
    }

    #[test]
    fn jsx_text_that_is_not_ascii_is_kept() {
        let src = format!("const a = (\n  <Router>\n    <p>{FA}</p>\n  </Router>\n)\n");
        let out = unwrap(&src, "Router").unwrap();
        assert!(out.contains(&format!("<p>{FA}</p>")), "{out}");
    }

    #[test]
    fn imports_after_non_ascii_text() {
        let src = format!("// {FA}\nimport {{ a }} from \"m\"\nconst x = \"{ASTRAL}\"\n");
        let out = ensure(&src, "m", &["b"]);
        assert_eq!(
            out,
            format!("// {FA}\nimport {{ a, b }} from \"m\"\nconst x = \"{ASTRAL}\"\n")
        );
        let out = ensure(&src, "n", &["c"]);
        assert_eq!(out, format!("// {FA}\nimport {{ a }} from \"m\"\nimport {{ c }} from \"n\"\nconst x = \"{ASTRAL}\"\n"));
    }
}

// ── JSX ─────────────────────────────────────────────────────────────────────

mod jsx {
    use super::*;

    const VITE_MAIN: &str = "import { StrictMode } from 'react'\nimport { createRoot } from 'react-dom/client'\nimport './index.css'\nimport App from './App.tsx'\n\ncreateRoot(document.getElementById('root')!).render(\n  <StrictMode>\n    <App />\n  </StrictMode>,\n)\n";

    fn rsk_props() -> Vec<Prop> {
        vec![
            prop("setup", Some("{rsk}")),
            prop("translations", Some("{translations}")),
        ]
    }

    #[test]
    fn wrap_the_app_of_a_fresh_vite_project() {
        let out = wrap(VITE_MAIN, "App", "RskApp", rsk_props()).unwrap();
        assert_eq!(
            out,
            "import { StrictMode } from 'react'\nimport { createRoot } from 'react-dom/client'\nimport './index.css'\nimport App from './App.tsx'\n\ncreateRoot(document.getElementById('root')!).render(\n  <StrictMode>\n    <RskApp setup={rsk} translations={translations}>\n      <App />\n    </RskApp>\n  </StrictMode>,\n)\n"
        );
    }

    #[test]
    fn wrapping_twice_changes_nothing() {
        let once = wrap(VITE_MAIN, "App", "RskApp", rsk_props()).unwrap();
        assert_eq!(wrap(&once, "App", "RskApp", rsk_props()).unwrap(), once);
    }

    #[test]
    fn a_one_line_tree_is_wrapped_on_one_line() {
        let out = wrap(
            "const a = <StrictMode><App /></StrictMode>\n",
            "App",
            "RskApp",
            vec![],
        )
        .unwrap();
        assert_eq!(
            out,
            "const a = <StrictMode><RskApp><App /></RskApp></StrictMode>\n"
        );
    }

    #[test]
    fn an_element_passed_straight_to_render() {
        let src = "root.render(<App />)\n";
        let out = wrap(src, "App", "RskApp", rsk_props()).unwrap();
        assert_eq!(
            out,
            "root.render(<RskApp setup={rsk} translations={translations}><App /></RskApp>)\n"
        );
    }

    #[test]
    fn an_element_with_children_and_attributes_is_wrapped_whole() {
        let src = "const a = (\n  <Layout title=\"x\">\n    <Page />\n  </Layout>\n)\n";
        let out = wrap(src, "Layout", "Shell", vec![]).unwrap();
        assert_eq!(out, "const a = (\n  <Shell>\n    <Layout title=\"x\">\n      <Page />\n    </Layout>\n  </Shell>\n)\n");
    }

    #[test]
    fn a_comment_inside_the_wrapped_element_is_kept() {
        let src = "const a = (\n  <Layout>\n    {/* the page */}\n    <Page />\n  </Layout>\n)\n";
        let out = wrap(src, "Layout", "Shell", vec![]).unwrap();
        assert!(out.contains("{/* the page */}"), "{out}");
    }

    #[test]
    fn a_member_expression_name() {
        let src = "const a = <Ui.Provider><App /></Ui.Provider>\n";
        let out = wrap(src, "Ui.Provider", "RskApp", vec![]).unwrap();
        assert_eq!(
            out,
            "const a = <RskApp><Ui.Provider><App /></Ui.Provider></RskApp>\n"
        );
    }

    #[test]
    fn a_missing_element_is_an_error() {
        let err = wrap("const a = <div />\n", "App", "RskApp", vec![]).unwrap_err();
        assert!(err.contains("App"), "{err}");
    }

    #[test]
    fn jsx_in_a_dialect_without_jsx_is_a_parse_error() {
        let err = edit(
            "const a = <App />\n",
            "ts",
            &Request::WrapJsx {
                element: "App".into(),
                wrapper: "RskApp".into(),
                props: vec![],
            },
        )
        .unwrap_err();
        assert!(err.contains("could not parse"), "{err}");
    }

    #[test]
    fn unwrap_a_single_element_child() {
        let src = "const a = (\n  <BrowserRouter>\n    <App />\n  </BrowserRouter>\n)\n";
        assert_eq!(
            unwrap(src, "BrowserRouter").unwrap(),
            "const a = (\n  <App />\n)\n"
        );
    }

    #[test]
    fn unwrap_inside_another_element_keeps_all_its_children() {
        let src = "const a = (\n  <StrictMode>\n    <BrowserRouter>\n      <Nav />\n      <App />\n    </BrowserRouter>\n  </StrictMode>\n)\n";
        let out = unwrap(src, "BrowserRouter").unwrap();
        assert_eq!(
            out,
            "const a = (\n  <StrictMode>\n    <Nav />\n    <App />\n  </StrictMode>\n)\n"
        );
    }

    #[test]
    fn unwrap_several_children_outside_jsx_makes_a_fragment() {
        let src = "root.render(<BrowserRouter><Nav /><App /></BrowserRouter>)\n";
        assert_eq!(
            unwrap(src, "BrowserRouter").unwrap(),
            "root.render(<><Nav /><App /></>)\n"
        );
    }

    #[test]
    fn unwrap_something_that_is_not_there_changes_nothing() {
        let src = "const a = <App />\n";
        assert_eq!(unwrap(src, "BrowserRouter").unwrap(), src);
    }

    #[test]
    fn unwrap_an_element_with_nothing_inside_is_an_error() {
        assert!(unwrap(
            "const a = <BrowserRouter></BrowserRouter>\n",
            "BrowserRouter"
        )
        .is_err());
    }

    #[test]
    fn add_a_prop() {
        let out = props(
            "const a = <RskApp><App /></RskApp>\n",
            "RskApp",
            vec![prop("setup", Some("{rsk}"))],
            Mode::Add,
        )
        .unwrap();
        assert_eq!(out, "const a = <RskApp setup={rsk}><App /></RskApp>\n");
    }

    #[test]
    fn add_a_prop_to_a_self_closing_element() {
        let out = props(
            "const a = <App />\n",
            "App",
            vec![prop("debug", None)],
            Mode::Add,
        )
        .unwrap();
        assert_eq!(out, "const a = <App debug />\n");
    }

    #[test]
    fn a_prop_already_there_is_left_alone_in_add_mode() {
        let src = "const a = <RskApp setup={mine} />\n";
        assert_eq!(
            props(src, "RskApp", vec![prop("setup", Some("{rsk}"))], Mode::Add).unwrap(),
            src
        );
    }

    #[test]
    fn set_mode_replaces_a_prop() {
        let out = props(
            "const a = <RskApp setup={mine} x=\"1\" />\n",
            "RskApp",
            vec![prop("setup", Some("{rsk}"))],
            Mode::Set,
        )
        .unwrap();
        assert_eq!(out, "const a = <RskApp setup={rsk} x=\"1\" />\n");
    }

    #[test]
    fn props_go_after_generic_type_arguments() {
        let out = props(
            "const a = <Table<Row> rows={r} />\n",
            "Table",
            vec![prop("dense", None)],
            Mode::Add,
        )
        .unwrap();
        assert_eq!(out, "const a = <Table<Row> dense rows={r} />\n");
    }

    #[test]
    fn spread_attributes_are_not_mistaken_for_props() {
        let out = props(
            "const a = <App {...rest} />\n",
            "App",
            vec![prop("x", Some("{1}"))],
            Mode::Add,
        )
        .unwrap();
        assert_eq!(out, "const a = <App x={1} {...rest} />\n");
    }

    #[test]
    fn missing_element_for_props_is_an_error() {
        assert!(props(
            "const a = <div />\n",
            "App",
            vec![prop("x", None)],
            Mode::Add
        )
        .is_err());
    }

    #[test]
    fn elements_are_listed_in_source_order() {
        let answer = query(
            "const a = <A><B /><C.D /></A>\n",
            "tsx",
            &Query::JsxElements,
        )
        .unwrap();
        assert_eq!(answer, "[\"A\",\"B\",\"C.D\"]");
    }
}

// ── imports ─────────────────────────────────────────────────────────────────

mod imports {
    use super::*;

    #[test]
    fn into_braces_that_already_hold_names() {
        assert_eq!(
            ensure("import { a } from \"m\"\n", "m", &["b"]),
            "import { a, b } from \"m\"\n"
        );
    }

    #[test]
    fn a_name_already_imported_changes_nothing() {
        let src = "import { a, b } from \"m\"\n";
        assert_eq!(ensure(src, "m", &["a", "b"]), src);
    }

    #[test]
    fn only_the_missing_names_are_added() {
        assert_eq!(
            ensure("import { a } from \"m\"\n", "m", &["a", "b"]),
            "import { a, b } from \"m\"\n"
        );
    }

    #[test]
    fn an_aliased_import_counts_by_its_original_name() {
        let src = "import { a as x } from \"m\"\n";
        assert_eq!(ensure(src, "m", &["a"]), src);
    }

    #[test]
    fn into_empty_braces() {
        assert_eq!(
            ensure("import {} from \"m\"\n", "m", &["a"]),
            "import { a } from \"m\"\n"
        );
    }

    #[test]
    fn next_to_a_default_import() {
        assert_eq!(
            ensure("import React from \"react\"\n", "react", &["useState"]),
            "import React, { useState } from \"react\"\n"
        );
    }

    #[test]
    fn a_new_line_after_the_last_import_in_the_files_own_style() {
        let src = "import a from 'a'\nimport b from 'b'\n\nconst x = 1\n";
        assert_eq!(
            ensure(src, "m", &["n"]),
            "import a from 'a'\nimport b from 'b'\nimport { n } from 'm'\n\nconst x = 1\n"
        );
    }

    #[test]
    fn semicolons_are_followed() {
        assert_eq!(
            ensure("import a from \"a\";\nconst x = 1;\n", "m", &["n"]),
            "import a from \"a\";\nimport { n } from \"m\";\nconst x = 1;\n"
        );
    }

    #[test]
    fn no_imports_at_all_goes_at_the_top() {
        assert_eq!(
            ensure("const x = 1\n", "m", &["n"]),
            "import { n } from \"m\"\nconst x = 1\n"
        );
    }

    #[test]
    fn a_type_only_import_is_not_extended() {
        let out = ensure("import type { T } from \"m\"\n", "m", &["n"]);
        assert_eq!(
            out,
            "import type { T } from \"m\"\nimport { n } from \"m\"\n"
        );
    }

    #[test]
    fn a_namespace_or_bare_import_is_not_extended() {
        assert_eq!(
            ensure("import * as m from \"m\"\n", "m", &["n"]),
            "import * as m from \"m\"\nimport { n } from \"m\"\n"
        );
        assert_eq!(
            ensure("import \"m\"\n", "m", &["n"]),
            "import \"m\"\nimport { n } from \"m\"\n"
        );
    }

    #[test]
    fn a_comment_between_imports_is_kept() {
        let src = "import a from \"a\" // the first\n// then\nimport b from \"b\"\n";
        let out = ensure(src, "m", &["n"]);
        assert!(out.contains("// the first\n// then"), "{out}");
    }

    #[test]
    fn ensuring_twice_changes_nothing() {
        let once = ensure("import a from \"a\"\n", "m", &["n", "o"]);
        assert_eq!(ensure(&once, "m", &["n", "o"]), once);
    }

    #[test]
    fn move_some_names_to_a_module_that_has_no_import_yet() {
        let src = "import { createRsk, createHttpAdapter, bearerAuth } from \"@rsk/core\"\n";
        let out = move_to(
            src,
            "@rsk/core",
            "@rsk/http",
            &["createHttpAdapter", "bearerAuth"],
        );
        assert_eq!(out, "import { createRsk } from \"@rsk/core\"\nimport { createHttpAdapter, bearerAuth } from \"@rsk/http\"\n");
    }

    #[test]
    fn move_into_an_existing_import() {
        let src = "import { a, b } from \"x\"\nimport { c } from \"y\"\n";
        assert_eq!(
            move_to(src, "x", "y", &["b"]),
            "import { a } from \"x\"\nimport { c, b } from \"y\"\n"
        );
    }

    #[test]
    fn moving_every_name_removes_the_import_line() {
        let src = "import { a } from \"x\"\nconst k = 1\n";
        // Nothing is left to follow, so the new import goes to the top.
        assert_eq!(
            move_to(src, "x", "y", &["a"]),
            "import { a } from \"y\"\nconst k = 1\n"
        );
    }

    #[test]
    fn moving_the_last_and_middle_names_keeps_the_commas_valid() {
        let src = "import { a, b, c, d } from \"x\"\n";
        let out = move_to(src, "x", "y", &["b", "c"]);
        assert!(out.starts_with("import { a, d } from \"x\""), "{out}");
        assert!(out.contains("import { b, c } from \"y\""), "{out}");
    }

    #[test]
    fn a_default_import_stays_when_the_named_ones_move() {
        let src = "import Core, { a } from \"x\"\n";
        let out = move_to(src, "x", "y", &["a"]);
        assert!(out.contains("import Core"), "{out}");
        assert!(out.contains("import { a } from \"y\""), "{out}");
    }

    #[test]
    fn moving_names_that_are_not_imported_changes_nothing() {
        let src = "import { a } from \"x\"\n";
        assert_eq!(move_to(src, "x", "y", &["zzz"]), src);
    }

    #[test]
    fn a_comment_on_a_neighbouring_import_survives_a_move() {
        let src = "// core\nimport { a, b } from \"x\" // two\n";
        let out = move_to(src, "x", "y", &["b"]);
        assert!(out.starts_with("// core\nimport { a }"), "{out}");
    }

    #[test]
    fn moving_twice_changes_nothing() {
        let once = move_to("import { a, b } from \"x\"\n", "x", "y", &["b"]);
        assert_eq!(move_to(&once, "x", "y", &["b"]), once);
    }

    #[test]
    fn imports_are_listed() {
        let answer = query("import a, { b, c as d } from \"m\"\nimport type { T } from \"t\"\nimport * as n from \"n\"\nimport \"side\"\n", "ts", &Query::Imports).unwrap();
        let v: serde_json::Value = serde_json::from_str(&answer).unwrap();
        assert_eq!(v[0]["module"], "m");
        assert_eq!(v[0]["names"], serde_json::json!(["b", "c"]));
        assert_eq!(v[0]["default"], true);
        assert_eq!(v[1]["type_only"], true);
        assert_eq!(v[2]["namespace"], true);
        assert_eq!(v[3]["module"], "side");
    }
}

// ── queries ─────────────────────────────────────────────────────────────────

mod queries {
    use super::*;

    #[test]
    fn the_members_of_an_object() {
        let src = "export const translations = { fa: {}, en: {} } as const\n";
        assert_eq!(
            members_of(src, "ts", var("translations"), &[]).unwrap(),
            vec!["fa", "en"]
        );
    }

    #[test]
    fn the_members_at_a_path() {
        let src = "export default { resolve: { alias: { a: 1, \"b-c\": 2 } } }\n";
        assert_eq!(
            members_of(src, "ts", Target::DefaultExport, &["resolve", "alias"]).unwrap(),
            vec!["a", "b-c"]
        );
    }

    #[test]
    fn a_missing_path_is_an_error_not_an_empty_list() {
        assert!(members_of(
            "export default { a: 1 }\n",
            "ts",
            Target::DefaultExport,
            &["resolve"]
        )
        .is_err());
    }

    #[test]
    fn json_members() {
        assert_eq!(
            members_of(
                "{\"name\": \"x\", \"private\": true}",
                "json",
                Target::Document,
                &[]
            )
            .unwrap(),
            vec!["name", "private"]
        );
    }

    #[test]
    fn non_ascii_member_names() {
        let src = format!("const t = {{ \"{FA}\": 1, b: 2 }}\n");
        assert_eq!(
            members_of(&src, "ts", var("t"), &[]).unwrap(),
            vec![FA.to_string(), "b".to_string()]
        );
    }
}
