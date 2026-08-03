# SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs/contributors>
#
# SPDX-License-Identifier: MIT

defmodule IgniterJSTest.HelpersTest do
  use ExUnit.Case
  require IgniterJs.Helpers
  alias IgniterJs.Helpers

  @valid_app_js "test/assets/validApp.js"

  setup do
    tmp = Path.join(System.tmp_dir!(), "igniter_js_helpers_#{System.unique_integer([:positive])}")
    File.mkdir_p!(tmp)
    on_exit(fn -> File.rm_rf(tmp) end)
    {:ok, tmp: tmp}
  end

  describe "read_and_validate_file/1" do
    test "reads an existing .js file" do
      assert {:ok, content} = Helpers.read_and_validate_file(@valid_app_js)
      assert content =~ "liveSocket"
    end

    test "accepts the .ts extension", %{tmp: tmp} do
      path = Path.join(tmp, "sample.ts")
      File.write!(path, "const answer: number = 42;")

      assert {:ok, "const answer: number = 42;"} = Helpers.read_and_validate_file(path)
    end

    test "rejects a file that does not exist" do
      assert {:error, "Invalid file path or format."} =
               Helpers.read_and_validate_file("definitely/missing.js")
    end

    test "rejects an existing file with an unsupported extension" do
      assert {:error, "Invalid file path or format."} = Helpers.read_and_validate_file("mix.exs")
    end

    test "rejects a directory" do
      assert {:error, "Invalid file path or format."} = Helpers.read_and_validate_file("lib")
    end

    test "surfaces the posix reason when the path is a directory named like a file", %{tmp: tmp} do
      path = Path.join(tmp, "iAmADirectory.js")
      File.mkdir_p!(path)

      assert {:error, :eisdir} = Helpers.read_and_validate_file(path)
    end
  end

  describe "call_nif_fn/4" do
    test "defaults to :content and passes the content straight through" do
      result =
        Helpers.call_nif_fn("some content", {:my_caller, 2}, fn content ->
          {:ok, :ignored_atom, String.upcase(content)}
        end)

      assert {:ok, :my_caller, "SOME CONTENT"} = result
    end

    test ":path reads the file before invoking the processing function" do
      result =
        Helpers.call_nif_fn(
          @valid_app_js,
          {:my_caller, 2},
          fn content -> {:ok, :ignored_atom, String.length(content)} end,
          :path
        )

      assert {:ok, :my_caller, length} = result
      assert length > 0
    end

    test ":path returns an error tuple without calling the processing function" do
      result =
        Helpers.call_nif_fn(
          "missing.js",
          {:my_caller, 2},
          fn _content -> raise "processing function must not run" end,
          :path
        )

      assert {:error, :my_caller, "Invalid file path or format."} = result
    end
  end

  describe "normalize_output/2" do
    test "replaces the middle element with the caller function name" do
      assert {:ok, :caller_name, "payload"} =
               Helpers.normalize_output({:ok, :from_nif, "payload"}, {:caller_name, 1})

      assert {:error, :caller_name, "reason"} =
               Helpers.normalize_output({:error, :from_nif, "reason"}, {:caller_name, 1})
    end
  end
end
