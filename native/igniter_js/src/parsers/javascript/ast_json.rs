// SPDX-FileCopyrightText: 2024 Shahryar Tavakkoli
// SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs.contributors>
//
// SPDX-License-Identifier: MIT

use oxc_allocator::Allocator;
use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_diagnostics::Severity;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
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
/// * `"program"` - The parsed AST in ESTree format.
/// * `"comments"` - Extracted comments from the source code.
/// * `"errors"` - A list of syntax errors with details.
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
    let source_type = SourceType::from_path("example.js").expect("Invalid file extension");
    let allocator = Allocator::default();
    let parser_return = Parser::new(&allocator, source_text, source_type)
        .with_options(ParseOptions {
            parse_regular_expression: true,
            ..ParseOptions::default()
        })
        .parse();

    let errors = parser_return
        .errors
        .into_iter()
        .map(|e| {
            let severity = match e.severity {
                Severity::Error => "Error",
                Severity::Warning => "Warning",
                Severity::Advice => "Advice",
            };

            let help = e.help.as_ref().map(|h| h.to_string());

            let labels = e.labels.as_ref().map(|labels| {
                labels
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
                    .collect::<Vec<_>>()
            });

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
    let estree_json = program.to_pretty_estree_ts_json(true);

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
}
