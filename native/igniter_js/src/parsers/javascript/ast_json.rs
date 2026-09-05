// SPDX-FileCopyrightText: 2024 Shahryar Tavakkoli
// SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs.contributors>
//
// SPDX-License-Identifier: MIT

use super::dialect::Dialect;
use oxc_allocator::Allocator;
use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_diagnostics::Severity;
use oxc_parser::{ParseOptions, Parser};
use serde_json::json;

/// Converts JavaScript AST to the ESTree format.
///
/// This function takes JavaScript source code, parses it into an Abstract Syntax Tree (AST),
/// and converts it into the ESTree-compatible JSON format. It also captures any parsing errors
/// and comments within the source.
///
/// # Arguments
/// * `source_text` - The JavaScript source code as a string.
///
/// # Returns
/// * `Ok(String)` - A pretty-printed JSON representation of the AST in ESTree format.
/// * `Err(String)` - If parsing or JSON serialization fails.
///
/// # Errors
/// * Returns `"Failed to serialize JSON"` if the ESTree AST cannot be converted to JSON.
/// * If there are syntax errors in `source_text`, they will be included in the `"errors"` field.
///
/// # Output Structure
/// The returned JSON contains:
/// * `"program"` - The parsed AST in ESTree format. Every node carries `range`
///   (`[start, end]`) alongside `start`/`end`, and TypeScript fields are included.
/// * `"comments"` - Extracted comments from the source code, each tagged `"Line"`
///   or `"Block"`. Offsets are UTF-16, not UTF-8 bytes.
/// * `"errors"` - A list of syntax errors with details. A diagnostic with no
///   labels serializes `"labels"` as `null` rather than `[]`.
///
/// # Example
/// ```rust
/// let js_code = "function test() { console.log('Hello, world!'); } // Comment";
/// let result = convert_ast_to_estree(js_code);
///
/// assert!(result.is_ok());
/// let json_output = result.unwrap();
/// assert!(json_output.contains("\"type\": \"Program\""));
/// assert!(json_output.contains("\"type\": \"FunctionDeclaration\""));
/// assert!(json_output.contains("\"comments\""));
/// ```
pub fn convert_ast_to_estree(source_text: &str) -> Result<String, String> {
    convert_ast_to_estree_as(source_text, Dialect::Js)
}

