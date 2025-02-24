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
end
