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
end
