# SPDX-FileCopyrightText: 2024 Shahryar Tavakkoli
# SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs/contributors>
#
# SPDX-License-Identifier: MIT

defmodule IgniterJs.Parsers.Javascript.Edit do
  @moduledoc """
  Edits that change a file **in place**: only the bytes the edit is about change, and every
  comment, blank line and indent the developer wrote stays as it was.

  The codemods in `IgniterJs.Parsers.Javascript.Parser` parse, change the tree and print it back,
  which reformats the whole file. That suits generated code. For a `vite.config.ts` or a
  `package.json` somebody wrote by hand it turns a one-line change into a diff of the file. These
  functions are built on a lossless syntax tree instead, so a comment above a member stays above
  it, and a file that was not touched is returned byte for byte.

  Every function returns the same tuple as the rest of this library — `{:ok, function, result}` or
  `{:error, function, reason}` — and every edit is **idempotent**: running it on its own output
  changes nothing, so a setup that stopped halfway can be run again.

  ## What can be edited

    * object and array members — `merge_object_members/4`, `merge_array_members/4`
    * JSX — `wrap_jsx/4`, `unwrap_jsx/3`, `add_jsx_props/4`
    * imports — `ensure_imports/4`, `move_imports/5`
    * and questions about a file — `object_members/3`, `member_values/3`, `jsx_elements/2`,
      `jsx_attributes/3`, `imports/2`

  ## Targets

  Object edits are aimed at a literal in the file:

  | target | the literal |
  |---|---|
  | `{:variable, "translations"}` | the initialiser of `const translations = {…}`, through `as const`, `satisfies` and parentheses |
  | `{:call_argument, "createRsk"}` | the first argument of `createRsk({…})`; `{:call_argument, "f", 1}` for a later one |
  | `:default_export` | what the file exports by default: `{…}`, `defineConfig({…})`, `defineConfig(() => ({…}))`, or a variable holding one |
  | `:document` | a whole JSON document (`package.json`, `tsconfig.json`), comments and trailing commas allowed |

  The `:path` option walks down from the target, and creates the levels that are missing.

  ## Dialects

  Pass `dialect:` as `:js`, `:jsx`, `:ts`, `:tsx` or `:json`. With the `:path` type the dialect
  comes from the file extension.

  ## Non-ASCII text

  Offsets are UTF-8 bytes throughout, so a comment in Persian, a zero-width non-joiner or an emoji
  before the edit point does not move the edit.
  """

  alias IgniterJs.Native
  import IgniterJs.Helpers, only: [call_nif_fn: 4, dialect_for: 3]

  @type target ::
          {:variable, String.t()}
          | {:call_argument, String.t()}
          | {:call_argument, String.t(), non_neg_integer()}
          | :default_export
          | :document

  @type source_type :: :content | :path

  # ── object and array members ──────────────────────────────────────────────

  @doc """
  Add members to an object literal, keeping the layout the object already has.

  `members` is a list of `{key, code}`: `key` is what "already there?" is asked about, `code` is
  what is written — the whole `key: value` for an object.

  Options:

    * `:path` — keys to walk down from the target, e.g. `["resolve", "alias"]`. Levels that do
      not exist are created.
    * `:mode` — `:add` (default) leaves a member that is already there alone, so the developer's
      value wins; `:set` replaces it.
    * `:at` — `:head` (default) puts new members before the existing ones; `:end` puts them
      after, the way a person adds one. `:end` keeps a trailing comma the developer wrote, and
      adds the separating comma itself when there is none; a comment after the old last member
      stays on its line. An empty container is filled either way.
    * `:dialect` — see the module docs.

  ```elixir
  alias IgniterJs.Parsers.Javascript.Edit

  # an inline object stays inline
  Edit.merge_object_members("export const t = { fa } as const", {:variable, "t"}, [{"en", "en: {}"}], dialect: :ts)
  #=> {:ok, :merge_object_members, "export const t = { en: {}, fa } as const"}
  ```

  A multiline object gets the member on its own line, indented like the first one, and a comment
  above a member stays above it:

  ```js
  export default {
    // keep me
    plugins: [],
  }
  ```

  ```elixir
  Edit.merge_object_members(source, :default_export, [{"base", ~s(base: "/")}], dialect: :ts)
  ```

  ```js
  export default {
    base: "/",
    // keep me
    plugins: [],
  }
  ```

  Creating the levels of a path — the case of a Vite config with no `resolve`:

  ```elixir
  Edit.merge_object_members(
    "export default {\\n  plugins: [],\\n}\\n",
    :default_export,
    [{"@rsk/core", ~s("@rsk/core": "/abs/core.ts")}],
    path: ["resolve", "alias"],
    dialect: :ts
  )
  ```

  ```js
  export default {
    resolve: {
      alias: {
        "@rsk/core": "/abs/core.ts",
      },
    },
    plugins: [],
  }
  ```

  `at: :end` for `package.json`, where `workspaces` belongs after the other keys:

  ```elixir
  Edit.merge_array_members(~s({\n  "name": "app"\n}\n), :document, ["vendor/@rsk/*"],
    path: ["workspaces"], at: :end, dialect: :json)
  #=> {:ok, :merge_array_members, ~s({\n  "name": "app",\n  "workspaces": [\n    "vendor/@rsk/*"\n  ]\n}\n)}
  ```

  A JSON document, with its key order and indentation kept:

  ```elixir
  Edit.merge_object_members(~s({\\n  "name": "app"\\n}\\n), :document, [{"type", ~s("type": "module")}], dialect: :json)
  #=> {:ok, :merge_object_members, ~s({\\n  "type": "module",\\n  "name": "app"\\n}\\n)}
  ```
  """
  @spec merge_object_members(String.t(), target(), [{String.t(), String.t()}], keyword()) ::
          {:ok, atom(), String.t()} | {:error, atom(), String.t()}
  def merge_object_members(file_path_or_content, target, members, opts \\ []) do
    run(file_path_or_content, __ENV__.function, opts, %{
      op: "merge_object_members",
      target: encode_target(target),
      path: Keyword.get(opts, :path, []),
      container: "object",
      members: encode_members(members),
      mode: Keyword.get(opts, :mode, :add),
      at: Keyword.get(opts, :at, :head)
    })
  end

  @doc """
  Add string elements to an array literal: `resolve.dedupe`, `workspaces`, `include`.

  `items` is a list of strings; an element already in the array is not added again.

  ```elixir
  Edit.merge_array_members(
    ~s({\\n  "workspaces": ["apps/*"]\\n}\\n),
    :document,
    ["vendor/@rsk/*"],
    path: ["workspaces"],
    dialect: :json
  )
  #=> {:ok, :merge_array_members, ~s({\\n  "workspaces": [ "vendor/@rsk/*", "apps/*"]\\n}\\n)}
  ```
  """
  @spec merge_array_members(String.t(), target(), [String.t()], keyword()) ::
          {:ok, atom(), String.t()} | {:error, atom(), String.t()}
  def merge_array_members(file_path_or_content, target, items, opts \\ []) do
    run(file_path_or_content, __ENV__.function, opts, %{
      op: "merge_object_members",
      target: encode_target(target),
      path: Keyword.get(opts, :path, []),
      container: "array",
      members: Enum.map(items, &%{key: &1, code: Jason.encode!(&1)}),
      mode: Keyword.get(opts, :mode, :add),
      at: Keyword.get(opts, :at, :head)
    })
  end

  # ── JSX ───────────────────────────────────────────────────────────────────

  @doc """
  Wrap the first `element` in `wrapper`, unless it is already wrapped.

  `:props` is a keyword list for the wrapper: `{:expr, "rsk"}` writes `setup={rsk}`,
  `{:string, "x"}` writes `x="x"`, and `true` writes a bare attribute.

  ```elixir
  Edit.wrap_jsx(main_tsx, "App", "RskApp",
    props: [setup: {:expr, "rsk"}, translations: {:expr, "translations"}],
    dialect: :tsx
  )
  ```

  ```jsx
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <App />
    </StrictMode>,
  )
  ```

  becomes

  ```jsx
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <RskApp setup={rsk} translations={translations}>
        <App />
      </RskApp>
    </StrictMode>,
  )
  ```

  A tree that is written on one line is wrapped on one line. If the file has no such element the
  result is `{:error, :wrap_jsx, "no <App> element to wrap"}`.
  """
  @spec wrap_jsx(String.t(), String.t(), String.t(), keyword()) ::
          {:ok, atom(), String.t()} | {:error, atom(), String.t()}
  def wrap_jsx(file_path_or_content, element, wrapper, opts \\ []) do
    run(file_path_or_content, __ENV__.function, with_dialect(opts, :tsx), %{
      op: "wrap_jsx",
      element: element,
      wrapper: wrapper,
      props: encode_props(Keyword.get(opts, :props, []))
    })
  end

  @doc """
  Replace the first `element` with its children. The reverse of `wrap_jsx/4`.

  ```jsx
  <StrictMode>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </StrictMode>
  ```

  ```elixir
  Edit.unwrap_jsx(source, "BrowserRouter", dialect: :tsx)
  ```

  ```jsx
  <StrictMode>
    <App />
  </StrictMode>
  ```

  Several children outside another element are put in a fragment (`<>…</>`) so the result is still
  one expression. An element that is not in the file changes nothing; one with no children is an
  error.
  """
  @spec unwrap_jsx(String.t(), String.t(), keyword()) ::
          {:ok, atom(), String.t()} | {:error, atom(), String.t()}
  def unwrap_jsx(file_path_or_content, element, opts \\ []) do
    run(file_path_or_content, __ENV__.function, with_dialect(opts, :tsx), %{
      op: "unwrap_jsx",
      element: element
    })
  end

  @doc """
  Add props to the first `element`. A prop that is already there is left alone unless
  `mode: :set`.

  ```elixir
  Edit.add_jsx_props("const a = <RskApp><App /></RskApp>", "RskApp", [setup: {:expr, "rsk"}], dialect: :tsx)
  #=> {:ok, :add_jsx_props, "const a = <RskApp setup={rsk}><App /></RskApp>"}
  ```
  """
  @spec add_jsx_props(String.t(), String.t(), keyword(), keyword()) ::
          {:ok, atom(), String.t()} | {:error, atom(), String.t()}
  def add_jsx_props(file_path_or_content, element, props, opts \\ []) do
    run(file_path_or_content, __ENV__.function, with_dialect(opts, :tsx), %{
      op: "add_jsx_props",
      element: element,
      props: encode_props(props),
      mode: Keyword.get(opts, :mode, :add)
    })
  end

  # ── imports ───────────────────────────────────────────────────────────────

  @doc """
  Make sure `names` are imported from `module`.

  A name that is already imported is not added again. The new name goes at the end of the braces of an
  existing import from that module, next to a default import, or on a new line after the last
  import — in the quote and semicolon style the file already uses.

  ```elixir
  Edit.ensure_imports(~s(import { createRsk } from "@rsk/core"\\n), "@rsk/core", ["i18nPreset"], dialect: :ts)
  #=> {:ok, :ensure_imports, ~s(import { createRsk, i18nPreset } from "@rsk/core"\\n)}

  Edit.ensure_imports("import a from 'a'\\n", "m", ["n"], dialect: :ts)
  #=> {:ok, :ensure_imports, "import a from 'a'\\nimport { n } from 'm'\\n"}
  ```

  A type-only import, a namespace import and a bare `import "m"` are not extended.
  """
  @spec ensure_imports(String.t(), String.t(), [String.t()], keyword()) ::
          {:ok, atom(), String.t()} | {:error, atom(), String.t()}
  def ensure_imports(file_path_or_content, module, names, opts \\ []) do
    run(file_path_or_content, __ENV__.function, opts, %{
      op: "ensure_imports",
      module: module,
      names: names
    })
  end

  @doc """
  Move named imports from one module to another.

  ```elixir
  Edit.move_imports(
    ~s(import { createRsk, bearerAuth } from "@rsk/core"\\n),
    "@rsk/core",
    "@rsk/http",
    ["bearerAuth"],
    dialect: :ts
  )
  #=> {:ok, :move_imports, ~s(import { createRsk } from "@rsk/core"\\nimport { bearerAuth } from "@rsk/http"\\n)}
  ```

  An import left with no names is removed; one that keeps others loses only the moved names, and
  what is written between the others, comments included, is untouched.
  """
  @spec move_imports(String.t(), String.t(), String.t(), [String.t()], keyword()) ::
          {:ok, atom(), String.t()} | {:error, atom(), String.t()}
  def move_imports(file_path_or_content, from, to, names, opts \\ []) do
    run(file_path_or_content, __ENV__.function, opts, %{
      op: "move_imports",
      from: from,
      to: to,
      names: names
    })
  end

  # ── questions ─────────────────────────────────────────────────────────────

  @doc """
  The member names of an object (or array) literal, in source order.

  Options: `:path`, `:dialect`, and `container: :array`.

  ```elixir
  Edit.object_members("export const t = { fa: {}, en: {} } as const", {:variable, "t"}, dialect: :ts)
  #=> {:ok, :object_members, ["fa", "en"]}
  ```

  A target or path that is not in the file is `{:error, :object_members, reason}`, not an empty
  list: "the file has no `resolve`" and "`resolve` is empty" are different answers.
  """
  @spec object_members(String.t(), target(), keyword()) ::
          {:ok, atom(), [String.t()]} | {:error, atom(), String.t()}
  def object_members(file_path_or_content, target, opts \\ []) do
    ask(file_path_or_content, __ENV__.function, opts, %{
      op: "object_members",
      target: encode_target(target),
      path: Keyword.get(opts, :path, []),
      container: Keyword.get(opts, :container, :object)
    })
  end

  @doc """
  The members of an object literal with their values **as written**: `[{name, value_source}]`.

  For telling what a member says, not just that it is there — an alias that is a relative string
  from one that is built from `import.meta.url`:

  ```elixir
  Edit.member_values(
    ~s|export default { resolve: { alias: { "@": "./src", a: fileURLToPath(x) } } }|,
    :default_export,
    path: ["resolve", "alias"],
    dialect: :ts
  )
  #=> {:ok, :member_values, [{"@", ~s|"./src"|}, {"a", "fileURLToPath(x)"}]}
  ```

  A shorthand member (`{ fa }`) is its own value. Options are those of `object_members/3`.
  """
  @spec member_values(String.t(), target(), keyword()) ::
          {:ok, atom(), [{String.t(), String.t()}]} | {:error, atom(), String.t()}
  def member_values(file_path_or_content, target, opts \\ []) do
    file_path_or_content
    |> ask(__ENV__.function, opts, %{
      op: "member_values",
      target: encode_target(target),
      path: Keyword.get(opts, :path, []),
      container: Keyword.get(opts, :container, :object)
    })
    |> case do
      {:ok, function, pairs} -> {:ok, function, Enum.map(pairs, fn [k, v] -> {k, v} end)}
      error -> error
    end
  end

  @doc """
  Every JSX attribute with a given name, in source order.

  Each is a map: `"element"` it is on, `"name"`, `"value"` as written (`nil` for a bare attribute
  such as `hidden`) and `"literal"` — `true` for a plain string (`className="flex"`), `false` for
  an expression (`className={cls}`).

  Found in the syntax tree, so a comment or docblock that merely *shows* the attribute is not one:

  ```elixir
  Edit.jsx_attributes(~s|const a = <div className="flex" id={x}><p className={c} /></div>|, "className", dialect: :tsx)
  #=> {:ok, :jsx_attributes, [
  #     %{"element" => "div", "name" => "className", "value" => ~s|"flex"|, "literal" => true},
  #     %{"element" => "p", "name" => "className", "value" => "{c}", "literal" => false}
  #   ]}
  ```
  """
  @spec jsx_attributes(String.t(), String.t(), keyword()) ::
          {:ok, atom(), [map()]} | {:error, atom(), String.t()}
  def jsx_attributes(file_path_or_content, name, opts \\ []) do
    ask(file_path_or_content, __ENV__.function, with_dialect(opts, :tsx), %{
      op: "jsx_attributes",
      name: name
    })
  end

  @doc """
  Every JSX element name in the file, in source order, repeats included.

  ```elixir
  Edit.jsx_elements("const a = <A><B /><C.D /></A>", dialect: :tsx)
  #=> {:ok, :jsx_elements, ["A", "B", "C.D"]}
  ```
  """
  @spec jsx_elements(String.t(), keyword()) ::
          {:ok, atom(), [String.t()]} | {:error, atom(), String.t()}
  def jsx_elements(file_path_or_content, opts \\ []) do
    ask(file_path_or_content, __ENV__.function, with_dialect(opts, :tsx), %{op: "jsx_elements"})
  end

  @doc """
  Every `import … from "…"` in the file.

  Each is a map with `"module"`, `"names"` (the imported names, before any `as`), `"default"`,
  `"namespace"` and `"type_only"`.

  ```elixir
  Edit.imports(~s(import a, { b, c as d } from "m"\\nimport type { T } from "t"\\n), dialect: :ts)
  #=> {:ok, :imports, [
  #=>   %{"module" => "m", "names" => ["b", "c"], "default" => true, "namespace" => false, "type_only" => false},
  #=>   %{"module" => "t", "names" => ["T"], "default" => false, "namespace" => false, "type_only" => true}
  #=> ]}
  ```
  """
  @spec imports(String.t(), keyword()) ::
          {:ok, atom(), [map()]} | {:error, atom(), String.t()}
  def imports(file_path_or_content, opts \\ []) do
    ask(file_path_or_content, __ENV__.function, opts, %{op: "imports"})
  end

  # ── plumbing ──────────────────────────────────────────────────────────────

  defp run(file_path_or_content, function, opts, request) do
    type = Keyword.get(opts, :type, :content)

    call_nif_fn(
      file_path_or_content,
      function,
      fn content ->
        Native.edit_source_nif(
          content,
          dialect_for(file_path_or_content, type, opts),
          Jason.encode!(request)
        )
      end,
      type
    )
  end

  defp ask(file_path_or_content, function, opts, request) do
    type = Keyword.get(opts, :type, :content)

    file_path_or_content
    |> call_nif_fn(
      function,
      fn content ->
        Native.query_source_nif(
          content,
          dialect_for(file_path_or_content, type, opts),
          Jason.encode!(request)
        )
      end,
      type
    )
    |> decode_answer()
  end

  defp decode_answer({:ok, function, json}), do: {:ok, function, Jason.decode!(json)}
  defp decode_answer(error), do: error

  # JSX needs a JSX dialect; saying nothing means TSX, the superset, so a caller does not have
  # to know what extension the file has.
  defp with_dialect(opts, default) do
    if Keyword.has_key?(opts, :dialect) or Keyword.get(opts, :type) == :path,
      do: opts,
      else: Keyword.put(opts, :dialect, default)
  end

  defp encode_target({:variable, name}), do: %{kind: "variable", name: name}

  defp encode_target({:call_argument, callee}),
    do: %{kind: "call_argument", callee: callee, index: 0}

  defp encode_target({:call_argument, callee, index}),
    do: %{kind: "call_argument", callee: callee, index: index}

  defp encode_target(:default_export), do: %{kind: "default_export"}
  defp encode_target(:document), do: %{kind: "document"}

  defp encode_members(members),
    do: Enum.map(members, fn {key, code} -> %{key: key, code: code} end)

  defp encode_props(props) do
    Enum.map(props, fn
      {name, true} -> %{name: to_string(name)}
      {name, {:expr, code}} -> %{name: to_string(name), value: "{" <> code <> "}"}
      {name, {:string, text}} -> %{name: to_string(name), value: Jason.encode!(text)}
    end)
  end
end
