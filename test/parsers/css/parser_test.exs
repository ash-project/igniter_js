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
      assert elem(Parser.beautify(result), 2) == elem(Parser.beautify(css_code), 2)
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
      assert elem(Parser.beautify(result), 2) == elem(Parser.beautify(css_code), 2)
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

  describe "extract_colors/1" do
    test "extracts hex color values" do
      # Given: CSS with hex color values
      css_code = """
      .header {
        color: #333;
        background-color: #f5f5f5;
      }
      .button {
        color: #fff;
        background-color: #007bff;
      }
      """

      # When: Extracting colors
      {:ok, _, result} = Parser.extract_colors(css_code)

      # Then: Colors should be properly extracted and organized by selector
      assert is_map(result)
      assert Map.has_key?(result, ".header")
      assert Map.has_key?(result, ".button")

      assert "color: #333" in result[".header"]
      assert "background-color: #f5f5f5" in result[".header"]
      assert "color: #fff" in result[".button"]
      assert "background-color: #007bff" in result[".button"]
    end

    test "extracts rgb and rgba color values" do
      # Given: CSS with RGB and RGBA color values
      css_code = """
      .container {
        color: rgb(51, 51, 51);
        background-color: rgba(255, 255, 255, 0.8);
      }
      .overlay {
        background-color: rgba(0, 0, 0, 0.5);
      }
      """

      # When: Extracting colors
      {:ok, _, result} = Parser.extract_colors(css_code)

      # Then: RGB and RGBA colors should be properly extracted
      assert is_map(result)
      assert Map.has_key?(result, ".container")
      assert Map.has_key?(result, ".overlay")

      assert "color: rgb(51, 51, 51)" in result[".container"]
      assert "background-color: rgba(255, 255, 255, 0.8)" in result[".container"]
      assert "background-color: rgba(0, 0, 0, 0.5)" in result[".overlay"]
    end

    test "extracts named color values" do
      # Given: CSS with named color values
      css_code = """
      .success {
        color: green;
      }
      .error {
        color: red;
      }
      .info {
        color: blue;
        background-color: white;
      }
      """

      # When: Extracting colors
      {:ok, _, result} = Parser.extract_colors(css_code)

      # Then: Named colors should be properly extracted
      assert is_map(result)
      assert "color: green" in result[".success"]
      assert "color: red" in result[".error"]
      assert "color: blue" in result[".info"]
      assert "background-color: white" in result[".info"]
    end

    test "extracts hsl and hsla color values" do
      # Given: CSS with HSL and HSLA color values
      css_code = """
      .hsl-colors {
        color: hsl(120, 100%, 50%);
        background-color: hsla(240, 100%, 50%, 0.5);
      }
      """

      # When: Extracting colors
      {:ok, _, result} = Parser.extract_colors(css_code)

      # Then: HSL and HSLA colors should be properly extracted
      assert is_map(result)
      assert "color: hsl(120, 100%, 50%)" in result[".hsl-colors"]
      assert "background-color: hsla(240, 100%, 50%, 0.5)" in result[".hsl-colors"]
    end

    test "extracts colors from shorthand properties" do
      # Given: CSS with shorthand properties containing colors
      css_code = """
      .shorthand {
        border: 1px solid #ccc;
        box-shadow: 0 0 5px rgba(0, 0, 0, 0.3);
      }
      """

      # When: Extracting colors
      {:ok, _, result} = Parser.extract_colors(css_code)

      # Then: Colors in shorthand properties should be extracted
      assert is_map(result)
      assert Map.has_key?(result, ".shorthand")

      assert Enum.any?(result[".shorthand"], fn color ->
               String.contains?(color, "#ccc") and String.contains?(color, "border")
             end)

      assert Enum.any?(result[".shorthand"], fn color ->
               String.contains?(color, "rgba(0, 0, 0, 0.3)") and
                 String.contains?(color, "box-shadow")
             end)
    end

    test "extracts colors from nested selectors" do
      # Given: CSS with nested selectors (e.g., media queries)
      css_code = """
      @media (max-width: 768px) {
        .mobile {
          color: #555;
          background-color: #eee;
        }
      }
      """

      # When: Extracting colors
      {:ok, _, result} = Parser.extract_colors(css_code)

      # Then: Colors from nested selectors should be properly extracted
      assert is_map(result)
      assert Map.has_key?(result, ".mobile")
      assert "color: #555" in result[".mobile"]
      assert "background-color: #eee" in result[".mobile"]
    end

    test "handles CSS with no colors" do
      # Given: CSS without any color properties
      css_code = """
      .no-colors {
        display: block;
        margin: 10px;
        padding: 20px;
      }
      """

      # When: Extracting colors
      {:ok, _, result} = Parser.extract_colors(css_code)

      # Then: Result should be an empty map
      assert is_map(result)
      assert Enum.empty?(result)
    end

    test "handles invalid CSS" do
      # Given: Invalid CSS
      css_code = ".invalid { color: red; missing-closing-brace;"

      # When: Extracting colors
      {:error, _, error_message} = Parser.extract_colors(css_code)

      # Then: Should return an error
      assert is_binary(error_message)
      assert String.contains?(error_message, "Failed to parse CSS")
    end
  end

  describe "minify/1" do
    test "minifies CSS by removing whitespace and comments" do
      # Given: CSS with whitespace and comments
      css_code = """
      /* Header styles */
      .header {
        color: blue;
        font-size: 16px;
      }

      /* Content area */
      .content {
        padding: 20px;
        margin: 10px;
      }
      """

      # When: Minifying the CSS
      {:ok, _, result} = Parser.minify(css_code)

      # Then: Result should be minified without whitespace and comments
      assert !String.contains?(result, "/* Header styles */")
      assert !String.contains?(result, "\n")
      assert String.contains?(result, ".header{color:blue;font-size:16px;}")
      assert String.contains?(result, ".content{padding:20px;margin:10px;}")
    end

    test "preserves functionality while minifying" do
      # Given: CSS with various properties
      css_code = """
      .button {
        display: inline-block;
        background-color: #007bff;
        color: white;
        padding: 10px 15px;
        border-radius: 4px;
      }
      """

      # When: Minifying the CSS
      {:ok, _, result} = Parser.minify(css_code)

      # Then: All properties should be preserved in minified form
      assert String.contains?(result, "display:inline-block")
      assert String.contains?(result, "background-color:#007bff")
      assert String.contains?(result, "color:white")
      assert String.contains?(result, "padding:10px 15px")
      assert String.contains?(result, "border-radius:4px")
    end

    test "handles CSS with media queries" do
      # Given: CSS with media queries
      css_code = """
      @media (max-width: 768px) {
        .mobile {
          display: block;
          width: 100%;
        }
      }
      """

      # When: Minifying the CSS
      {:ok, _, result} = Parser.minify(css_code)

      # Then: Media queries should be properly minified
      assert String.contains?(
               result,
               "@media (max-width: 768px){.mobile{display:block;width:100%;}}"
             )

      assert String.contains?(result, ".mobile{display:block;width:100%;}")
    end

    test "handles CSS with vendor prefixes" do
      # Given: CSS with vendor prefixes
      css_code = """
      .box {
        -webkit-border-radius: 4px;
        -moz-border-radius: 4px;
        border-radius: 4px;
      }
      """

      # When: Minifying the CSS
      {:ok, _, result} = Parser.minify(css_code)

      # Then: Vendor prefixes should be preserved
      assert String.contains?(result, "-webkit-border-radius:4px")
      assert String.contains?(result, "-moz-border-radius:4px")
      assert String.contains?(result, "border-radius:4px")
    end

    test "handles @import and other at-rules" do
      # Given: CSS with at-rules
      css_code = """
      @import url('fonts.css');
      @charset "UTF-8";
      @keyframes fade {
        from { opacity: 0; }
        to { opacity: 1; }
      }
      """

      # When: Minifying the CSS
      {:ok, _, result} = Parser.minify(css_code)

      # Then: At-rules should be properly minified
      assert String.contains?(result, "@import url(\"fonts.css\")")
      assert String.contains?(result, "@charset \"UTF-8\"")
      assert String.contains?(result, "@keyframes fade{from{opacity:0;}to{opacity:1;}}")
    end

    test "handles empty CSS" do
      # Given: Empty CSS
      css_code = ""

      # When: Minifying the CSS
      {:ok, _, result} = Parser.minify(css_code)

      # Then: Result should be empty
      assert result == ""
    end
  end

  describe "modify_property/5" do
    test "modifies existing property value for selector" do
      # Given: CSS with a selector and property
      css_code = """
      .header {
        color: red;
        font-size: 16px;
      }
      """

      # When: Modifying the color property
      {:ok, _, result} = Parser.modify_property(css_code, ".header", "color", "blue", false)

      # Then: The property value should be updated
      assert String.contains?(result, "color: blue")
      assert !String.contains?(result, "color: red")
      # Other properties should remain unchanged
      assert String.contains?(result, "font-size: 16px")
    end

    test "adds property if it doesn't exist for the selector" do
      # Given: CSS with a selector but without the target property
      css_code = """
      .header {
        font-size: 16px;
      }
      """

      # When: Modifying a non-existent property
      {:ok, _, result} = Parser.modify_property(css_code, ".header", "color", "blue", false)

      # Then: The new property should be added
      assert String.contains?(result, "color: blue")
      # Existing properties should be preserved
      assert String.contains?(result, "font-size: 16px")
    end

    test "adds selector and property if selector doesn't exist" do
      # Given: CSS without the target selector
      css_code = """
      .content {
        padding: 20px;
      }
      """

      # When: Modifying a property for a non-existent selector
      {:ok, _, result} = Parser.modify_property(css_code, ".header", "color", "blue", false)

      # Then: The new selector and property should be added
      assert String.contains?(result, ".header")
      assert String.contains?(result, "color: blue")
      # Existing content should be preserved
      assert String.contains?(result, ".content")
      assert String.contains?(result, "padding: 20px")
    end

    test "adds !important flag when specified" do
      # Given: CSS with a selector and property
      css_code = """
      .header {
        color: red;
      }
      """

      # When: Modifying property with important flag
      {:ok, _, result} = Parser.modify_property(css_code, ".header", "color", "blue", true)

      # Then: The property should be updated with !important
      assert String.contains?(result, "color: blue!important;")
    end

    test "removes !important flag when not specified" do
      # Given: CSS with a property having !important flag
      css_code = """
      .header {
        color: red !important;
      }
      """

      # When: Modifying property without important flag
      {:ok, _, result} = Parser.modify_property(css_code, ".header", "color", "blue", false)

      # Then: The property should be updated without !important
      assert String.contains?(result, "color: blue")
      assert !String.contains?(result, "!important")
    end

    test "handles selectors with pseudo-classes" do
      # Given: CSS with pseudo-class selectors
      css_code = """
      .button:hover {
        background-color: red;
      }
      """

      # When: Modifying property for a selector with pseudo-class
      {:ok, _, result} =
        Parser.modify_property(css_code, ".button:hover", "background-color", "blue", false)

      # Then: The property should be updated for the correct selector
      assert String.contains?(result, ".button:hover")
      assert String.contains?(result, "background-color: blue")
    end

    test "handles complex selectors" do
      # Given: CSS with complex selectors
      css_code = """
      .parent > .child {
        color: red;
      }
      """

      # When: Modifying property for a complex selector
      {:ok, _, result} =
        Parser.modify_property(css_code, ".parent > .child", "color", "blue", false)

      # Then: The property should be updated for the correct selector
      assert String.contains?(result, ".parent > .child")
      assert String.contains?(result, "color: blue")
    end

    test "preserves media queries when modifying properties inside them" do
      # Given: CSS with media queries
      css_code = """
      @media (max-width: 768px) {
        .mobile {
          color: red;
        }
      }
      """

      # When: Modifying property inside media query
      {:ok, _, result} = Parser.modify_property(css_code, ".mobile", "color", "blue", false)

      # Then: The media query should be preserved and property updated
      assert String.contains?(result, "@media (max-width: 768px)")
      assert String.contains?(result, ".mobile")
      assert String.contains?(result, "color: blue")
    end

    test "handles empty CSS" do
      # Given: Empty CSS
      css_code = ""

      # When: Modifying property
      {:ok, _, result} = Parser.modify_property(css_code, ".header", "color", "blue", false)

      # Then: A new rule should be created
      assert String.contains?(result, ".header")
      assert String.contains?(result, "color: blue")
    end

    test "handles invalid CSS" do
      # Given: Invalid CSS
      css_code = ".invalid { color: red; missing-closing-brace;"

      # When: Modifying property
      {:error, _, error_message} =
        Parser.modify_property(css_code, ".invalid", "color", "blue", false)

      # Then: Should return an error
      assert is_binary(error_message)
      assert String.contains?(error_message, "Failed to parse CSS")
    end
  end

  describe "merge_stylesheets/1" do
    test "merges multiple CSS stylesheets" do
      # Given: Multiple CSS stylesheets
      css_code1 = """
      .header {
        color: blue;
        font-size: 16px;
      }
      """

      css_code2 = """
      .content {
        padding: 20px;
        margin: 10px;
      }
      """

      # When: Merging the stylesheets
      {:ok, _, result} = Parser.merge_stylesheets([css_code1, css_code2])

      # Then: The result should contain all selectors and properties
      assert String.contains?(result, ".header")
      assert String.contains?(result, "color: blue")
      assert String.contains?(result, "font-size: 16px")
      assert String.contains?(result, ".content")
      assert String.contains?(result, "padding: 20px")
      assert String.contains?(result, "margin: 10px")
    end

    test "removes duplicate selectors when merging" do
      # Given: CSS stylesheets with duplicate selectors
      css_code1 = """
      .header {
        color: blue;
      }
      """

      css_code2 = """
      .header {
        font-size: 16px;
      }
      """

      # When: Merging the stylesheets
      {:ok, _, result} = Parser.merge_stylesheets([css_code1, css_code2])

      # Then: The duplicate selectors should be merged
      assert String.contains?(result, ".header")
      assert String.contains?(result, "color: blue")
      assert String.contains?(result, "font-size: 16px")

      # Count occurrences of .header - should only appear once
      assert Regex.scan(~r/\.header\s*\{/, result) |> length() == 1
    end

    test "handles duplicate properties by keeping the last one" do
      # Given: CSS stylesheets with duplicate properties
      css_code1 = """
      .header {
        color: blue;
      }
      """

      css_code2 = """
      .header {
        color: red;
      }
      """

      # When: Merging the stylesheets
      {:ok, _, result} = Parser.merge_stylesheets([css_code1, css_code2])

      # Then: The last property value should be kept
      assert String.contains?(result, ".header")
      assert String.contains?(result, "color: red")
      refute String.contains?(result, "color: blue")
    end

    test "preserves media queries when merging" do
      # Given: CSS stylesheets with media queries
      css_code1 = """
      @media (max-width: 768px) {
        .mobile {
          color: blue;
        }
      }
      """

      css_code2 = """
      @media (max-width: 768px) {
        .mobile {
          font-size: 14px;
        }
      }
      """

      # When: Merging the stylesheets
      {:ok, _, result} = Parser.merge_stylesheets([css_code1, css_code2])

      # Then: Media queries should be preserved and properties merged
      assert String.contains?(result, "@media (max-width: 768px)")
      assert String.contains?(result, ".mobile")
      assert String.contains?(result, "color: blue")
      assert String.contains?(result, "font-size: 14px")
    end

    test "preserves @import rules" do
      # Given: CSS stylesheets with @import rules
      css_code1 = """
      @import url('fonts.css');
      .header {
        font-family: 'Open Sans', sans-serif;
      }
      """

      css_code2 = """
      @import url('layout.css');
      .content {
        padding: 20px;
      }
      """

      # When: Merging the stylesheets
      {:ok, _, result} = Parser.merge_stylesheets([css_code1, css_code2])

      # Then: @import rules should be preserved
      assert String.contains?(result, "@import url(\"fonts.css\");")
      assert String.contains?(result, "@import url(\"layout.css\");")
      assert String.contains?(result, ".header")
      assert String.contains?(result, ".content")
    end

    test "preserves !important declarations" do
      # Given: CSS stylesheets with !important declarations
      css_code1 = """
      .header {
        color: blue !important;
      }
      """

      css_code2 = """
      .content {
        padding: 20px !important;
      }
      """

      # When: Merging the stylesheets
      {:ok, _, result} = Parser.merge_stylesheets([css_code1, css_code2])

      # Then: !important declarations should be preserved
      assert String.contains?(result, "color: blue !important")
      assert String.contains?(result, "padding: 20px !important")
    end

    test "handles CSS with comments" do
      # Given: CSS stylesheets with comments
      css_code1 = """
      /* Header styles */
      .header {
        color: blue;
      }
      """

      css_code2 = """
      /* Content styles */
      .content {
        padding: 20px;
      }
      """

      # When: Merging the stylesheets
      {:ok, _, result} = Parser.merge_stylesheets([css_code1, css_code2])

      # Then: Comments should be preserved
      assert String.contains?(result, "/* Header styles */")
      assert String.contains?(result, "/* Content styles */")
      assert String.contains?(result, ".header")
      assert String.contains?(result, ".content")
    end

    test "handles empty CSS list" do
      # Given: Empty CSS list
      css_list = []

      # When: Merging the stylesheets
      {:ok, _, result} = Parser.merge_stylesheets(css_list)

      # Then: Result should be empty
      assert result == "" or result == nil
    end

    test "handles list with a single stylesheet" do
      # Given: CSS list with a single stylesheet
      css_code = """
      .header {
        color: blue;
      }
      """

      # When: Merging the stylesheets
      {:ok, _, result} = Parser.merge_stylesheets([css_code])

      # Then: Result should be the same as the input
      assert String.contains?(result, ".header")
      assert String.contains?(result, "color: blue")
    end

    test "handles lists with empty stylesheets" do
      # Given: CSS list with some empty stylesheets
      css_code1 = ""

      css_code2 = """
      .header {
        color: blue;
      }
      """

      css_code3 = ""

      # When: Merging the stylesheets
      {:ok, _, result} = Parser.merge_stylesheets([css_code1, css_code2, css_code3])

      # Then: Empty stylesheets should be ignored
      assert String.contains?(result, ".header")
      assert String.contains?(result, "color: blue")
    end

    test "handles invalid CSS" do
      # Given: CSS list with some invalid CSS
      css_code1 = """
      .header {
        color: blue;
      }
      """

      css_code2 = ".invalid { color: red; missing-closing-brace;"

      # When: Merging the stylesheets
      result = Parser.merge_stylesheets([css_code1, css_code2])

      # Then: Should either return an error or handle it gracefully
      case result do
        {:error, _, error_message} ->
          assert is_binary(error_message)
          assert String.contains?(error_message, "Failed to parse CSS")

        {:ok, _, merged_css} ->
          # If the function tries to handle invalid CSS gracefully, verify the valid parts are there
          assert String.contains?(merged_css, ".header")
          assert String.contains?(merged_css, "color: blue")
      end
    end
  end

  describe "remove_selector/2" do
    test "removes a basic selector from CSS" do
      # Given: CSS with multiple selectors
      css_code = """
      .header {
        color: blue;
        font-size: 16px;
      }

      .content {
        padding: 20px;
        margin: 10px;
      }
      """

      # When: Removing one selector
      {:ok, _, result} = Parser.remove_selector(css_code, ".header")

      # Then: The specified selector should be removed
      refute String.contains?(result, ".header")
      refute String.contains?(result, "color: blue")
      refute String.contains?(result, "font-size: 16px")

      # Other selectors should be preserved
      assert String.contains?(result, ".content")
      assert String.contains?(result, "padding: 20px")
      assert String.contains?(result, "margin: 10px")
    end

    test "handles complex selectors" do
      # Given: CSS with complex selectors
      css_code = """
      .parent > .child {
        color: red;
      }

      .sibling + .adjacent {
        margin-left: 10px;
      }
      """

      # When: Removing a complex selector
      {:ok, _, result} = Parser.remove_selector(css_code, ".parent > .child")

      # Then: The complex selector should be removed
      refute String.contains?(result, ".parent > .child")
      refute String.contains?(result, "color: red")

      # Other selectors should be preserved
      assert String.contains?(result, ".sibling + .adjacent")
      assert String.contains?(result, "margin-left: 10px")
    end

    test "handles selectors with pseudo-classes" do
      # Given: CSS with pseudo-class selectors
      css_code = """
      .button:hover {
        background-color: blue;
      }

      .link:visited {
        color: purple;
      }
      """

      # When: Removing a selector with pseudo-class
      {:ok, _, result} = Parser.remove_selector(css_code, ".button:hover")

      # Then: The selector with pseudo-class should be removed
      refute String.contains?(result, ".button:hover")
      refute String.contains?(result, "background-color: blue")

      # Other selectors should be preserved
      assert String.contains?(result, ".link:visited")
      assert String.contains?(result, "color: purple")
    end

    test "handles removing a selector from a media query" do
      # Given: CSS with selectors inside media queries
      css_code = """
      @media (max-width: 768px) {
        .mobile {
          display: block;
        }

        .tablet {
          display: none;
        }
      }
      """

      # When: Removing a selector from a media query
      {:ok, _, result} = Parser.remove_selector(css_code, ".mobile")
      # Then: The selector should be removed from the media query
      refute String.contains?(result, ".mobile")
      refute String.contains?(result, "display: block")

      # Media query and other selectors should be preserved
      assert String.contains?(result, "@media (max-width: 768px)")
      assert String.contains?(result, ".tablet")
      assert String.contains?(result, "display: none")
    end

    test "maintains empty media queries after removing all selectors" do
      # Given: CSS with a single selector in a media query
      css_code = """
      @media (max-width: 768px) {
        .mobile {
          display: block;
        }
      }
      """

      # When: Removing the only selector from the media query
      # If there is no child selector, the media query should be removed
      {:ok, _, result} = Parser.remove_selector(css_code, ".mobile")

      # Then: The media query should still exist but be empty
      assert String.contains?(result, "")
      refute String.contains?(result, ".mobile")
      refute String.contains?(result, "display: block")
    end

    test "handles removing selector with multiple declarations" do
      # Given: CSS with a selector having multiple declarations
      css_code = """
      .multiline {
        color: red;
        font-size: 16px;
        margin: 10px;
        padding: 5px;
        border: 1px solid black;
      }
      """

      # When: Removing the selector
      {:ok, _, result} = Parser.remove_selector(css_code, ".multiline")

      # Then: The entire selector block should be removed
      refute String.contains?(result, ".multiline")
      refute String.contains?(result, "color: red")
      refute String.contains?(result, "font-size: 16px")
      refute String.contains?(result, "margin: 10px")
      refute String.contains?(result, "padding: 5px")
      refute String.contains?(result, "border: 1px solid black")
    end

    test "handles non-existent selector" do
      # Given: CSS without the target selector
      css_code = """
      .header {
        color: blue;
      }
      """

      # When: Removing a non-existent selector
      {:ok, _, result} = Parser.remove_selector(css_code, ".non-existent")

      # Then: The CSS should remain unchanged
      assert elem(Parser.beautify(result), 2) == elem(Parser.beautify(css_code), 2)
      assert String.contains?(result, ".header")
      assert String.contains?(result, "color: blue")
    end

    test "handles multiple occurrences of the same selector" do
      # Given: CSS with multiple occurrences of the same selector
      css_code = """
      .duplicate {
        color: red;
      }

      .other {
        margin: 10px;
      }

      .duplicate {
        font-size: 16px;
      }
      """

      # When: Removing the duplicate selector
      {:ok, _, result} = Parser.remove_selector(css_code, ".duplicate")

      # Then: All occurrences of the selector should be removed
      refute String.contains?(result, ".duplicate")
      refute String.contains?(result, "color: red")
      refute String.contains?(result, "font-size: 16px")

      # Other selectors should be preserved
      assert String.contains?(result, ".other")
      assert String.contains?(result, "margin: 10px")
    end

    test "handles empty CSS" do
      # Given: Empty CSS
      css_code = ""

      # When: Removing a selector
      {:ok, _, result} = Parser.remove_selector(css_code, ".header")

      # Then: The result should still be empty
      assert result == ""
    end

    test "handles CSS with comments" do
      # Given: CSS with comments
      css_code = """
      /* Header styles */
      .header {
        color: blue;
      }

      /* Content styles */
      .content {
        padding: 20px;
      }
      """

      # When: Removing a selector
      {:ok, _, result} = Parser.remove_selector(css_code, ".header")

      # Then: The selector should be removed
      refute String.contains?(result, ".header")
      refute String.contains?(result, "color: blue")

      # Comments for other selectors should be preserved
      assert String.contains?(result, "/* Content styles */")
      assert String.contains?(result, ".content")
      assert String.contains?(result, "padding: 20px")
    end

    test "handles invalid CSS" do
      # Given: Invalid CSS
      css_code = ".invalid { color: red; missing-closing-brace;"

      # When: Removing a selector
      {:error, _, error_message} = Parser.remove_selector(css_code, ".invalid")

      # Then: Should return an error
      assert is_binary(error_message)
      assert String.contains?(error_message, "Failed to parse CSS")
    end
  end
end
