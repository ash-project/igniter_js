// SPDX-FileCopyrightText: 2024 Shahryar Tavakkoli
// SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs.contributors>
//
// SPDX-License-Identifier: MIT

use biome_css_formatter::{context::CssFormatOptions, format_node};
use biome_css_parser::{parse_css, CssParserOptions};
use biome_css_syntax::CssFileSource;
use biome_formatter::{IndentStyle, IndentWidth};

pub fn format(source_code: &str) -> Result<String, String> {
    let parsed = parse_css(source_code, CssParserOptions::default());

    if parsed.has_errors() {
        return Err("Parsing failed due to syntax errors.".into());
    }

    let options = CssFormatOptions::new(CssFileSource::default())
        .with_indent_style(IndentStyle::Space)
        .with_indent_width(IndentWidth::default());

    let result = format_node(options, &parsed.syntax())
        .map_err(|err| format!("Formatting failed: {}", err))?;

    let formatted = result.print().map_err(|err| err.to_string())?;

    Ok(formatted.into_code())
}

pub fn is_formatted(source_code: &str) -> Result<bool, String> {
    let formatted_code = format(source_code)?;
    Ok(formatted_code.trim() == source_code.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_css() -> &'static str {
        r##"
        body {
                    background-color: #fff;
        }

        /* Test top comment */





        h1 {
        /* Test inner comment */
            font-size: 20px; /* this is for comment */
        }
        "##
    }

    #[test]
    fn test_format_css() {
        match format(app_css()) {
            Ok(formatted_code) => {
                println!("Formatted CSS:\n{}", formatted_code);
            }
            Err(error) => {
                eprintln!("Error: {}", error);
            }
        }
    }

    #[test]
    fn test_is_formatted_css() {
        assert!(is_formatted(app_css()).is_ok());
        let css_unformatted = "body{background-color:#fff;}h1{font-size:20px;}";
        let css_formatted = r#"body {
  background-color: #fff;
}
h1 {
  font-size: 20px;
}
"#;
        assert!(!is_formatted(css_unformatted).unwrap());

        let formatted = format(css_formatted).unwrap();
        assert!(is_formatted(&formatted).unwrap());
    }

    #[test]
    fn test_format_rejects_syntax_errors() {
        assert!(format("body { color").is_err());
        assert!(format("@@@").is_err());
    }

    #[test]
    fn test_is_formatted_propagates_syntax_errors() {
        assert!(is_formatted("body { color").is_err());
    }

    #[test]
    fn test_format_is_idempotent() {
        let once = format("body{color:red;}").unwrap();
        let twice = format(&once).unwrap();
        assert_eq!(once, twice);
    }

    #[test]
    fn test_format_uses_two_space_indent() {
        assert_eq!(
            format("body{color:red;}").unwrap(),
            "body {\n  color: red;\n}\n"
        );
    }

    #[test]
    fn test_format_preserves_comments() {
        let output = format("/* keep me */\nbody{color:red;}").unwrap();
        assert!(output.contains("/* keep me */"), "got: {output}");
    }

    #[test]
    fn test_format_empty_input() {
        assert_eq!(format("").unwrap(), "");
    }
}
