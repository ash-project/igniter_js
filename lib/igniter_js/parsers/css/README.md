# CSS Tools for Elixir

A Python package that provides CSS manipulation tools designed to be used with Elixir applications via Pythonx.

## Features

- **CSS Parsing**: Parse and analyze CSS stylesheets using tinycss2
- **CSS Modification**: Add, modify, or remove CSS properties and selectors
- **CSS Extraction**: Extract specific elements like colors, media queries, animations
- **CSS Minification**: Minify CSS for production or beautify for development
- **CSS Analysis**: Get statistics and insights about your CSS

## Installation

### 1. Create the Python package

From the project root directory:

```bash
cd plibs/css_tools
python -m pip install --upgrade build
python -m build
```

This will create a distributable package in the `dist` directory.

### 2. Configure Elixir to use the package

In your `config/config.exs`:

```elixir
config :pythonx, :uv_init,
  pyproject_toml: """
  [project]
  name = "igniter_py"
  version = "0.4.4"
  requires-python = "==3.13.*"
  dependencies = [
    "css_tools"
  ]
  [tool.uv.sources]
  css_tools = { path = "#{File.cwd!()}/plibs/css_tools/dist/css_tools-0.1.0-py3-none-any.whl" }
  """
```

## Usage in Elixir

```elixir
defmodule YourApp.CssExample do
  alias IgniterJs.Parsers.CSS.Parser

  def add_property_example(css_code) do
    # Add a display: none property to .hide-scrollbar
    updated_css = Parser.add_hide_scrollbar_property(css_code)
    IO.puts("Updated CSS:")
    IO.puts(updated_css)
  end

  def analyze_css_example(css_code) do
    # Get statistics about the CSS
    stats = Parser.analyze_css(css_code)
    IO.inspect(stats, label: "CSS Statistics")
  end

  def extract_colors_example(css_code) do
    # Extract all color values
    colors = Parser.extract_colors(css_code)
    IO.inspect(colors, label: "Colors Used")
  end

  def minify_example(css_code) do
    # Minify the CSS for production
    minified = Parser.minify(css_code)
    IO.puts("Minified CSS:")
    IO.puts(minified)
  end
end
```

## Advanced Usage with IgniterJs.CssProcessor

The `IgniterJs.CssProcessor` module provides higher-level functions for common CSS processing tasks:

```elixir
alias IgniterJs.CssProcessor

# Process CSS for production (minify, add prefixes, etc.)
production_css = CssProcessor.process_for_production(css_content)

# Process CSS for development (beautify, analyze)
dev_result = CssProcessor.process_for_development(css_content)
IO.puts(dev_result.beautified_css)
IO.inspect(dev_result.analytics)

# Apply browser compatibility fixes
compatible_css = CssProcessor.apply_browser_compatibility(css_content)

# Extract critical CSS for above-the-fold content
{critical, non_critical} = CssProcessor.extract_critical_css(css_content, [".header", ".hero"])

# Merge multiple CSS files
merged_css = CssProcessor.merge_css_files(%{
  "styles.css" => styles_content,
  "components.css" => components_content
})
```

## Python API Reference

The package exposes several modules for working with CSS:

- `css_tools.parser`: Core parsing functionality
- `css_tools.modifier`: Tools for modifying CSS
- `css_tools.extractor`: Tools for extracting information from CSS
- `css_tools.minifier`: Tools for minifying and beautifying CSS

See the Python code documentation for more details on available functions.
