defmodule IgniterJSTest.Parsers.Javascript.ParserTest do
  use ExUnit.Case
  alias IgniterJs.Parsers.Javascript.Parser

  @valid_app_js "test/assets/validApp.js"
  @invalid_app_without_live_socket "test/assets/invalidAppWithoutLiveSocket.js"
  @invalid_app_with_removed_import "test/assets/invalidAppWithRemovedImport.js"
  @invalid_app_without_live_socket_object "test/assets/invalidAppWithoutLiveSockerObject.js"
  @invalid_app_without_hooks_key "test/assets/invalidAppWithoutHooksKey.js"
  @valid_app_with_hooks_objects "test/assets/validAppWithSomeHooksObjects.js"
  @invalid_error_import "test/assets/errorImport.js"
  @valid_ast_statistics "test/assets/validASTStatistics.js"
  @valid_extend_var_object "test/assets/extendVarObject.js"

  test "User requested module imported? :: module_imported" do
    {:ok, :module_imported, true} =
      assert Parser.module_imported(
               @valid_app_js,
               "import { LiveSocket } from \"phoenix_live_view\";",
               :path
             )

    {:error, :module_imported, false} =
      assert Parser.module_imported(@invalid_app_without_live_socket, "none_live_view", :path)

    assert Parser.module_imported?(
             @valid_app_js,
             "import { LiveSocket } from \"phoenix_live_view\";",
             :path
           )

    assert !Parser.module_imported?(@invalid_app_without_live_socket, "none_live_view", :path)

    {:ok, :module_imported, true} =
      assert Parser.module_imported(
               File.read!(@valid_app_js),
               "import { LiveSocket } from \"phoenix_live_view\";"
             )

    {:error, :module_imported, false} =
      assert Parser.module_imported(
               File.read!(@invalid_app_without_live_socket),
               "none_live_view"
             )

    assert Parser.module_imported?(
             File.read!(@valid_app_js),
             "import { LiveSocket } from \"phoenix_live_view\";"
           )

    assert !Parser.module_imported?(
             File.read!(@invalid_app_without_live_socket),
             "none_live_view"
           )

    code = """
    import "phoenix_html";
    import { Socket, SocketV1 } from "phoenix";
    import { TS } from "tsobject";

    // This is first test we need to have
    console.log("We are here");

    const min = ()          => {return "Shahryar" + "Tavakkoli"};
    """

    imports = """
    import "phoenix_html";
    import { Socket, SocketV1 } from "phoenix";
    import { TS } from "tsobject";
    """

    assert Parser.module_imported?(code, imports)

    imports = """
    import "phoenix_html";
    import { Socket, SocketV1 } from "phoenix";
    import { TS1 } from "tsobject1";
    import { TS } from "tsobject";
    """

    assert !Parser.module_imported?(code, imports)

    assert !Parser.module_imported?("", "")
  end

  test "Insert some js lines for import modules :: insert_imports" do
    imports = """
    import { foo } from "module-name";
    import bar from "another-module";
    """

    considerd_output =
      "import { foo } from \"module-name\";\nimport bar from \"another-module\";\nlet Hooks = {};\n"

    {:ok, :insert_imports, js_output} =
      assert Parser.insert_imports(@invalid_app_without_live_socket, imports, :path)

    ^js_output = assert considerd_output

    {:ok, :insert_imports, js_output} =
      assert Parser.insert_imports(File.read!(@invalid_app_without_live_socket), imports)

    ^js_output = assert considerd_output
  end

  test "Remove imported modules :: remove_imports" do
    none_imported_module_output =
      "import { foo } from \"module-name\";\nimport bar from \"another-module\";\nlet Hooks = {};\n"

    {:ok, :remove_imports, outptu} =
      Parser.remove_imports(@invalid_app_with_removed_import, "phoenix_live_view", :path)

    ^none_imported_module_output = assert outptu

    remove_a_module_output = "import { foo } from \"module-name\";\nlet Hooks = {};\n"

    {:ok, :remove_imports, outptu} =
      Parser.remove_imports(
        @invalid_app_with_removed_import,
        "import bar from \"another-module\"",
        :path
      )

    ^remove_a_module_output = assert outptu

    remove_two_modules_output = "import { foo } from \"module-name\";\nlet Hooks = {};\n"

    {:ok, :remove_imports, outptu} =
      Parser.remove_imports(
        @invalid_app_with_removed_import,
        "import bar from \"another-module\";",
        :path
      )

    ^remove_two_modules_output = assert outptu

    none_imported_module_output =
      "import { foo } from \"module-name\";\nimport bar from \"another-module\";\nlet Hooks = {};\n"

    {:ok, :remove_imports, outptu} =
      Parser.remove_imports(File.read!(@invalid_app_with_removed_import), "phoenix_live_view")

    ^none_imported_module_output = assert outptu

    remove_a_module_output =
      "import { foo } from \"module-name\";\nimport bar from \"another-module\";\nlet Hooks = {};\n"

    {:ok, :remove_imports, outptu} =
      Parser.remove_imports(File.read!(@invalid_app_with_removed_import), "module-name")

    ^remove_a_module_output = assert outptu

    remove_two_modules_output = "let Hooks = {};\n"

    {:ok, :remove_imports, outptu} =
      Parser.remove_imports(
        File.read!(@invalid_app_with_removed_import),
        """
        import { foo } from "module-name";
        import bar from "another-module";
        """
      )

    ^remove_two_modules_output = assert outptu
  end

  test "LiveSocket var exist :: exist_live_socket" do
    {:ok, :exist_live_socket, true} =
      assert Parser.exist_live_socket(@valid_app_js, :path)

    {:error, :exist_live_socket, false} =
      assert Parser.exist_live_socket(@invalid_app_without_live_socket, :path)

    assert Parser.exist_live_socket?(@valid_app_js, :path)

    assert !Parser.exist_live_socket?(@invalid_app_without_live_socket, :path)

    {:ok, :exist_live_socket, true} =
      assert Parser.exist_live_socket(File.read!(@valid_app_js))

    {:error, :exist_live_socket, false} =
      assert Parser.exist_live_socket(File.read!(@invalid_app_without_live_socket))

    assert Parser.exist_live_socket?(File.read!(@valid_app_js))

    assert !Parser.exist_live_socket?(File.read!(@invalid_app_without_live_socket))
  end

  test "Extend hook objects :: extend_hook_object" do
    {:error, :extend_hook_object, _msg} =
      Parser.extend_hook_object(@invalid_app_without_live_socket, "something", :path)

    {:error, :extend_hook_object, _msg} =
      Parser.extend_hook_object(@invalid_app_without_live_socket_object, "something", :path)

    considerd_output =
      "let Hooks = {};\nlet liveSocket = new LiveSocket(\"/live\", Socket, {\n    longPollFallbackMs: 2500,\n    params: {\n        _csrf_token: csrfToken\n    },\n    hooks: {\n        something\n    }\n});\n"

    {:ok, :extend_hook_object, output} =
      assert Parser.extend_hook_object(@invalid_app_without_hooks_key, "something", :path)

    ^considerd_output = assert output

    {:ok, :extend_hook_object, output} =
      assert Parser.extend_hook_object(
               @invalid_app_without_hooks_key,
               [
                 "something",
                 "another"
               ],
               :path
             )

    considerd_output =
      "let Hooks = {};\nlet liveSocket = new LiveSocket(\"/live\", Socket, {\n    longPollFallbackMs: 2500,\n    params: {\n        _csrf_token: csrfToken\n    },\n    hooks: {\n        another,\n        something\n    }\n});\n"

    ^considerd_output = assert output

    {:error, :extend_hook_object, _msg} =
      Parser.extend_hook_object(File.read!(@invalid_app_without_live_socket), "something")

    {:error, :extend_hook_object, _msg} =
      Parser.extend_hook_object(File.read!(@invalid_app_without_live_socket_object), "something")

    considerd_output =
      "let Hooks = {};\nlet liveSocket = new LiveSocket(\"/live\", Socket, {\n    longPollFallbackMs: 2500,\n    params: {\n        _csrf_token: csrfToken\n    },\n    hooks: {\n        something\n    }\n});\n"

    {:ok, :extend_hook_object, output} =
      assert Parser.extend_hook_object(File.read!(@invalid_app_without_hooks_key), "something")

    ^considerd_output = assert output

    {:ok, :extend_hook_object, output} =
      assert Parser.extend_hook_object(
               File.read!(@invalid_app_without_hooks_key),
               ["something", "another"]
             )

    considerd_output =
      "let Hooks = {};\nlet liveSocket = new LiveSocket(\"/live\", Socket, {\n    longPollFallbackMs: 2500,\n    params: {\n        _csrf_token: csrfToken\n    },\n    hooks: {\n        another,\n        something\n    }\n});\n"

    ^considerd_output = assert output

    js_input_code =
      """
      let Hooks = {};
      let liveSocket = new LiveSocket("/live", Socket, {
        longPollFallbackMs: 2500,
        params: {
          _csrf_token: csrfToken,
        },
        hooks: {
          something,
          ...MishkaComponent
        },
      });
      """

    {:ok, :extend_hook_object, output} =
      assert Parser.extend_hook_object(js_input_code, ["...MishkaComponent", "something"])

    considerd_output =
      "let Hooks = {};\nlet liveSocket = new LiveSocket(\"/live\", Socket, {\n    longPollFallbackMs: 2500,\n    params: {\n        _csrf_token: csrfToken\n    },\n    hooks: {\n        something,\n        ...MishkaComponent\n    }\n});\n"

    ^considerd_output = assert output

    1 = assert string_counter(considerd_output, "\\.\\.\\.MishkaComponent")
  end

  test "Remove objects of hooks key inside LiveSocket:: remove_objects_from_hooks" do
    considerd_output =
      "let liveSocket = new LiveSocket(\"/live\", Socket, {\n    hooks: {\n        ...Hooks,\n        CopyMixInstallationHook,\n        OXCExampleObjectHook\n    },\n    longPollFallbackMs: 2500,\n    params: {\n        _csrf_token: csrfToken\n    }\n});\n"

    {:ok, :remove_objects_from_hooks, output} =
      assert Parser.remove_objects_from_hooks(
               @valid_app_with_hooks_objects,
               ["something", "another"],
               :path
             )

    ^considerd_output = assert output

    considerd_output =
      "let liveSocket = new LiveSocket(\"/live\", Socket, {\n    hooks: {\n        ...Hooks,\n        CopyMixInstallationHook\n    },\n    longPollFallbackMs: 2500,\n    params: {\n        _csrf_token: csrfToken\n    }\n});\n"

    {:ok, :remove_objects_from_hooks, output} =
      assert Parser.remove_objects_from_hooks(
               @valid_app_with_hooks_objects,
               "OXCExampleObjectHook",
               :path
             )

    ^considerd_output = assert output

    considerd_output =
      "let liveSocket = new LiveSocket(\"/live\", Socket, {\n    hooks: {\n        ...Hooks\n    },\n    longPollFallbackMs: 2500,\n    params: {\n        _csrf_token: csrfToken\n    }\n});\n"

    {:ok, :remove_objects_from_hooks, output} =
      assert Parser.remove_objects_from_hooks(
               @valid_app_with_hooks_objects,
               ["OXCExampleObjectHook", "CopyMixInstallationHook"],
               :path
             )

    ^considerd_output = assert output

    considerd_output =
      "let liveSocket = new LiveSocket(\"/live\", Socket, {\n    hooks: {\n        ...Hooks,\n        CopyMixInstallationHook,\n        OXCExampleObjectHook\n    },\n    longPollFallbackMs: 2500,\n    params: {\n        _csrf_token: csrfToken\n    }\n});\n"

    {:ok, :remove_objects_from_hooks, output} =
      assert Parser.remove_objects_from_hooks(
               File.read!(@valid_app_with_hooks_objects),
               ["something", "another"]
             )

    ^considerd_output = assert output

    considerd_output =
      "let liveSocket = new LiveSocket(\"/live\", Socket, {\n    hooks: {\n        ...Hooks,\n        CopyMixInstallationHook\n    },\n    longPollFallbackMs: 2500,\n    params: {\n        _csrf_token: csrfToken\n    }\n});\n"

    {:ok, :remove_objects_from_hooks, output} =
      assert Parser.remove_objects_from_hooks(
               File.read!(@valid_app_with_hooks_objects),
               "OXCExampleObjectHook"
             )

    ^considerd_output = assert output

    considerd_output =
      "let liveSocket = new LiveSocket(\"/live\", Socket, {\n    hooks: {\n        ...Hooks\n    },\n    longPollFallbackMs: 2500,\n    params: {\n        _csrf_token: csrfToken\n    }\n});\n"

    {:ok, :remove_objects_from_hooks, output} =
      assert Parser.remove_objects_from_hooks(
               File.read!(@valid_app_with_hooks_objects),
               ["OXCExampleObjectHook", "CopyMixInstallationHook"]
             )

    ^considerd_output = assert output
  end

  test "Get statistics from the given file or content :: statistics" do
    {:ok, :statistics, %{functions: 0, imports: 0, classes: 0, debuggers: 0, trys: 0, throws: 0}} =
      assert Parser.statistics(@invalid_error_import, :path)

    {:ok, :statistics, statistics} = assert Parser.statistics(@valid_ast_statistics, :path)
    1 = assert statistics.functions
    1 = assert statistics.classes
    2 = assert statistics.debuggers
    2 = assert statistics.imports
    0 = assert statistics.trys
    0 = assert statistics.throws
  end

  test "Extend some objects inside a var object :: extend_var_object_by_object_names" do
    objects_names = ["OXCTestHook", "MishkaHooks", "MishkaHooks", "OXCTestHook"]

    considerd_output =
      "const Components = {\n    MishkaHooks,\n    OXCTestHook\n};\nexport default Components;\n"

    # It prevents duplicate objects
    {:ok, :extend_var_object_by_object_names, output} =
      assert Parser.extend_var_object_by_object_names(
               @valid_extend_var_object,
               "Components",
               objects_names,
               :path
             )

    ^considerd_output = assert output

    {:error, :extend_var_object_by_object_names, _output} =
      assert Parser.extend_var_object_by_object_names("None", "Components", objects_names)

    {:ok, :extend_var_object_by_object_names, _output} =
      assert Parser.extend_var_object_by_object_names(
               @valid_extend_var_object,
               "Components",
               "TestHook",
               :path
             )

    code = """
    import ScrollArea from "./scrollArea.js";

    const Components = {};

    export default Components;
    """

    considerd_output =
      "import ScrollArea from \"./scrollArea.js\";\nconst Components = {\n    ...NoneComponent,\n    NoneComponent,\n    ScrollArea\n};\nexport default Components;\n"

    names = ["ScrollArea", "NoneComponent", "...NoneComponent", "NoneComponent", "ScrollArea"]

    {:ok, :extend_var_object_by_object_names, output} =
      assert Parser.extend_var_object_by_object_names(code, "Components", names)

    ^considerd_output = assert output

    1 = assert string_counter(considerd_output, "\\.\\.\\.NoneComponent")
    2 = assert string_counter(considerd_output, "ScrollArea")
    1 = assert string_counter(considerd_output, "(^|[^.])NoneComponent")
  end

  test "Check existing vars :: exist_var" do
    code = """
    import { foo } from "module-name";
    import bar from "another-module";
    """

    {:error, :exist_var, false} = assert Parser.exist_var(code, "test_name")
    assert !Parser.var_exists?(code, "test_name")

    code = """
    import { foo } from "module-name";
    import bar from "another-module";

    let mishka_ash = () => {1 + 1}

    let igniterJS = %{stack: ["rust", "elixir", "js"]}
    """

    assert Parser.var_exists?(code, "igniterJS")
    {:ok, :exist_var, true} = assert Parser.exist_var(code, "igniterJS")
  end

  defp string_counter(string, pattern) do
    Regex.scan(Regex.compile!(pattern), string)
    |> length()
  end
end