/// As [`convert_ast_to_estree`], in a given dialect.
///
/// The source type used to be `SourceType::from_path("example.js")` with an `.expect()` on it —
/// a hardcoded dialect *and* a panic path, in one line, for a filename that was never real.
pub fn convert_ast_to_estree_as(source_text: &str, dialect: Dialect) -> Result<String, String> {
    let source_type = dialect.oxc_source_type();
    let allocator = Allocator::default();
    let parser_return = Parser::new(&allocator, source_text, source_type)
        .with_options(ParseOptions {
            parse_regular_expression: true,
            ..ParseOptions::default()
        })
        .parse();

    let errors = parser_return
        .diagnostics
        .into_iter()
        .map(|e| {
            let severity = match e.severity {
                Severity::Error => "Error",
                Severity::Warning => "Warning",
                Severity::Advice => "Advice",
            };

            let help = e.help.as_ref().map(|h| h.to_string());

            let labels = if e.labels.is_empty() {
                None
            } else {
                Some(
                    e.labels
                        .iter()
                        .map(|label| {
                            let span = label.inner();
                            let start = span.offset();
                            let end = start + span.len();

                            json!({
                                "start": start,
                                "end": end,
                                "label": label.label().map(|s| s.to_string()),
                                "primary": label.primary()
                            })
                        })
                        .collect::<Vec<_>>(),
                )
            };

            let code = e.code.to_string();
            let url = e.url.as_ref().map(|u| u.to_string());

            json!({
                "severity": severity,
                "message": e.message,
                "help": help,
                "labels": labels,
                "code": code,
                "url": url
            })
        })
        .collect::<Vec<_>>();

    let mut program = parser_return.program;
    let span_converter = Utf8ToUtf16::new(source_text);
    span_converter.convert_program(&mut program);

    let comments_json: Vec<_> = program
        .comments
        .iter()
        .map(|comment| {
            let value = comment.content_span().source_text(source_text).to_string();
            let mut span = comment.span;
            if let Some(mut converter) = span_converter.converter() {
                converter.convert_span(&mut span);
            }
            json!({
                "type": if comment.is_line() { "Line" } else { "Block" },
                "value": value,
                "start": span.start,
                "end": span.end
            })
        })
        .collect();
    let estree_json = program.to_pretty_estree_json(true, true);

    let full_json = json!({
        "program": serde_json::from_str::<serde_json::Value>(&estree_json).unwrap_or(json!({})),
        "comments": comments_json,
        "errors": errors
    });

    serde_json::to_string_pretty(&full_json)
        .map_err(|e| format!("Failed to serialize JSON: {:?}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn is_valid_json(json_str: &str) -> bool {
        serde_json::from_str::<Value>(json_str).is_ok()
    }

    #[test]
    fn test_convert_ast_to_estree() {
        let js_code = r#"
            function test() { console.log("Hello, world!"); } // comment1
            const alert = true
            function test2() { console.log("Hello, world!"); } // comment2
            "#;

        let result = convert_ast_to_estree(js_code);
        assert!(result.is_ok());
        let json_output = result.unwrap();
        println!("{}", json_output);
        assert!(is_valid_json(&json_output));
    }

    fn parse_to_value(code: &str) -> Value {
        serde_json::from_str(&convert_ast_to_estree(code).expect("conversion must succeed"))
            .expect("output must be valid JSON")
    }

    #[test]
    fn test_top_level_shape() {
        let value = parse_to_value("const a = 1;");

        assert!(value.get("program").is_some());
        assert!(value.get("comments").is_some());
        assert!(value.get("errors").is_some());
        assert_eq!(value["program"]["type"], "Program");
    }

    #[test]
    fn test_valid_source_has_no_errors() {
        let value = parse_to_value("const a = 1;");
        assert_eq!(value["errors"].as_array().unwrap().len(), 0);
    }

    /// Syntax errors are reported in the `errors` array rather than failing the
    /// whole conversion.
    #[test]
    fn test_syntax_errors_are_reported_not_raised() {
        let value = parse_to_value("const a = ;");
        let errors = value["errors"].as_array().unwrap();

        assert!(!errors.is_empty());
        assert_eq!(errors[0]["severity"], "Error");
        assert!(errors[0]["message"].is_string());
    }

    /// Diagnostics without labels serialize as `null`, not `[]`.
    #[test]
    fn test_label_less_diagnostics_serialize_as_null() {
        let value = parse_to_value("const a = ;");

        for error in value["errors"].as_array().unwrap() {
            let labels = &error["labels"];
            assert!(
                labels.is_null() || labels.is_array(),
                "labels must be null or an array, got {labels}"
            );
        }
    }

    #[test]
    fn test_comments_are_captured_with_kind_and_span() {
        let value = parse_to_value("const a = 1; /* block */ // line");
        let comments = value["comments"].as_array().unwrap();

        let kinds: Vec<_> = comments
            .iter()
            .map(|c| c["type"].as_str().unwrap())
            .collect();
        assert_eq!(kinds, vec!["Block", "Line"]);

        for comment in comments {
            assert!(comment["start"].as_u64() < comment["end"].as_u64());
            assert!(comment["value"].is_string());
        }
    }

    /// `ranges` is enabled, so every node carries `range` next to `start`/`end`.
    #[test]
    fn test_nodes_carry_range() {
        let value = parse_to_value("const a = 1;");
        let program = &value["program"];

        let range = program["range"]
            .as_array()
            .expect("program must have a range");
        assert_eq!(range[0], program["start"]);
        assert_eq!(range[1], program["end"]);

        let node = &program["body"][0];
        assert_eq!(node["type"], "VariableDeclaration");
        assert!(node["range"].is_array());
    }

    #[test]
    fn test_empty_source_still_produces_a_program() {
        let value = parse_to_value("");

        assert_eq!(value["program"]["type"], "Program");
        assert_eq!(value["program"]["body"].as_array().unwrap().len(), 0);
        assert_eq!(value["errors"].as_array().unwrap().len(), 0);
    }

    /// Multi-byte characters shift UTF-8 offsets, which are converted to the
    /// UTF-16 offsets ESTree consumers expect.
    #[test]
    fn test_spans_are_converted_to_utf16_offsets() {
        let value = parse_to_value("const emoji = \"😀\"; // tail");
        let comments = value["comments"].as_array().unwrap();

        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0]["start"].as_u64().unwrap(), 20);
    }
}
