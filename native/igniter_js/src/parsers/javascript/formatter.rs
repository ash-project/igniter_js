use biome_formatter::{IndentStyle, IndentWidth};
use biome_js_formatter::context::JsFormatOptions;
use biome_js_formatter::format_node;
use biome_js_parser::{parse, JsParserOptions};
use biome_js_syntax::{JsFileSource, ModuleKind};

/// Formats JavaScript source code using a standardized formatting style.
///
/// This function parses the provided JavaScript `source_code`, checks for syntax errors,
/// and then formats it according to predefined formatting options. It ensures consistent
/// indentation and structure.
///
/// # Arguments
/// * `source_code` - A string containing JavaScript source code.
///
/// # Returns
/// * `Ok(String)` - The formatted JavaScript code.
/// * `Err(String)` - If parsing or formatting fails.
///
/// # Errors
/// * Returns `"Parsing failed due to syntax errors."` if the input code contains syntax errors.
/// * Returns `"Formatting failed: <error message>"` if the formatting process encounters an issue.
///
/// # Example
/// ```rust
/// let js_code = "function test(){console.log('Hello, world!');}";
/// let result = format(js_code);
///
/// assert!(result.is_ok());
/// let formatted_code = result.unwrap();
/// assert!(formatted_code.contains("function test() {"));
/// assert!(formatted_code.contains("console.log('Hello, world!');"));
/// ```
pub fn format(source_code: &str) -> Result<String, String> {
    let parsed = parse(
        source_code,
        JsFileSource::default().with_module_kind(ModuleKind::Module),
        JsParserOptions::default(),
    );

    if parsed.has_errors() {
        return Err("Parsing failed due to syntax errors.".into());
    }

    let options =
        JsFormatOptions::new(JsFileSource::default().with_module_kind(ModuleKind::Module))
            .with_indent_style(IndentStyle::Space)
            .with_indent_width(IndentWidth::default());

    let result = format_node(options, &parsed.syntax())
        .map_err(|err| format!("Formatting failed: {}", err))?;

    let formatted = result.print().map_err(|err| err.to_string())?;

    Ok(formatted.into_code())
}

/// Checks if the given JavaScript source code is already formatted.
///
/// This function formats the provided `source_code` and compares it with the original.
/// If the formatted version matches the input (ignoring leading and trailing spaces),
/// it returns `true`; otherwise, it returns `false`.
///
/// # Arguments
/// * `source_code` - A string containing JavaScript source code.
///
/// # Returns
/// * `Ok(true)` - If the input is already correctly formatted.
/// * `Ok(false)` - If formatting would modify the input code.
/// * `Err(String)` - If formatting fails due to syntax errors or other issues.
///
/// # Errors
/// * Returns an error if the formatting process encounters a failure.
///
/// # Example
/// ```rust
/// let formatted_js = "function test() {\n    console.log('Hello, world!');\n}";
/// let result = is_formatted(formatted_js);
/// assert_eq!(result, Ok(true));
///
/// let unformatted_js = "function test(){console.log('Hello, world!');}";
/// let result = is_formatted(unformatted_js);
/// assert_eq!(result, Ok(false));
/// ```
pub fn is_formatted(source_code: &str) -> Result<bool, String> {
    let formatted_code = format(source_code)?;
    Ok(formatted_code.trim() == source_code.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_js() -> &'static str {
        r##"
        // If you want to use Phoenix channels, run `mix help phx.gen.channel`
        // to get started and then uncomment the line below.
        // import "./user_socket.js"
        // You can include dependencies in two ways.
        //
        // The simplest option is to put them in assets/vendor and
        // import them using relative paths:
        //
        //     import "../vendor/some-package.js"
        //
        // Alternatively, you can `npm install some-package --prefix assets` and import
        // them using a path starting with the package name:
        //
        //     import "some-package"
        //
        // Include phoenix_html to handle method=PUT/DELETE in forms and buttons.
        import "phoenix_html";
        // Establish Phoenix Socket and LiveView configuration.
        import { Socket } from "phoenix";
        import { LiveSocket } from "phoenix_live_view";
        import topbar from "../vendor/topbar";
        import MishkaComponents from "../vendor/mishka_components.js";
        let csrfToken = document.querySelector("meta[name='csrf-token']").getAttribute("content");
        let liveSocket = new LiveSocket("/live", Socket, {
            longPollFallbackMs: 2500,
            params: {
                _csrf_token: csrfToken
            },
            hooks: {
                ...MishkaComponents
            }
        });
        // Show progress bar on live navigation and form submits
        topbar.config({
            barColors: {
                0: "#29d"
            },
            shadowColor: "rgba(0, 0, 0, .3)"
        });
        window.addEventListener("phx:page-loading-start", (_info)=>topbar.show(300));
        window.addEventListener("phx:page-loading-stop", (_info)=>topbar.hide());
        // connect if there are any LiveViews on the page
        liveSocket.connect();
        // expose liveSocket on window for web console debug logs and latency simulation:
        // >> liveSocket.enableDebug()
        // >> liveSocket.enableLatencySim(1000)  // enabled for duration of browser session
        // >> liveSocket.disableLatencySim()
        window.liveSocket = liveSocket;
        "##
    }

    #[test]
    fn test_format_js() {
        match format(app_js()) {
            Ok(formatted_code) => {
                println!("Formatted JS:\n{}", formatted_code);
            }
            Err(error) => {
                eprintln!("Error: {}", error);
            }
        }
    }

    #[test]
    fn test_is_formatted_js() {
        assert!(is_formatted(app_js()).is_ok());
        let js_code_unformatted = "function test(){console.log('hello world');}";
        let js_code_formatted = r#"function test() {
          console.log("hello world");
        }"#;
        assert_eq!(is_formatted(js_code_unformatted).unwrap(), false);

        let formatted = format(js_code_formatted).unwrap();
        assert_eq!(is_formatted(&formatted).unwrap(), true);
    }
}
