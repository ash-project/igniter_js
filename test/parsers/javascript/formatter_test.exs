# SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs/contributors>
#
# SPDX-License-Identifier: MIT

defmodule IgniterJSTest.Parsers.Javascript.FormatterTest do
  use ExUnit.Case
  alias IgniterJs.Parsers.Javascript.Formatter

  test "The CSS considered is formatted :: is_formatted" do
    js_code_unformatted = "function test(){console.log('hello world');}"

    js_code_formatted = """
    function test() {
        console.log("hello world");
    }
    """

    {:ok, _, formatted} = assert Formatter.format(js_code_formatted)
    {:ok, _, true} = assert Formatter.is_formatted(formatted)
    {:error, _, false} = assert Formatter.is_formatted(js_code_unformatted)
  end

  test "Format The JS considered:: format" do
    js_code_formatted = """
    function test() {
    // expose liveSocket on window for web console debug logs and latency simulation:
                console.log("hello world");
                // expose liveSocket on window for web console debug logs and latency simulation:
    }
    """

    {:ok, _, formatted} = assert Formatter.format(js_code_formatted)

    ^formatted =
      assert "function test() {\n  // expose liveSocket on window for web console debug logs and latency simulation:\n  console.log(\"hello world\");\n  // expose liveSocket on window for web console debug logs and latency simulation:\n}\n"
  end

  describe "invalid input" do
    @invalid_js "function {{{"

    test "format/2 reports a syntax error" do
      assert {:error, :format, "Parsing failed due to syntax errors."} =
               Formatter.format(@invalid_js, :content)
    end

    test "is_formatted/2 reports false without raising" do
      assert {:error, :is_formatted, false} = Formatter.is_formatted(@invalid_js, :content)
    end

    test "is_formatted?/2 returns false" do
      refute Formatter.is_formatted?(@invalid_js, :content)
    end
  end

  describe ":path mode" do
    @valid_app_js "test/assets/validApp.js"

    test "format/2 formats a file from disk" do
      assert {:ok, :format, output} = Formatter.format(@valid_app_js, :path)
      assert output =~ "liveSocket"
    end

    test "format/2 errors for a missing file" do
      assert {:error, :format, "Invalid file path or format."} =
               Formatter.format("test/assets/missing.js", :path)
    end

    test "is_formatted?/2 errors safely for a missing file" do
      refute Formatter.is_formatted?("test/assets/missing.js", :path)
    end
  end

  describe "formatting properties" do
    test "format/2 is idempotent" do
      assert {:ok, :format, once} = Formatter.format("const   a=1;const b   =2;", :content)
      assert {:ok, :format, twice} = Formatter.format(once, :content)
      assert once == twice
    end

    test "its own output is considered formatted" do
      assert {:ok, :format, output} = Formatter.format("const   a=1;", :content)
      assert Formatter.is_formatted?(output, :content)
    end

    test "uses two-space indentation" do
      assert {:ok, :format, output} = Formatter.format("function a(){return 1;}", :content)
      assert output =~ "\n  return 1;"
    end

    test "empty content formats to empty output" do
      assert {:ok, :format, ""} = Formatter.format("", :content)
    end
  end
end
