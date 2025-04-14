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
end
