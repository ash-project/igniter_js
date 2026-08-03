# SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs/contributors>
#
# SPDX-License-Identifier: MIT

defmodule IgniterJSTest.Parsers.Javascript.ParserEdgeCasesTest do
  use ExUnit.Case
  alias IgniterJs.Parsers.Javascript.Parser

  @valid_app_js "test/assets/validApp.js"
  @missing_path "test/assets/thisFileDoesNotExist.js"
  @unsupported_path "mix.exs"

  describe ":path mode with an unreadable file" do
    defp path_cases do
      [
        {:module_imported, fn path -> Parser.module_imported(path, "phoenix", :path) end},
        {:insert_imports,
         fn path -> Parser.insert_imports(path, ~s|import a from "b";|, :path) end},
        {:remove_imports, fn path -> Parser.remove_imports(path, "phoenix", :path) end},
        {:exist_live_socket, fn path -> Parser.exist_live_socket(path, :path) end},
        {:exist_var, fn path -> Parser.exist_var(path, "Hooks", :path) end},
        {:extend_hook_object, fn path -> Parser.extend_hook_object(path, "Hook", :path) end},
        {:remove_objects_from_hooks,
         fn path -> Parser.remove_objects_from_hooks(path, "Hook", :path) end},
        {:statistics, fn path -> Parser.statistics(path, :path) end},
        {:extend_var_object_by_object_names,
         fn path -> Parser.extend_var_object_by_object_names(path, "C", "H", :path) end},
        {:ast_to_estree, fn path -> Parser.ast_to_estree(path, :path) end},
        {:insert_at_index, fn path -> Parser.insert_at_index(path, "let a = 1;", 0, :path) end},
        {:replace_at_index, fn path -> Parser.replace_at_index(path, "let a = 1;", 0, :path) end}
      ]
    end

    test "every entry point returns an error tuple for a missing file" do
      for {fn_atom, call} <- path_cases() do
        assert {:error, ^fn_atom, "Invalid file path or format."} = call.(@missing_path),
               "#{fn_atom} did not return the expected error tuple for a missing file"
      end
    end

    test "every entry point returns an error tuple for an unsupported extension" do
      for {fn_atom, call} <- path_cases() do
        assert {:error, ^fn_atom, "Invalid file path or format."} = call.(@unsupported_path),
               "#{fn_atom} did not return the expected error tuple for an unsupported extension"
      end
    end

    test "statistics/2 returns an error tuple instead of raising MatchError" do
      assert {:error, :statistics, "Invalid file path or format."} =
               Parser.statistics(@missing_path, :path)
    end

    test "boolean helpers report false rather than raising for a missing file" do
      refute Parser.module_imported?(@missing_path, "phoenix", :path)
      refute Parser.exist_live_socket?(@missing_path, :path)
      refute Parser.var_exists?(@missing_path, "Hooks", :path)
    end
  end

  describe "var_exists?/3" do
    test "finds a let-declared variable in content" do
      assert Parser.var_exists?("let Hooks = {};", "Hooks")
      refute Parser.var_exists?("let Hooks = {};", "Missing")
    end

    test "works against a file path" do
      assert Parser.var_exists?(@valid_app_js, "Hooks", :path)
      refute Parser.var_exists?(@valid_app_js, "NotThere", :path)
    end

    test "agrees with the tuple-returning exist_var/3" do
      assert {:ok, :exist_var, true} = Parser.exist_var("let Hooks = {};", "Hooks")
      assert {:error, :exist_var, false} = Parser.exist_var("let Hooks = {};", "Missing")
    end
  end

  describe "empty and trivial input" do
    test "statistics/2 reports all zeros for empty content" do
      assert {:ok, :statistics, stats} = Parser.statistics("")

      assert %{functions: 0, classes: 0, debuggers: 0, imports: 0, trys: 0, throws: 0} = stats
    end

    test "statistics/2 returns a plain map without the struct key" do
      assert {:ok, :statistics, stats} = Parser.statistics("function a() {}")
      refute Map.has_key?(stats, :__struct__)
      assert stats.functions == 1
    end

    test "lookups on empty content report not-found rather than crashing" do
      assert {:error, :exist_live_socket, false} = Parser.exist_live_socket("")
      assert {:error, :module_imported, false} = Parser.module_imported("", "phoenix")
      assert {:error, :exist_var, false} = Parser.exist_var("", "Hooks")
    end
  end

  describe "index boundaries" do
    @code "let a = 1;\nlet b = 2;\n"

    test "insert_at_index/4 inserts at the very beginning" do
      assert {:ok, :insert_at_index, output} = Parser.insert_at_index(@code, "let z = 0;", 0)
      assert String.starts_with?(output, "let z = 0;")
    end

    test "insert_at_index/4 rejects an out-of-bounds index" do
      assert {:error, :insert_at_index, "Index out of bounds"} =
               Parser.insert_at_index(@code, "let z = 0;", 99)
    end

    test "replace_at_index/4 replaces the targeted statement" do
      assert {:ok, :replace_at_index, output} = Parser.replace_at_index(@code, "let z = 0;", 0)
      assert output =~ "let z = 0;"
      refute output =~ "let a = 1;"
      assert output =~ "let b = 2;"
    end

    test "replace_at_index/4 rejects an out-of-bounds index" do
      assert {:error, :replace_at_index, "Index out of bounds"} =
               Parser.replace_at_index(@code, "let z = 0;", 99)
    end
  end

  describe "extend_var_object_by_object_names/4" do
    test "errors when the target variable is absent" do
      assert {:error, :extend_var_object_by_object_names, reason} =
               Parser.extend_var_object_by_object_names("let x = 1;", "Missing", "Hook")

      assert is_binary(reason)
    end

    test "de-duplicates repeated object names" do
      code = "const Components = {};\n"

      assert {:ok, :extend_var_object_by_object_names, output} =
               Parser.extend_var_object_by_object_names(code, "Components", [
                 "A",
                 "B",
                 "A",
                 "B"
               ])

      assert length(Regex.scan(~r/\bA\b/, output)) == 1
      assert length(Regex.scan(~r/\bB\b/, output)) == 1
    end
  end

  describe "ast_to_estree/2" do
    test "captures both line and block comments with positions" do
      assert {:ok, :ast_to_estree, ast} =
               Parser.ast_to_estree("const a = 1; /* block */ // line", :content)

      assert Enum.map(ast["comments"], & &1["type"]) == ["Block", "Line"]

      for comment <- ast["comments"] do
        assert is_integer(comment["start"])
        assert is_integer(comment["end"])
        assert comment["end"] > comment["start"]
      end
    end

    test "emits range alongside start/end on every node" do
      assert {:ok, :ast_to_estree, ast} = Parser.ast_to_estree("const a = 1;", :content)

      program = ast["program"]
      assert [start_pos, end_pos] = program["range"]
      assert start_pos == program["start"]
      assert end_pos == program["end"]

      node = hd(program["body"])
      assert node["type"] == "VariableDeclaration"
      assert [_, _] = node["range"]
    end

    test "reports syntax errors in the errors list instead of failing" do
      assert {:ok, :ast_to_estree, ast} = Parser.ast_to_estree("const a = ;", :content)

      assert [error | _] = ast["errors"]
      assert error["severity"] == "Error"
      assert is_binary(error["message"])
    end

    test "returns an empty errors list for valid source" do
      assert {:ok, :ast_to_estree, ast} = Parser.ast_to_estree("const a = 1;", :content)
      assert ast["errors"] == []
    end

    test "parses a real file through :path" do
      assert {:ok, :ast_to_estree, ast} = Parser.ast_to_estree(@valid_app_js, :path)
      assert ast["program"]["type"] == "Program"
      assert ast["errors"] == []
    end
  end

  describe "import handling" do
    test "insert_imports/3 does not duplicate an import that already exists" do
      code = ~s|import { Socket } from "phoenix";\n|

      assert {:ok, :insert_imports, output} =
               Parser.insert_imports(code, ~s|import { Socket } from "phoenix";|)

      assert length(Regex.scan(~r/from "phoenix"/, output)) == 1
    end

    test "insert_imports/3 takes several imports as one newline-separated string" do
      code = "let a = 1;\n"

      assert {:ok, :insert_imports, output} =
               Parser.insert_imports(
                 code,
                 ~s|import a from "mod-a";\nimport b from "mod-b";|
               )

      assert output =~ "mod-a"
      assert output =~ "mod-b"
    end

    test "insert_imports/3 rejects a list, which the NIF cannot decode" do
      assert_raise ArgumentError, fn ->
        Parser.insert_imports("let a = 1;\n", [~s|import a from "mod-a";|])
      end
    end

    test "remove_imports/3 removes a matching full import statement" do
      code = ~s|import a from "mod-a";\nimport b from "mod-b";\nlet x = 1;\n|

      assert {:ok, :remove_imports, output} =
               Parser.remove_imports(code, ~s|import a from "mod-a";|)

      refute output =~ "mod-a"
      assert output =~ "mod-b"
      assert output =~ "let x = 1;"
    end

    test "remove_imports/3 removes several imports given as one string" do
      code = ~s|import a from "mod-a";\nimport b from "mod-b";\nlet x = 1;\n|

      assert {:ok, :remove_imports, output} =
               Parser.remove_imports(
                 code,
                 ~s|import a from "mod-a";\nimport b from "mod-b";|
               )

      refute output =~ "mod-a"
      refute output =~ "mod-b"
      assert output =~ "let x = 1;"
    end

    test "remove_imports/3 reports :ok but changes nothing for a bare module name" do
      code = ~s|import a from "mod-a";\nimport b from "mod-b";\n|

      assert {:ok, :remove_imports, output} = Parser.remove_imports(code, "mod-a")

      assert output =~ "mod-a"
      assert output =~ "mod-b"
    end
  end

  describe "list arguments" do
    test "extend_hook_object/3 accepts a list and de-duplicates" do
      code = ~s|let liveSocket = new LiveSocket("/live", Socket, { hooks: {} });\n|

      assert {:ok, :extend_hook_object, output} =
               Parser.extend_hook_object(code, ["A", "B", "A"])

      assert length(Regex.scan(~r/\bA\b/, output)) == 1
      assert length(Regex.scan(~r/\bB\b/, output)) == 1
    end

    test "remove_objects_from_hooks/3 accepts a list" do
      code = ~s|let liveSocket = new LiveSocket("/live", Socket, { hooks: { A, B, C } });\n|

      assert {:ok, :remove_objects_from_hooks, output} =
               Parser.remove_objects_from_hooks(code, ["A", "C"])

      refute output =~ "A"
      assert output =~ "B"
      refute output =~ "C"
    end
  end
end
