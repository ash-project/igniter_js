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
end
