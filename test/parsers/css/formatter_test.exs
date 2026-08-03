# SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs/contributors>
#
# SPDX-License-Identifier: MIT

defmodule IgniterJSTest.Parsers.CSS.FormatterTest do
  use ExUnit.Case
  alias IgniterJs.Parsers.CSS.Formatter

  test "The CSS considered is formatted :: is_formatted" do
    {:ok, _, formatted} = assert Formatter.format("body { color: red; }")

    {:ok, _, true} = assert Formatter.is_formatted(formatted)
    {:error, _, false} = assert Formatter.is_formatted("body { color: red; }")
  end

  test "Format The CSS considered:: format" do
    {:ok, _, formatted} = assert Formatter.format("body { color: red; }")
    ^formatted = assert "body {\n  color: red;\n}\n"
  end

  describe "invalid input" do
    @invalid_css "body { color"

    test "format/2 reports a syntax error" do
      assert {:error, :format, "Parsing failed due to syntax errors."} =
               Formatter.format(@invalid_css, :content)
    end

    test "is_formatted/2 reports false without raising" do
      assert {:error, :is_formatted, false} = Formatter.is_formatted(@invalid_css, :content)
    end

    test "is_formatted?/2 returns false" do
      refute Formatter.is_formatted?(@invalid_css, :content)
    end
  end

  describe ":path mode" do
    setup do
      path =
        Path.join(System.tmp_dir!(), "igniter_js_#{System.unique_integer([:positive])}.css")

      File.write!(path, "body{color:red;}")
      on_exit(fn -> File.rm(path) end)
      {:ok, path: path}
    end

    test "rejects a .css file because the extension allow-list is js/ts only", %{path: path} do
      assert {:error, :format, "Invalid file path or format."} = Formatter.format(path, :path)
    end

    test "errors for a missing file" do
      assert {:error, :format, "Invalid file path or format."} =
               Formatter.format("missing.css", :path)
    end
  end

  describe "formatting properties" do
    test "format/2 is idempotent" do
      assert {:ok, :format, once} = Formatter.format("body{color:red;}", :content)
      assert {:ok, :format, twice} = Formatter.format(once, :content)
      assert once == twice
    end

    test "uses two-space indentation" do
      assert {:ok, :format, output} = Formatter.format("body{color:red;}", :content)
      assert output == "body {\n  color: red;\n}\n"
    end

    test "its own output is considered formatted" do
      assert {:ok, :format, output} = Formatter.format("body{color:red;}", :content)
      assert Formatter.is_formatted?(output, :content)
    end

    test "preserves comments" do
      assert {:ok, :format, output} =
               Formatter.format("/* keep me */\nbody{color:red;}", :content)

      assert output =~ "/* keep me */"
    end

    test "empty content formats to empty output" do
      assert {:ok, :format, ""} = Formatter.format("", :content)
    end
  end
end
