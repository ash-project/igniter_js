defmodule IgniterJs.CSS.CssProcessor do
  @moduledoc """
  A module that provides higher-level CSS processing functionality by leveraging
  the CSS.Parser module.
  """

  alias IgniterJs.Parsers.CSS.Parser

  @doc """
  Processes a CSS file for production by:
  1. Adding vendor prefixes for browser compatibility
  2. Removing duplicate rules
  3. Sorting properties for better diff comparison
  4. Minifying the CSS

  ## Parameters

    * `css_content` - The CSS content as a string
    * `opts` - Options for processing:
      * `:minify` - Whether to minify the output (default: `true`)
      * `:add_prefixes` - Whether to add vendor prefixes (default: `true`)
      * `:sort` - Whether to sort properties (default: `true`)
      * `:remove_duplicates` - Whether to remove duplicates (default: `true`)

  ## Returns

  The processed CSS as a string
  """
  def process_for_production(css_content, opts \\ []) do
    # Default options
    opts =
      Keyword.merge(
        [
          minify: true,
          add_prefixes: true,
          sort: true,
          remove_duplicates: true
        ],
        opts
      )

    # Process the CSS according to options
    css_content
    |> maybe_add_prefixes(opts[:add_prefixes])
    |> maybe_remove_duplicates(opts[:remove_duplicates])
    |> maybe_sort_properties(opts[:sort])
    |> maybe_minify(opts[:minify])
  end

  @doc """
  Processes CSS for development with beautification and analytics.

  ## Parameters

    * `css_content` - The CSS content as a string
    * `html_content` - Optional HTML content to check for unused selectors

  ## Returns

  A map with the processed CSS and analysis information:

  ```
  %{
    beautified_css: "the beautified CSS",
    analytics: %{...CSS analytics...},
    unused_selectors: [...list of unused selectors...],
    colors_used: %{...map of colors...},
    fonts: %{...font information...}
  }
  ```
  """
  def process_for_development(css_content, html_content \\ nil) do
    # Start with beautification
    beautified = Parser.beautify(css_content)

    # Collect analytics
    analytics = Parser.analyze_css(css_content)

    # Extract colors and fonts
    colors = Parser.extract_colors(css_content)
    fonts = Parser.extract_fonts(css_content)

    # Find unused selectors if HTML is provided
    unused_selectors =
      if html_content do
        Parser.find_unused_selectors(css_content, html_content)
      else
        []
      end

    # Return results
    %{
      beautified_css: beautified,
      analytics: analytics,
      unused_selectors: unused_selectors,
      colors_used: colors,
      fonts: fonts
    }
  end

  @doc """
  Applies browser compatibility fixes to CSS.

  Makes CSS work across browsers by:
  1. Adding vendor prefixes for properties that need them
  2. Adding standard fallbacks for newer CSS features
  3. Adding the .hide-scrollbar modifier as needed

  ## Returns

  The CSS with compatibility fixes applied
  """
  def apply_browser_compatibility(css_content) do
    # Properties that need vendor prefixes
    properties_needing_prefixes = [
      {"user-select", ["-webkit-", "-moz-", "-ms-"]},
      {"appearance", ["-webkit-", "-moz-"]},
      {"backdrop-filter", ["-webkit-"]},
      {"text-size-adjust", ["-webkit-", "-ms-"]},
      {"font-smoothing", ["-webkit-", "-moz-osx-"]}
    ]

    # Start with the original CSS
    css_with_prefixes = css_content

    # Add each set of prefixes
    css_with_prefixes =
      Enum.reduce(properties_needing_prefixes, css_with_prefixes, fn {property, prefixes}, css ->
        Parser.add_vendor_prefixes(css, property, prefixes)
      end)

    # Add the hide-scrollbar property
    css_with_hide_scrollbar = Parser.add_hide_scrollbar_property(css_with_prefixes)

    css_with_hide_scrollbar
  end

  @doc """
  Extracts critical CSS by identifying and extracting all styles needed for above-the-fold content.

  ## Parameters

    * `css_content` - The full CSS content as a string
    * `critical_selectors` - List of selectors considered critical for above-the-fold content

  ## Returns

  A tuple with `{critical_css, non_critical_css}`
  """
  def extract_critical_css(css_content, critical_selectors) do
    # Use Enum.reduce to accumulate both results in a single pass
    {critical_css, non_critical_css} =
      Enum.reduce(
        critical_selectors,
        # Initial accumulator: {critical_css, non_critical_css}
        {"", css_content},
        fn selector, {critical_acc, non_critical_acc} ->
          # Find selectors in the CSS that match this critical selector
          {result, _globals} =
            Pythonx.eval(
              """
              import tinycss2
              from css_tools.parser import parse_stylesheet, get_selector_text, get_rule_declarations

              rules = parse_stylesheet(css_code)
              matching_rules = []

              for rule in rules:
                  if rule.type == "qualified-rule":
                      selector = get_selector_text(rule)
                      if selector == critical_selector or critical_selector in selector.split(','):
                          declarations = get_rule_declarations(rule)
                          serialized_content = tinycss2.serialize(declarations).strip()
                          serialized_content = "\\n".join("    " + line.strip() for line in serialized_content.splitlines() if line.strip())
                          formatted_rule = f"{selector} {{\\n{serialized_content}\\n}}\\n"
                          matching_rules.append(formatted_rule)

              result = "\\n".join(matching_rules)
              result
              """,
              %{"css_code" => non_critical_acc, "critical_selector" => selector}
            )

          # Extract the matching rules
          matching_css = Pythonx.decode(result)

          # Update both critical and non-critical CSS
          updated_critical = critical_acc <> matching_css <> "\n"
          updated_non_critical = Parser.remove_selector(non_critical_acc, selector)

          # Return updated tuple for next iteration
          {updated_critical, updated_non_critical}
        end
      )

    # Return the final result
    {critical_css, non_critical_css}
  end

  @doc """
  Merges multiple CSS files into one optimized stylesheet.

  ## Parameters

    * `css_files` - Map of `{filename, content}` pairs
    * `opts` - Options (same as process_for_production)

  ## Returns

  The merged and optimized CSS
  """
  def merge_css_files(css_files, opts \\ []) do
    # Extract contents
    css_contents = Map.values(css_files)

    # Merge the CSS files
    merged = Parser.merge_stylesheets(css_contents)

    # Process the merged result for production
    process_for_production(merged, opts)
  end

  # Helper functions for conditional processing

  defp maybe_add_prefixes(css, true) do
    apply_browser_compatibility(css)
  end

  defp maybe_add_prefixes(css, false), do: css

  defp maybe_remove_duplicates(css, true) do
    Parser.remove_duplicates(css)
  end

  defp maybe_remove_duplicates(css, false), do: css

  defp maybe_sort_properties(css, true) do
    Parser.sort_properties(css)
  end

  defp maybe_sort_properties(css, false), do: css

  defp maybe_minify(css, true) do
    Parser.minify(css)
  end

  defp maybe_minify(css, false), do: css
end
