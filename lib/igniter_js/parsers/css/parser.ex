defmodule IgniterJs.Parsers.CSS.Parser do
  @moduledoc """
  CSS parsing and manipulation using Python's tinycss2 library.

  This module provides functions to work with CSS files by leveraging
  a Python toolkit built on tinycss2 for parsing, modifying, and analyzing CSS.
  """

  @doc """
  Adds a display: none property to the .hide-scrollbar class.
  If the class doesn't exist, it creates it.

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.add_hide_scrollbar_property(css_code)
      updated css with .hide-scrollbar having display: none
  """
  def add_hide_scrollbar_property(css_code) when is_binary(css_code) do
    {result, _globals} =
      Pythonx.eval(
        """
        from css_tools.modifier import add_property_to_selector

        result = add_property_to_selector(
            css_code,
            ".hide-scrollbar",
            "display",
            "none"
        )
        result
        """,
        %{"css_code" => css_code}
      )

    Pythonx.decode(result)
  end

  @doc """
  Adds vendor prefixes to specified CSS properties throughout the stylesheet.

  ## Parameters

    * `css_code` - The CSS code as a string
    * `property_name` - The CSS property to add prefixes to
    * `prefixes` - List of prefixes to add (e.g., ["-webkit-", "-moz-"])

  ## Examples

      iex> prefixes = ["-webkit-", "-moz-", "-ms-"]
      iex> IgniterJs.Parsers.CSS.Parser.add_vendor_prefixes(css_code, "user-select", prefixes)
      "updated css with vendor prefixes"
  """
  def add_vendor_prefixes(css_code, property_name, prefixes)
      when is_binary(css_code) and is_binary(property_name) and is_list(prefixes) do
    {result, _globals} =
      Pythonx.eval(
        """
        from css_tools.modifier import add_prefix_to_property

        result = add_prefix_to_property(
            css_code,
            property_name,
            prefixes
        )
        result
        """,
        %{
          "css_code" => css_code,
          "property_name" => property_name,
          "prefixes" => prefixes
        }
      )

    Pythonx.decode(result)
  end

  @doc """
  Analyzes a CSS stylesheet and returns various statistics.

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.analyze_css(css_code)
      %{
        "selectors_count" => 15,
        "unique_selectors" => 12,
        "properties_count" => 45,
        "unique_properties" => 20,
        ...
      }
  """
  def analyze_css(css_code) when is_binary(css_code) do
    {result, _globals} =
      Pythonx.eval(
        """
        from css_tools.parser import analyze_stylesheet

        result = analyze_stylesheet(css_code)
        result
        """,
        %{"css_code" => css_code}
      )

    Pythonx.decode(result)
  end

  @doc """
  Extracts all color values from a CSS stylesheet.

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.extract_colors(css_code)
      %{
        ".header" => ["color: #333", "background-color: white"],
        ".footer" => ["color: rgba(0, 0, 0, 0.8)"]
      }
  """
  def extract_colors(css_code) when is_binary(css_code) do
    {result, _globals} =
      Pythonx.eval(
        """
        from css_tools.extractor import extract_colors

        result = extract_colors(css_code)
        result
        """,
        %{"css_code" => css_code}
      )

    Pythonx.decode(result)
  end

  @doc """
  Minifies a CSS stylesheet by removing comments, whitespace, and unnecessary characters.
  We recommend not using this.

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.minify(css_code)
      ".header{color:#333;background:#fff;}.footer{color:#000;}"
  """
  def minify(css_code) when is_binary(css_code) do
    {result, _globals} =
      Pythonx.eval(
        """
        from css_tools.minifier import minify_css

        result = minify_css(css_code)
        result
        """,
        %{"css_code" => css_code}
      )

    Pythonx.decode(result)
  end

  @doc """
  Beautifies a CSS stylesheet by adding proper indentation and formatting.
  We recommend using `IgniterJs.Parsers.CSS.Formatter` module instead.

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.beautify(css_code)
      ".header {
          color: #333;
          background: #fff;
      }

      .footer {
          color: #000;
      }"
  """
  def beautify(css_code) when is_binary(css_code) do
    {result, _globals} =
      Pythonx.eval(
        """
        from css_tools.minifier import beautify_css

        result = beautify_css(css_code)
        result
        """,
        %{"css_code" => css_code}
      )

    Pythonx.decode(result)
  end

  @doc """
  Modifies a property value for a specific selector.

  ## Parameters

    * `css_code` - The CSS code as a string
    * `selector` - The CSS selector to modify
    * `property_name` - The property name to modify
    * `new_value` - The new property value
    * `important` - Whether to mark the property as !important (default: false)

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.modify_property(css_code, ".header", "color", "blue")
      "updated css with .header color: blue"
  """
  def modify_property(css_code, selector, property_name, new_value, important \\ false)
      when is_binary(css_code) and is_binary(selector) and is_binary(property_name) and
             is_binary(new_value) do
    {result, _globals} =
      Pythonx.eval(
        """
        from css_tools.modifier import modify_property_value

        result = modify_property_value(
            css_code,
            selector,
            property_name,
            new_value,
            important
        )
        result
        """,
        %{
          "css_code" => css_code,
          "selector" => selector,
          "property_name" => property_name,
          "new_value" => new_value,
          "important" => important
        }
      )

    Pythonx.decode(result)
  end

  @doc """
  Merges multiple CSS stylesheets into one, removing duplicates.

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.merge_stylesheets([css_code1, css_code2])
      "merged css"
  """
  def merge_stylesheets(css_list) when is_list(css_list) do
    {result, _globals} =
      Pythonx.eval(
        """
        from css_tools.modifier import merge_stylesheets

        result = merge_stylesheets(css_list)
        result
        """,
        %{"css_list" => css_list}
      )

    Pythonx.decode(result)
  end

  @doc """
  Removes a CSS selector and all its properties.

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.remove_selector(css_code, ".unused-class")
      "css without .unused-class"
  """
  def remove_selector(css_code, selector) when is_binary(css_code) and is_binary(selector) do
    {result, _globals} =
      Pythonx.eval(
        """
        from css_tools.modifier import remove_selector

        result = remove_selector(css_code, selector)
        result
        """,
        %{"css_code" => css_code, "selector" => selector}
      )

    Pythonx.decode(result)
  end

  @doc """
  Extracts all media queries and their contents.

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.extract_media_queries(css_code)
      %{
        "(max-width: 768px)" => [
          %{
            "selector" => ".header",
            "properties" => %{"font-size" => "14px"}
          }
        ]
      }
  """
  def extract_media_queries(css_code) when is_binary(css_code) do
    {result, _globals} =
      Pythonx.eval(
        """
        from css_tools.extractor import extract_media_queries

        result = extract_media_queries(css_code)
        result
        """,
        %{"css_code" => css_code}
      )

    Pythonx.decode(result)
  end

  @doc """
  Extracts all CSS animations and keyframes.

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.extract_animations(css_code)
      %{
        "fade-in" => %{
          "keyframes" => %{
            "0%" => %{"opacity" => "0"},
            "100%" => %{"opacity" => "1"}
          },
          "used_by" => [".header", ".modal"]
        }
      }
  """
  def extract_animations(css_code) when is_binary(css_code) do
    {result, _globals} =
      Pythonx.eval(
        """
        from css_tools.extractor import extract_animations

        result = extract_animations(css_code)
        result
        """,
        %{"css_code" => css_code}
      )

    Pythonx.decode(result)
  end

  @doc """
  Sorts CSS properties alphabetically within each rule.

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.sort_properties(css_code)
      ".header {
          background: #fff;
          color: #333;
          font-size: 16px;
      }"
  """
  def sort_properties(css_code) when is_binary(css_code) do
    {result, _globals} =
      Pythonx.eval(
        """
        from css_tools.minifier import sort_properties

        result = sort_properties(css_code)
        result
        """,
        %{"css_code" => css_code}
      )

    Pythonx.decode(result)
  end

  @doc """
  Removes duplicate selectors and properties from CSS.

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.remove_duplicates(css_code)
      "css without duplicates"
  """
  def remove_duplicates(css_code) when is_binary(css_code) do
    {result, _globals} =
      Pythonx.eval(
        """
        from css_tools.minifier import remove_duplicates

        result = remove_duplicates(css_code)
        result
        """,
        %{"css_code" => css_code}
      )

    Pythonx.decode(result)
  end

  @doc """
  Extracts all selectors that use a specific CSS property.

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.extract_selectors_by_property(css_code, "display")
      %{
        ".header" => "flex",
        ".sidebar" => "none",
        ".content" => "grid"
      }
  """
  def extract_selectors_by_property(css_code, property_name)
      when is_binary(css_code) and is_binary(property_name) do
    {result, _globals} =
      Pythonx.eval(
        """
        from css_tools.extractor import extract_selectors_by_property

        result = extract_selectors_by_property(css_code, property_name)
        result
        """,
        %{"css_code" => css_code, "property_name" => property_name}
      )

    Pythonx.decode(result)
  end

  @doc """
  Extracts all comments from CSS and associates them with nearby rules when possible.

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.extract_comments(css_code)
      %{
        "standalone_comments" => ["This is a standalone comment"],
        "rule_comments" => %{
          ".header" => ["Header styles"]
        },
        "declaration_comments" => %{
          ".footer" => %{
            "color" => ["Footer text color"]
          }
        }
      }
  """
  def extract_comments(css_code) when is_binary(css_code) do
    {result, _globals} =
      Pythonx.eval(
        """
        from css_tools.extractor import extract_comments

        result = extract_comments(css_code)
        result
        """,
        %{"css_code" => css_code}
      )

    Pythonx.decode(result)
  end

  @doc """
  Checks if the CSS code is valid by attempting to parse it.
  Returns :ok if valid, or {:error, reason} if invalid.

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.validate_css(css_code)
      :ok

      iex> IgniterJs.Parsers.CSS.Parser.validate_css("invalid { css")
      {:error, "Parse error at line 1, column 10: Missing closing brace"}
  """
  def validate_css(css_code) when is_binary(css_code) do
    {result, _globals} =
      Pythonx.eval(
        """
        import tinycss2

        try:
            # Attempt to parse the CSS
            parsed = tinycss2.parse_stylesheet(css_code)
            # Check for obvious parsing errors
            for rule in parsed:
                if hasattr(rule, 'content') and isinstance(rule.content, str) and 'syntax error' in rule.content.lower():
                    raise Exception(f"Parse error: {rule.content}")
            result = {"valid": True, "message": "CSS is valid"}
        except Exception as e:
            result = {"valid": False, "message": str(e)}

        result
        """,
        %{"css_code" => css_code}
      )

    parsed_result = Pythonx.decode(result)

    if parsed_result["valid"] do
      :ok
    else
      {:error, parsed_result["message"]}
    end
  end

  @doc """
  Finds and extracts unused CSS selectors by comparing with HTML content.

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.find_unused_selectors(css_code, html_content)
      [".unused-class", "#unused-id"]
  """
  def find_unused_selectors(css_code, html_content)
      when is_binary(css_code) and is_binary(html_content) do
    {result, _globals} =
      Pythonx.eval(
        """
        from css_tools.extractor import extract_unused_selectors

        result = extract_unused_selectors(css_code, html_content)
        result
        """,
        %{"css_code" => css_code, "html_content" => html_content}
      )

    Pythonx.decode(result)
  end

  @doc """
  Extracts all font-related properties from CSS.

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.extract_fonts(css_code)
      %{
        ".header" => [
          %{"property" => "font-family", "value" => "Arial, sans-serif"},
          %{"property" => "font-size", "value" => "16px"}
        ]
      }
  """
  def extract_fonts(css_code) when is_binary(css_code) do
    {result, _globals} =
      Pythonx.eval(
        """
        from css_tools.extractor import extract_fonts

        result = extract_fonts(css_code)
        result
        """,
        %{"css_code" => css_code}
      )

    Pythonx.decode(result)
  end

  @doc """
  Replaces an entire CSS rule for a specific selector with new declarations.

  ## Parameters

    * `css_code` - The CSS code as a string
    * `selector` - The CSS selector to replace
    * `new_declarations` - The new CSS declarations as a string (without curly braces)

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.replace_selector_rule(css_code, ".header", "color: blue; font-size: 20px; padding: 10px;")
      "css with .header rule replaced"
  """
  def replace_selector_rule(css_code, selector, new_declarations)
      when is_binary(css_code) and is_binary(selector) and is_binary(new_declarations) do
    {result, _globals} =
      Pythonx.eval(
        """
        import tinycss2
        from css_tools.parser import parse_stylesheet, get_selector_text

        if isinstance(selector, bytes):
            selector = selector.decode('utf-8')
        if isinstance(new_declarations, bytes):
            new_declarations = new_declarations.decode('utf-8')

        rules = parse_stylesheet(css_code)
        modified_css = ""
        selector_found = False

        for rule in rules:
            if rule.type == "qualified-rule":
                rule_selector = get_selector_text(rule)

                if rule_selector == selector:
                    selector_found = True
                    # Format the new declarations
                    formatted_declarations = "\\n".join("    " + line.strip() for line in new_declarations.split(";") if line.strip())
                    # Replace the rule with new content
                    formatted_rule = f"{selector} {{\\n{formatted_declarations}\\n}}\\n"
                    modified_css += formatted_rule
                else:
                    # Keep other rules as they are
                    modified_css += tinycss2.serialize([rule])
            else:
                # Keep other non-qualified rules as they are
                modified_css += tinycss2.serialize([rule])

        # Add the selector if it wasn't found
        if not selector_found:
            formatted_declarations = "\\n".join("    " + line.strip() for line in new_declarations.split(";") if line.strip())
            new_rule = f"\\n{selector} {{\\n{formatted_declarations}\\n}}\\n"
            modified_css += new_rule

        result = modified_css.strip()
        result
        """,
        %{
          "css_code" => css_code,
          "selector" => selector,
          "new_declarations" => new_declarations
        }
      )

    Pythonx.decode(result)
  end

  @doc """
  Adds an @import rule to the CSS if it doesn't already exist.

  ## Parameters

    * `css_code` - The CSS code as a string
    * `import_url` - The URL or path to import (without quotes)
    * `media_query` - Optional media query to apply to the import (e.g., "screen and (max-width: 768px)")

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.add_import(css_code, "styles.css")
      "css with @import url('styles.css') added"

      iex> IgniterJs.Parsers.CSS.Parser.add_import(css_code, "mobile.css", "screen and (max-width: 768px)")
      "css with @import url('mobile.css') screen and (max-width: 768px) added"
  """
  def add_import(css_code, import_url, media_query \\ nil)
      when is_binary(css_code) and is_binary(import_url) and
             (is_binary(media_query) or is_nil(media_query)) do
    {result, _globals} =
      Pythonx.eval(
        """
        import tinycss2
        from css_tools.parser import parse_stylesheet

        # Ensure we're working with strings
        if isinstance(css_code, bytes):
            css_code = css_code.decode('utf-8')
        if isinstance(import_url, bytes):
            import_url = import_url.decode('utf-8')

        media_query_str = ""
        if media_query is not None:
            if isinstance(media_query, bytes):
                media_query = media_query.decode('utf-8')
            media_query_str = f" {media_query}"

        # Format the import rule
        if import_url.startswith(("http://", "https://", "/")):
            # URLs need to be quoted
            new_import = f"@import url('{import_url}'){media_query_str};"
        else:
            # Relative paths can be with or without quotes
            new_import = f"@import '{import_url}'{media_query_str};"

        rules = parse_stylesheet(css_code)

        # Check if the import already exists
        exists = False
        for rule in rules:
            if rule.type == "at-rule" and rule.at_keyword.lower() == "import":
                if import_url in tinycss2.serialize(rule.prelude):
                    exists = True
                    break

        if exists:
            # Don't add duplicate import
            modified_css = css_code
        else:
            # Add new import at the beginning - imports must come before other rules
            has_imports = any(rule.type == "at-rule" and rule.at_keyword.lower() == "import" for rule in rules)

            if has_imports:
                # Add after the last import
                modified_parts = []
                last_import_index = -1

                for i, rule in enumerate(rules):
                    if rule.type == "at-rule" and rule.at_keyword.lower() == "import":
                        last_import_index = i

                # Add all rules up to the last import
                for i, rule in enumerate(rules):
                    part = tinycss2.serialize([rule])
                    # Ensure this is a string
                    if isinstance(part, bytes):
                        part = part.decode('utf-8')
                    modified_parts.append(part)

                    if i == last_import_index:
                        # Add the new import after the last existing import
                        modified_parts.append(f"\\n{new_import}\\n")

                modified_css = "".join(modified_parts)
            else:
                # No existing imports, add at the beginning
                # Make sure to convert any bytes to strings
                if isinstance(css_code, bytes):
                    css_code = css_code.decode('utf-8')
                modified_css = f"{new_import}\\n{css_code}"

        # Final check to ensure we return a string, not bytes
        if isinstance(modified_css, bytes):
            modified_css = modified_css.decode('utf-8')

        result = modified_css.strip()
        result
        """,
        %{
          "css_code" => css_code,
          "import_url" => import_url,
          "media_query" => media_query
        }
      )

    Pythonx.decode(result)
  end

  @doc """
  Removes a specific @import rule from the CSS.

  ## Parameters

    * `css_code` - The CSS code as a string
    * `import_url` - The URL or path to remove (matches partial URL)

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.remove_import(css_code, "styles.css")
      "css with @import url('styles.css') removed"
  """
  def remove_import(css_code, import_url) when is_binary(css_code) and is_binary(import_url) do
    {result, _globals} =
      Pythonx.eval(
        """
        import tinycss2
        from css_tools.parser import parse_stylesheet

        if isinstance(import_url, bytes):
            import_url = import_url.decode('utf-8')

        rules = parse_stylesheet(css_code)
        modified_css = ""

        for rule in rules:
            if rule.type == "at-rule" and rule.at_keyword.lower() == "import":
                # Check if this import contains the URL we want to remove
                serialized = tinycss2.serialize(rule.prelude)
                if import_url not in serialized:
                    # Keep imports that don't match
                    modified_css += tinycss2.serialize([rule])
            else:
                # Keep all other rules
                modified_css += tinycss2.serialize([rule])

        result = modified_css.strip()
        result
        """,
        %{
          "css_code" => css_code,
          "import_url" => import_url
        }
      )

    Pythonx.decode(result)
  end

  @doc """
  Checks if a specific CSS selector exists in the stylesheet.

  ## Parameters

    * `css_code` - The CSS code as a string
    * `selector` - The CSS selector to check for

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.selector_exists?(css_code, ".header")
      true

      iex> IgniterJs.Parsers.CSS.Parser.selector_exists?(css_code, "#nonexistent")
      false
  """
  def selector_exists?(css_code, selector) when is_binary(css_code) and is_binary(selector) do
    {result, _globals} =
      Pythonx.eval(
        """
        from css_tools.parser import parse_stylesheet, get_selector_text

        if isinstance(selector, bytes):
            selector = selector.decode('utf-8')

        rules = parse_stylesheet(css_code)
        exists = False

        for rule in rules:
            if rule.type == "qualified-rule":
                rule_selector = get_selector_text(rule)
                if rule_selector == selector:
                    exists = True
                    break

        result = exists
        result
        """,
        %{
          "css_code" => css_code,
          "selector" => selector
        }
      )

    Pythonx.decode(result)
  end

  @doc """
  Gets the CSS properties for a specific selector if it exists, or returns nil.

  ## Parameters

    * `css_code` - The CSS code as a string
    * `selector` - The CSS selector to check for

  ## Examples

      iex> IgniterJs.Parsers.CSS.Parser.get_selector_properties(css_code, ".header")
      %{"color" => "blue", "font-size" => "16px"}

      iex> IgniterJs.Parsers.CSS.Parser.get_selector_properties(css_code, "#nonexistent")
      nil
  """
  def get_selector_properties(css_code, selector)
      when is_binary(css_code) and is_binary(selector) do
    {result, _globals} =
      Pythonx.eval(
        """
        import tinycss2
        from css_tools.parser import parse_stylesheet, get_selector_text, get_rule_declarations

        if isinstance(selector, bytes):
            selector = selector.decode('utf-8')

        rules = parse_stylesheet(css_code)
        properties = None

        for rule in rules:
            if rule.type == "qualified-rule":
                rule_selector = get_selector_text(rule)
                if rule_selector == selector:
                    declarations = get_rule_declarations(rule)
                    properties = {}

                    for decl in declarations:
                        if decl.type == "declaration":
                            value = tinycss2.serialize(decl.value).strip()
                            properties[decl.name] = value

                    break

        result = properties
        result
        """,
        %{
          "css_code" => css_code,
          "selector" => selector
        }
      )

    Pythonx.decode(result)
  end
end
