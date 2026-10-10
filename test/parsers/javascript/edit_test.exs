# SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs/contributors>
#
# SPDX-License-Identifier: MIT

defmodule IgniterJSTest.Parsers.Javascript.EditTest do
  @moduledoc """
  The in-place edits, called the way a user of the library calls them.

  The Rust side has its own suite for every shape of source. What these pin down is the Elixir
  surface: that each function builds the right request, returns the library's usual tuple, accepts
  every target and option, reads the dialect from a path, and carries non-ASCII text across the
  NIF boundary unharmed.

  Text that is not ASCII is written with `\\u` escapes so this file stays ASCII; the constants
  below stand for a short word in Persian, a word with a zero-width non-joiner, and an emoji (two
  UTF-16 units, four UTF-8 bytes — the one that tells the two offset counts apart).
  """

  use ExUnit.Case, async: true

  alias IgniterJs.Parsers.Javascript.Edit

  @fa "سلام"
  @zwnj "می‌خواهم"
  @emoji "\u{1F9FE}"

  @main_tsx """
  import { StrictMode } from 'react'
  import { createRoot } from 'react-dom/client'
  import './index.css'
  import App from './App.tsx'

  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <App />
    </StrictMode>,
  )
  """

  describe "merge_object_members/4" do
    test "an inline object stays inline" do
      assert {:ok, :merge_object_members, "export const t = { en: {}, fa } as const"} =
               Edit.merge_object_members(
                 "export const t = { fa } as const",
                 {:variable, "t"},
                 [{"en", "en: {}"}],
                 dialect: :ts
               )
    end

    test "a comment above a member stays above it" do
      source = "export default {\n  // keep me\n  plugins: [],\n}\n"

      assert {:ok, _, out} =
               Edit.merge_object_members(source, :default_export, [{"base", ~s(base: "/")}],
                 dialect: :ts
               )

      assert out == "export default {\n  base: \"/\",\n  // keep me\n  plugins: [],\n}\n"
    end

    test "the levels of a path are created" do
      assert {:ok, _, out} =
               Edit.merge_object_members(
                 "export default {\n  plugins: [],\n}\n",
                 :default_export,
                 [{"@rsk/core", ~s("@rsk/core": "/abs/core.ts")}],
                 path: ["resolve", "alias"],
                 dialect: :ts
               )

      assert out ==
               "export default {\n  resolve: {\n    alias: {\n      \"@rsk/core\": \"/abs/core.ts\",\n    },\n  },\n  plugins: [],\n}\n"
    end

    test "a JSON document keeps its key order and indentation" do
      assert {:ok, :merge_object_members, out} =
               Edit.merge_object_members(
                 ~s({\n  "name": "app"\n}\n),
                 :document,
                 [{"type", ~s("type": "module")}],
                 dialect: :json
               )

      assert out == ~s({\n  "type": "module",\n  "name": "app"\n}\n)
    end

    test "a member that is there is left alone, and replaced with mode: :set" do
      source = ~s(const t = { a: "./x" }\n)
      member = [{"a", ~s(a: "/abs/x")}]

      assert {:ok, _, ^source} =
               Edit.merge_object_members(source, {:variable, "t"}, member, dialect: :ts)

      assert {:ok, _, ~s(const t = { a: "/abs/x" }\n)} =
               Edit.merge_object_members(source, {:variable, "t"}, member,
                 dialect: :ts,
                 mode: :set
               )
    end

    test "every kind of target" do
      source = """
      export const translations = { fa: {} } as const
      export const rsk = createRsk({ adapter: A })
      export default defineConfig(() => ({ plugins: [] }))
      """

      for {target, key} <- [
            {{:variable, "translations"}, "x"},
            {{:call_argument, "createRsk"}, "y"},
            {:default_export, "z"}
          ] do
        assert {:ok, _, out} =
                 Edit.merge_object_members(source, target, [{key, "#{key}: 1"}], dialect: :ts)

        assert out =~ "#{key}: 1"
      end
    end

    test "a later call argument" do
      assert {:ok, _, "const x = lib.make(1, { b: 2, a: 1 })"} =
               Edit.merge_object_members(
                 "const x = lib.make(1, { a: 1 })",
                 {:call_argument, "make", 1},
                 [{"b", "b: 2"}],
                 dialect: :ts
               )
    end

    test "running it again changes nothing" do
      args = [:default_export, [{"a", "a: 1"}], [path: ["resolve", "alias"], dialect: :ts]]
      {:ok, _, once} = apply(Edit, :merge_object_members, ["export default {}\n" | args])
      assert {:ok, _, ^once} = apply(Edit, :merge_object_members, [once | args])
    end

    test "a target that is not in the file is an error with a reason" do
      assert {:error, :merge_object_members, reason} =
               Edit.merge_object_members("const a = {}", {:variable, "nope"}, [{"x", "x: 1"}],
                 dialect: :ts
               )

      assert reason =~ "nope"
    end

    test "a source that does not parse is an error" do
      assert {:error, :merge_object_members, reason} =
               Edit.merge_object_members("const t = { a: ", {:variable, "t"}, [], dialect: :ts)

      assert reason =~ "could not parse"
    end

    test "an unknown dialect is an error" do
      assert {:error, :merge_object_members, _} =
               Edit.merge_object_members("a = 1", {:variable, "a"}, [], dialect: :coffee)
    end
  end

  describe "at: :end" do
    test "package.json gets workspaces after the other keys, as strict JSON" do
      source = ~s({\n  "name": "app",\n  "private": true\n}\n)

      assert {:ok, _, out} =
               Edit.merge_array_members(source, :document, ["vendor/@rsk/*"],
                 path: ["workspaces"],
                 at: :end,
                 dialect: :json
               )

      assert out ==
               ~s({\n  "name": "app",\n  "private": true,\n  "workspaces": [\n    "vendor/@rsk/*"\n  ]\n}\n)

      assert Jason.decode!(out)["workspaces"] == ["vendor/@rsk/*"]
    end

    test "members go after the last one and the developer's trailing comma stays" do
      assert {:ok, _, "export default {\n  a: 1,\n  b: 2,\n}\n"} =
               Edit.merge_object_members(
                 "export default {\n  a: 1,\n}\n",
                 :default_export,
                 [{"b", "b: 2"}],
                 at: :end,
                 dialect: :ts
               )
    end

    test "it is idempotent" do
      args = [:default_export, [{"b", "b: 2"}], [at: :end, dialect: :ts]]

      {:ok, _, once} =
        apply(Edit, :merge_object_members, ["export default {\n  a: 1\n}\n" | args])

      assert {:ok, _, ^once} = apply(Edit, :merge_object_members, [once | args])
    end
  end

  describe "merge_array_members/4" do
    test "adds to a JSON array and skips what is there" do
      source = ~s({\n  "workspaces": ["apps/*"]\n}\n)

      assert {:ok, :merge_array_members, out} =
               Edit.merge_array_members(source, :document, ["vendor/@rsk/*"],
                 path: ["workspaces"],
                 dialect: :json
               )

      assert out == ~s({\n  "workspaces": ["vendor/@rsk/*", "apps/*"]\n}\n)

      assert {:ok, _, ^out} =
               Edit.merge_array_members(out, :document, ["vendor/@rsk/*"],
                 path: ["workspaces"],
                 dialect: :json
               )
    end

    test "creates the array when it is missing" do
      assert {:ok, _, out} =
               Edit.merge_array_members(~s({\n  "name": "x"\n}\n), :document, ["a"],
                 path: ["workspaces"],
                 dialect: :json
               )

      assert out == ~s({\n  "workspaces": [\n    "a"\n  ],\n  "name": "x"\n}\n)
      # strict JSON: no trailing comma, so npm and bun accept it
      assert Jason.decode!(out) == %{"workspaces" => ["a"], "name" => "x"}
    end

    test "an empty package.json gets strict JSON, never a trailing comma" do
      for source <- ["{}", "{\n}\n", "{ }"] do
        assert {:ok, _, out} =
                 Edit.merge_array_members(source, :document, ["vendor/@rsk/*"],
                   path: ["workspaces"],
                   dialect: :json
                 )

        assert Jason.decode!(out) == %{"workspaces" => ["vendor/@rsk/*"]}
      end
    end

    test "dedupe in a Vite config" do
      source = "export default {\n  resolve: {\n    dedupe: [\"react\"],\n  },\n}\n"

      assert {:ok, _, out} =
               Edit.merge_array_members(source, :default_export, ["react", "zod"],
                 path: ["resolve", "dedupe"],
                 dialect: :ts
               )

      assert out =~ ~s(dedupe: ["zod", "react"])
    end
  end

  describe "JSX" do
    test "wrap the app of a fresh Vite project" do
      assert {:ok, :wrap_jsx, out} =
               Edit.wrap_jsx(@main_tsx, "App", "RskApp",
                 props: [setup: {:expr, "rsk"}, translations: {:expr, "translations"}],
                 dialect: :tsx
               )

      assert out =~
               "<StrictMode>\n    <RskApp setup={rsk} translations={translations}>\n      <App />\n    </RskApp>\n  </StrictMode>,"

      assert {:ok, :wrap_jsx, ^out} =
               Edit.wrap_jsx(out, "App", "RskApp",
                 props: [setup: {:expr, "rsk"}, translations: {:expr, "translations"}]
               )
    end

    test "the dialect defaults to tsx for JSX edits" do
      assert {:ok, _, "const a = <RskApp><App /></RskApp>"} =
               Edit.wrap_jsx("const a = <App />", "App", "RskApp")
    end

    test "the three prop forms" do
      assert {:ok, _, ~s(const a = <W flag title="x" n={1}><A /></W>)} =
               Edit.wrap_jsx("const a = <A />", "A", "W",
                 props: [flag: true, title: {:string, "x"}, n: {:expr, "1"}]
               )
    end

    test "a missing element is an error naming it" do
      assert {:error, :wrap_jsx, reason} = Edit.wrap_jsx("const a = <div />", "App", "RskApp")
      assert reason =~ "App"
    end

    test "unwrap keeps the children" do
      source = "const a = (\n  <BrowserRouter>\n    <App />\n  </BrowserRouter>\n)\n"

      assert {:ok, :unwrap_jsx, "const a = (\n  <App />\n)\n"} =
               Edit.unwrap_jsx(source, "BrowserRouter")
    end

    test "unwrapping what is not there changes nothing" do
      assert {:ok, _, "const a = <App />"} = Edit.unwrap_jsx("const a = <App />", "BrowserRouter")
    end

    test "add_jsx_props adds, skips and sets" do
      assert {:ok, _, "const a = <RskApp setup={rsk}><App /></RskApp>"} =
               Edit.add_jsx_props("const a = <RskApp><App /></RskApp>", "RskApp",
                 setup: {:expr, "rsk"}
               )

      mine = "const a = <RskApp setup={mine} />"

      assert {:ok, _, ^mine} =
               Edit.add_jsx_props(mine, "RskApp", [setup: {:expr, "rsk"}], mode: :add)

      assert {:ok, _, "const a = <RskApp setup={rsk} />"} =
               Edit.add_jsx_props(mine, "RskApp", [setup: {:expr, "rsk"}], mode: :set)
    end

    test "jsx_elements lists names in source order" do
      assert {:ok, :jsx_elements, ["A", "B", "C.D"]} =
               Edit.jsx_elements("const a = <A><B /><C.D /></A>")
    end
  end

  describe "imports" do
    test "ensure_imports goes into existing braces" do
      assert {:ok, :ensure_imports, ~s(import { createRsk, i18nPreset } from "@rsk/core"\n)} =
               Edit.ensure_imports(
                 ~s(import { createRsk } from "@rsk/core"\n),
                 "@rsk/core",
                 ["i18nPreset"],
                 dialect: :ts
               )
    end

    test "ensure_imports writes a new line in the file's own style" do
      assert {:ok, _, "import a from 'a'\nimport { n } from 'm'\n"} =
               Edit.ensure_imports("import a from 'a'\n", "m", ["n"], dialect: :ts)

      assert {:ok, _, "import a from \"a\";\nimport { n } from \"m\";\n"} =
               Edit.ensure_imports("import a from \"a\";\n", "m", ["n"], dialect: :ts)
    end

    test "ensure_imports is idempotent" do
      {:ok, _, once} = Edit.ensure_imports("const x = 1\n", "m", ["n"], dialect: :ts)
      assert {:ok, _, ^once} = Edit.ensure_imports(once, "m", ["n"], dialect: :ts)
    end

    test "move_imports moves names between modules" do
      assert {:ok, :move_imports,
              ~s(import { createRsk } from "@rsk/core"\nimport { bearerAuth } from "@rsk/http"\n)} =
               Edit.move_imports(
                 ~s(import { createRsk, bearerAuth } from "@rsk/core"\n),
                 "@rsk/core",
                 "@rsk/http",
                 ["bearerAuth"],
                 dialect: :ts
               )
    end

    test "imports/2 lists every import" do
      assert {:ok, :imports, [first, second]} =
               Edit.imports(~s(import a, { b, c as d } from "m"\nimport type { T } from "t"\n),
                 dialect: :ts
               )

      assert first == %{
               "module" => "m",
               "names" => ["b", "c"],
               "default" => true,
               "namespace" => false,
               "type_only" => false
             }

      assert second["type_only"] == true
    end
  end

  describe "object_members/3" do
    test "lists the names in source order" do
      assert {:ok, :object_members, ["fa", "en"]} =
               Edit.object_members(
                 "export const t = { fa: {}, en: {} } as const",
                 {:variable, "t"},
                 dialect: :ts
               )
    end

    test "at a path, and for JSON" do
      assert {:ok, _, ["a", "b-c"]} =
               Edit.object_members(
                 "export default { resolve: { alias: { a: 1, \"b-c\": 2 } } }",
                 :default_export,
                 path: ["resolve", "alias"],
                 dialect: :ts
               )

      assert {:ok, _, ["name", "private"]} =
               Edit.object_members(~s({"name": "x", "private": true}), :document, dialect: :json)
    end

    test "a path that is not there is an error, not an empty list" do
      assert {:error, :object_members, _} =
               Edit.object_members("export default { a: 1 }", :default_export,
                 path: ["resolve"],
                 dialect: :ts
               )
    end

    test "arrays are asked with container: :array" do
      assert {:ok, _, ["react", "zod"]} =
               Edit.object_members(~s(const a = ["react", 'zod']), {:variable, "a"},
                 container: :array,
                 dialect: :ts
               )
    end
  end

  describe "member_values/3" do
    test "values as written, shorthand included" do
      assert {:ok, :member_values, [{"@", ~s|"./src"|}, {"a", "fileURLToPath(x)"}, {"fa", "fa"}]} =
               Edit.member_values(
                 ~s|export default { resolve: { alias: { "@": "./src", a: fileURLToPath(x), fa } } }|,
                 :default_export,
                 path: ["resolve", "alias"],
                 dialect: :ts
               )
    end

    test "JSON values keep their quotes and structure" do
      assert {:ok, _, [{"name", ~s|"x"|}, {"list", "[1, 2]"}]} =
               Edit.member_values(~s({"name": "x", "list": [1, 2]}), :document, dialect: :json)
    end

    test "a path that is not there is an error" do
      assert {:error, :member_values, _} =
               Edit.member_values("export default { a: 1 }", :default_export,
                 path: ["resolve"],
                 dialect: :ts
               )
    end
  end

  describe "text that is not ASCII crosses the NIF boundary unharmed" do
    test "right-to-left text, a zero-width non-joiner and an emoji before the edit" do
      for before <- [@fa, @zwnj, @emoji <> @emoji, "#{@fa} #{@emoji} #{@zwnj}"] do
        source = "// #{before}\nexport const t = { fa }\nconst fa = {}\n"

        assert {:ok, _, out} =
                 Edit.merge_object_members(source, {:variable, "t"}, [{"en", "en: {}"}],
                   dialect: :ts
                 )

        assert out == "// #{before}\nexport const t = { en: {}, fa }\nconst fa = {}\n"
      end
    end

    test "text inside and after the object is untouched" do
      source = "const t = {\n  a: \"#{@fa}\", // #{@zwnj}\n  b: \"#{@emoji}\",\n}\n// #{@fa}\n"

      assert {:ok, _, out} =
               Edit.merge_object_members(source, {:variable, "t"}, [{"c", "c: 3"}], dialect: :ts)

      assert out ==
               "const t = {\n  c: 3,\n  a: \"#{@fa}\", // #{@zwnj}\n  b: \"#{@emoji}\",\n}\n// #{@fa}\n"
    end

    test "member names that are not ASCII are matched" do
      source = "const t = { \"#{@fa}\": 1 }\n"

      assert {:ok, _, ^source} =
               Edit.merge_object_members(source, {:variable, "t"}, [{@fa, "x: 1"}], dialect: :ts)

      assert {:ok, _, [@fa]} = Edit.object_members(source, {:variable, "t"}, dialect: :ts)
    end

    test "JSON and JSX" do
      json = ~s({\n  "title": "#{@fa}"\n}\n)

      assert {:ok, _, out} =
               Edit.merge_object_members(json, :document, [{"a", ~s("a": 1)}], dialect: :json)

      assert out == ~s({\n  "a": 1,\n  "title": "#{@fa}"\n}\n)

      jsx = "// #{@fa} #{@emoji}\nconst a = <App />\n"
      assert {:ok, _, out} = Edit.wrap_jsx(jsx, "App", "RskApp")
      assert out == "// #{@fa} #{@emoji}\nconst a = <RskApp><App /></RskApp>\n"
    end

    test "imports" do
      source = "// #{@fa}\nimport { a } from \"m\"\nconst x = \"#{@emoji}\"\n"

      assert {:ok, _, "// #{@fa}\nimport { a, b } from \"m\"\nconst x = \"#{@emoji}\"\n"} =
               Edit.ensure_imports(source, "m", ["b"], dialect: :ts)
    end
  end

  describe "files" do
    @describetag :tmp_dir

    test "the dialect comes from the extension", %{tmp_dir: dir} do
      main = Path.join(dir, "main.tsx")
      File.write!(main, "const a = <App />\n")

      assert {:ok, :wrap_jsx, "const a = <RskApp><App /></RskApp>\n"} =
               Edit.wrap_jsx(main, "App", "RskApp", type: :path)

      package = Path.join(dir, "package.json")
      File.write!(package, ~s({\n  "name": "x"\n}\n))

      assert {:ok, _, out} =
               Edit.merge_object_members(package, :document, [{"type", ~s("type": "module")}],
                 type: :path
               )

      assert out == ~s({\n  "type": "module",\n  "name": "x"\n}\n)
    end

    test "a file that does not exist is an error", %{tmp_dir: dir} do
      assert {:error, :wrap_jsx, _} =
               Edit.wrap_jsx(Path.join(dir, "nope.tsx"), "App", "RskApp", type: :path)
    end
  end
end
