defmodule IgniterJSTest.Parsers.Css.ParserTest do
  use ExUnit.Case
  alias IgniterJs.Parsers.CSS.Parser

  describe "add_hide_scrollbar_property/1" do
    test "adds display: none to existing .hide-scrollbar class" do
      # Given: CSS with .hide-scrollbar class but without display: none
      css_code = """
      .header {
        color: blue;
      }

      .hide-scrollbar {
        scrollbar-width: none; /* Firefox */
      }
      """

      # When: Adding hide-scrollbar property
      {:ok, _, result} = Parser.add_hide_scrollbar_property(css_code)

      # Then: The .hide-scrollbar class should have display: none added
      assert String.contains?(result, ".hide-scrollbar")
      assert String.contains?(result, "display: none")
      # Original property preserved
      assert String.contains?(result, "scrollbar-width: none")
      # Other selectors preserved
      assert String.contains?(result, ".header")
    end

    test "creates .hide-scrollbar class when it doesn't exist" do
      # Given: CSS without .hide-scrollbar class
      css_code = """
      .header {
        color: blue;
      }

      .content {
        padding: 20px;
      }
      """

      # When: Adding hide-scrollbar property
      {:ok, _, result} = Parser.add_hide_scrollbar_property(css_code)

      # Then: The .hide-scrollbar class should be created with display: none
      assert String.contains?(result, ".hide-scrollbar")
      assert String.contains?(result, "display: none")
      # Original content preserved
      assert String.contains?(result, ".header")
      # Original content preserved
      assert String.contains?(result, ".content")
    end

    test "updates existing display property in .hide-scrollbar class" do
      # Given: CSS with .hide-scrollbar class that has a different display value
      css_code = """
      .hide-scrollbar {
        display: flex;
      }
      """

      # When: Adding hide-scrollbar property
      {:ok, _, result} = Parser.add_hide_scrollbar_property(css_code)

      # Then: The display property should be updated to none
      assert String.contains?(result, ".hide-scrollbar")
      assert String.contains?(result, "display: none")
      # Old value removed
      refute String.contains?(result, "display: flex")
    end

    test "works with empty CSS" do
      # Given: Empty CSS
      css_code = ""

      # When: Adding hide-scrollbar property
      {:ok, _, result} = Parser.add_hide_scrollbar_property(css_code)

      # Then: The .hide-scrollbar class should be created with display: none
      assert String.contains?(result, ".hide-scrollbar")
      assert String.contains?(result, "display: none")
    end

    test "handles CSS with comments" do
      # Given: CSS with comments
      css_code = """
      /* Header styles */
      .header {
        color: blue;
      }

      /* This class hides scrollbars */
      .hide-scrollbar {
        /* Firefox */
        scrollbar-width: none;
      }
      """

      # When: Adding hide-scrollbar property
      {:ok, _, result} = Parser.add_hide_scrollbar_property(css_code)

      # Then: Comments should be preserved and display: none added
      assert String.contains?(result, "/* Header styles */")
      assert String.contains?(result, "/* This class hides scrollbars */")
      assert String.contains?(result, "/* Firefox */")
      assert String.contains?(result, ".hide-scrollbar")
      assert String.contains?(result, "display: none")

      {:error, _, "Failed to parse CSS: Can not serialize <ParseError invalid>"} =
        assert Parser.add_hide_scrollbar_property("1")
    end
  end

  describe "add_vendor_prefixes/3" do
    test "adds vendor prefixes to existing property" do
      # Given: CSS with user-select property
      css_code = """
      .selectable {
        user-select: none;
        color: blue;
      }
      """

      # When: Adding vendor prefixes
      prefixes = ["-webkit-", "-moz-", "-ms-"]
      {:ok, _, result} = Parser.add_vendor_prefixes(css_code, "user-select", prefixes)

      # Then: Prefixed properties should be added
      assert String.contains?(result, "-webkit-user-select: none")
      assert String.contains?(result, "-moz-user-select: none")
      assert String.contains?(result, "-ms-user-select: none")
      # Original property should be preserved
      assert String.contains?(result, "user-select: none")
      # Other properties should be preserved
      assert String.contains?(result, "color: blue")
    end

    test "does nothing when property doesn't exist" do
      # Given: CSS without the target property
      css_code = """
      .header {
        color: blue;
        font-size: 16px;
      }
      """

      # When: Adding vendor prefixes for a non-existent property
      prefixes = ["-webkit-", "-moz-"]
      {:ok, _, result} = Parser.add_vendor_prefixes(css_code, "user-select", prefixes)

      # Then: CSS should remain unchanged
      assert Parser.beautify(result) == Parser.beautify(css_code)
    end

    test "handles multiple occurrences of the property" do
      # Given: CSS with multiple elements having the same property
      css_code = """
      .one {
        user-select: none;
      }
      .two {
        user-select: text;
      }
      """

      # When: Adding vendor prefixes
      prefixes = ["-webkit-", "-moz-"]
      {:ok, _, result} = Parser.add_vendor_prefixes(css_code, "user-select", prefixes)

      # Then: All occurrences should be prefixed
      assert String.contains?(result, ".one {")
      assert String.contains?(result, "-webkit-user-select: none")
      assert String.contains?(result, "-moz-user-select: none")
      assert String.contains?(result, "user-select: none")

      assert String.contains?(result, ".two {")
      assert String.contains?(result, "-webkit-user-select: text")
      assert String.contains?(result, "-moz-user-select: text")
      assert String.contains?(result, "user-select: text")
    end

    test "works with empty prefixes list" do
      # Given: CSS with a property
      css_code = """
      .box {
        user-select: none;
      }
      """

      # When: Adding an empty list of prefixes
      prefixes = []
      {:ok, _, result} = Parser.add_vendor_prefixes(css_code, "user-select", prefixes)

      # Then: CSS should remain unchanged
      assert Parser.beautify(result) == Parser.beautify(css_code)
    end

    test "preserves !important flags" do
      # Given: CSS with !important
      css_code = """
      .important {
        user-select: none !important;
      }
      """

      # When: Adding vendor prefixes
      prefixes = ["-webkit-", "-moz-"]
      {:ok, _, result} = Parser.add_vendor_prefixes(css_code, "user-select", prefixes)

      # Then: !important should be preserved in all versions
      assert String.contains?(result, "-webkit-user-select: none !important")
      assert String.contains?(result, "-moz-user-select: none !important")
      assert String.contains?(result, "user-select: none !important")
    end

    test "handles CSS with comments" do
      # Given: CSS with comments
      css_code = """
      /* Header styles */
      .header {
        /* Prevent selection */
        user-select: none;
      }
      """

      # When: Adding vendor prefixes
      prefixes = ["-webkit-", "-moz-"]
      {:ok, _, result} = Parser.add_vendor_prefixes(css_code, "user-select", prefixes)

      # Then: Comments should be preserved
      assert String.contains?(result, "/* Header styles */")
      assert String.contains?(result, "/* Prevent selection */")
      assert String.contains?(result, "-webkit-user-select: none")
      assert String.contains?(result, "-moz-user-select: none")
    end

    test "handles properties with multiple values" do
      # Given: CSS with property having multiple values
      css_code = """
      .complex {
        transform: translateX(10px) rotate(45deg);
      }
      """

      # When: Adding vendor prefixes
      prefixes = ["-webkit-", "-moz-"]
      {:ok, _, result} = Parser.add_vendor_prefixes(css_code, "transform", prefixes)

      # Then: Complex values should be preserved
      assert String.contains?(result, "-webkit-transform: translateX(10px) rotate(45deg)")
      assert String.contains?(result, "-moz-transform: translateX(10px) rotate(45deg)")
      assert String.contains?(result, "transform: translateX(10px) rotate(45deg)")
    end

    test "handles media queries" do
      # Given: CSS with media queries
      css_code = """
      @media (max-width: 768px) {
        .mobile {
          user-select: none;
        }
      }
      """

      # When: Adding vendor prefixes
      prefixes = ["-webkit-", "-moz-"]
      {:ok, _, result} = Parser.add_vendor_prefixes(css_code, "user-select", prefixes)

      # Then: Properties inside media queries should be prefixed
      assert String.contains?(result, "@media (max-width: 768px)")
      assert String.contains?(result, "-webkit-user-select: none")
      assert String.contains?(result, "-moz-user-select: none")
    end

    test "handles invalid CSS" do
      # Given: Invalid CSS
      css_code = "invalid { css syntax"
      # When: Adding vendor prefixes
      prefixes = ["-webkit-", "-moz-"]
      {:error, _, error_message} = Parser.add_vendor_prefixes(css_code, "user-select", prefixes)

      # Then: Should return an error
      assert String.contains?(error_message, "Failed to parse CSS")
    end
  end

  describe "analyze_css/1" do
    test "returns analysis for valid CSS with multiple selectors" do
      # Given: CSS with multiple selectors, properties, and values
      css_code = """
      .header {
        color: #333;
        font-size: 16px;
      }

      #main-content {
        margin: 0 auto;
        width: 100%;
        max-width: 1200px;
      }

      .footer {
        background-color: #f5f5f5;
        padding: 20px;
      }
      """

      # When: Analyzing the CSS
      {:ok, _, result} = Parser.analyze_css(css_code)

      # Then: Result should include comprehensive analysis
      assert is_map(result)
      assert result["selectors_count"] == 3
      assert result["properties_count"] >= 7
      assert is_list(result["selectors"])
      assert ".header" in result["selectors"]
      assert "#main-content" in result["selectors"]
      assert ".footer" in result["selectors"]

      # Check for specific property values
      assert ".header" in Map.keys(result["selector_properties"])
      assert "#333" in Map.values(result["selector_properties"][".header"])
    end

    test "handles CSS with media queries" do
      # Given: CSS with media queries
      css_code = """
      @media (max-width: 768px) {
        .mobile {
          display: block;
          font-size: 14px;
        }
      }

      @media print {
        .no-print {
          display: none;
        }
      }
      """

      # When: Analyzing the CSS
      {:ok, _, result} = Parser.analyze_css(css_code)

      # Then: Media queries should be properly analyzed
      assert is_map(result)

      assert result["media_queries_count"] == 2
    end

    test "analyzes CSS with complex selectors" do
      # Given: CSS with complex selectors
      css_code = """
      .parent > .child {
        color: blue;
      }

      .sibling + .adjacent {
        margin-left: 10px;
      }

      ul li:hover {
        background-color: #f0f0f0;
      }

      input[type="text"] {
        border: 1px solid #ccc;
      }
      """

      # When: Analyzing the CSS
      {:ok, _, result} = Parser.analyze_css(css_code)

      # Then: Complex selectors should be analyzed correctly
      assert is_map(result)
      assert result["selectors_count"] == 4
      assert ".parent > .child" in result["selectors"]
      assert ".sibling + .adjacent" in result["selectors"]
      assert "ul li:hover" in result["selectors"]

      assert "input[type=\"text\"]" in result["selectors"] or
               "input[type=\"text\"]" in result["selectors"]
    end

    test "analyzes CSS for color usage" do
      # Given: CSS with various color formats
      css_code = """
      .hex-color {
        color: #ff0000;
      }

      .rgb-color {
        color: rgb(0, 128, 255);
      }

      .rgba-color {
        background-color: rgba(255, 255, 255, 0.8);
      }

      .named-color {
        border-color: blue;
      }
      """

      # When: Analyzing the CSS
      {:ok, _, result} = Parser.analyze_css(css_code)

      # Then: Color analysis should be included

      assert result["colors_used"] >= 4
    end

    test "analyzes empty CSS" do
      # Given: Empty CSS
      css_code = ""

      # When: Analyzing the CSS
      {:ok, _, result} = Parser.analyze_css(css_code)

      # Then: Analysis should handle empty CSS gracefully
      assert is_map(result)
      assert result["selectors_count"] == 0
      assert result["properties_count"] == 0
      assert length(result["selectors"]) == 0
    end

    test "analyzes CSS with comments" do
      css_code = """
      /* Header styles */
      .header {
        color: black;
      }

      /* Main content area */
      .content {
        /* Inner padding */
        padding: 20px;
      }
      """

      # When: Analyzing the CSS
      {:ok, _, result} = Parser.analyze_css(css_code)

      # Then: Comments should be properly analyzed
      assert result["comments_count"] >= 2
    end

    test "handles invalid CSS" do
      # Given: Invalid CSS
      css_code = ".invalid { color: red; missing-closing-brace;"

      # When: Analyzing the CSS
      {:error, _, error_message} = Parser.analyze_css(css_code)

      # Then: Should return an error
      assert is_binary(error_message)
      assert String.contains?(error_message, "Failed to parse CSS")
    end

    test "analyzes CSS with imports" do
      # Given: CSS with import statements
      css_code = """
      @import url('fonts.css');
      @import 'typography.css' screen and (min-width: 800px);

      .content {
        font-family: 'Open Sans', sans-serif;
      }
      """

      # When: Analyzing the CSS
      {:ok, _, result} = Parser.analyze_css(css_code)

      # Then: Import statements should be analyzed
      assert result["imports_count"] == 2
      assert "fonts.css" in result["imports"]
      assert "typography.css" in result["imports"]

      # Check media queries for imports
      assert is_map(result["import_media_queries"])
      assert "typography.css" in Map.keys(result["import_media_queries"])
      assert "screen and (min-width: 800px)" in Map.values(result["import_media_queries"])
    end
  end
end
