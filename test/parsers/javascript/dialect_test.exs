# SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs/contributors>
#
# SPDX-License-Identifier: MIT

defmodule IgniterJSTest.Parsers.Javascript.DialectTest do
  @moduledoc """
  TypeScript and JSX sources, which this library could not parse at all.

  Every parser hardcoded plain ECMAScript in three separate places — the codemods used
  `Syntax::Es(Default::default())` under a virtual filename of `virtual_file.js`, the ESTree dump
  asked oxc for `SourceType::from_path("example.js")`, and the formatter used
  `JsFileSource::default()`. A `.ts` or `.tsx` file was not badly handled, it was rejected, so a
  caller could not so much as insert an import into a React component.

  The rule these tests pin down: **anything that parsed before parses the same way**. Plain
  JavaScript is still the default for content, so no existing caller changes behaviour. The
  dialect is opt-in, either by naming it or by passing a path whose extension implies it.
  """

  use ExUnit.Case, async: true

  alias IgniterJs.Helpers
  alias IgniterJs.Parsers.Javascript.{Formatter, Parser}

  @ts """
  import { createRsk } from "@rsk/core"

  interface Options {
    baseURL: string
  }

  const options: Options = { baseURL: "/api" }
  export const rsk = createRsk(options)
  """

  @tsx """
  import React from "react"
  import { RskApp } from "@rsk/router"

  export const App = (): JSX.Element => <RskApp><div className="root">hi</div></RskApp>
  """

  @jsx """
  import React from "react"
  export const App = () => <div className="root">hi</div>
  """

  describe "resolving which dialect to use" do
    test "content with no opinion is JavaScript — that is what keeps old callers working" do
      assert Helpers.dialect_for("const a = 1", :content, []) == "js"
    end

    test "a path is read from its extension" do
      assert Helpers.dialect_for("vite.config.ts", :path, []) == "ts"
      assert Helpers.dialect_for("src/main.tsx", :path, []) == "tsx"
      assert Helpers.dialect_for("Component.jsx", :path, []) == "jsx"
      assert Helpers.dialect_for("app.js", :path, []) == "js"
    end

    test "the module variants map onto their base dialect" do
      for ext <- ~w(.mts .cts), do: assert(Helpers.dialect_of_path("x#{ext}") == "ts")
      for ext <- ~w(.mjs .cjs), do: assert(Helpers.dialect_of_path("x#{ext}") == "js")
    end

    test "an explicit option beats the extension" do
      assert Helpers.dialect_for("x.js", :path, dialect: :tsx) == "tsx"
      assert Helpers.dialect_for("x.tsx", :path, dialect: :js) == "js"
    end

    test "an unknown extension falls back to JavaScript rather than failing" do
      assert Helpers.dialect_of_path("Makefile") == "js"
      assert Helpers.dialect_of_path("styles.css") == "js"
    end

    test "extensions are matched case-insensitively" do
      assert Helpers.dialect_of_path("Main.TSX") == "tsx"
    end
  end

  describe "TypeScript" do
    test "an import is found when the dialect says TypeScript, and not before" do
      # The `false` here is not a bug being asserted — it is the old behaviour, kept, so that a
      # caller who never asked for TypeScript sees exactly what they saw before.
      statement = ~s|import { createRsk } from "@rsk/core";|

      refute Parser.module_imported?(@ts, statement)
      assert Parser.module_imported?(@ts, statement, :content, dialect: :ts)
    end

    test "an import can be inserted into a file with type annotations" do
      assert {:ok, _, out} =
               Parser.insert_imports(@ts, ~s|import { z } from "zod"|, :content, dialect: :ts)

      assert out =~ ~s|import { z } from "zod"|
      assert out =~ "interface Options", "the interface must survive the round trip"
      assert out =~ "const options: Options", "the annotation must survive too"
    end

    test "it formats without losing its types" do
      assert {:ok, _, out} = Formatter.format(@ts, :content, dialect: :ts)
      assert out =~ "interface Options"
      assert out =~ ": Options"
    end
  end

  describe "JSX and TSX" do
    test "a React component parses as jsx" do
      assert Parser.module_imported?(@jsx, ~s|import React from "react";|, :content,
               dialect: :jsx
             )
    end

    test "a TypeScript React component parses as tsx" do
      assert Parser.module_imported?(@tsx, ~s|import { RskApp } from "@rsk/router";|, :content,
               dialect: :tsx
             )
    end

    test "an import can be inserted into a component that returns JSX" do
      assert {:ok, _, out} =
               Parser.insert_imports(@tsx, ~s|import { rsk } from "./rsk"|, :content,
                 dialect: :tsx
               )

      assert out =~ ~s|from "./rsk"|
      assert out =~ "<RskApp>", "the JSX must survive the round trip"
      assert out =~ "className=", "attributes too"
    end

    test "tsx formats without losing either the types or the elements" do
      assert {:ok, _, out} = Formatter.format(@tsx, :content, dialect: :tsx)
      assert out =~ "JSX.Element"
      assert out =~ "<div"
    end

    # In .tsx, `<T>(x)` is a JSX element; in .ts it is a type assertion. Collapsing the two would
    # silently mis-parse one of them, which is why they stay separate dialects.
    test "ts and tsx are not interchangeable" do
      angle_bracket_assertion = "const a = <string>someValue;"

      assert {:ok, _, _} = Formatter.format(angle_bracket_assertion, :content, dialect: :ts)
      assert {:error, _, _} = Formatter.format(angle_bracket_assertion, :content, dialect: :tsx)
    end
  end

  describe "reading structure" do
    test "the ESTree dump handles TypeScript, which it could not before" do
      assert {:ok, _, json} = Parser.ast_to_estree(@ts, :content, dialect: :ts)
      assert is_map(json) or is_binary(json)
    end
  end

  describe "backward compatibility" do
    test "plain JavaScript is untouched by any of this" do
      js = ~s|import { a } from "m"\nconst x = 1\n|

      assert {:ok, _, out} = Formatter.format(js)
      assert out == ~s|import { a } from "m";\nconst x = 1;\n|
    end

    test "every dialect-aware function still works with its original arity" do
      assert {:ok, _, _} = Formatter.format("const x=1")
      assert is_boolean(Formatter.is_formatted?("const x = 1;\n"))

      assert is_boolean(
               Parser.module_imported?(~s|import { a } from "m";|, ~s|import { a } from "m";|)
             )

      assert is_boolean(Parser.var_exists?("let a = 1", "a"))
      assert {:ok, _, _} = Parser.statistics("const a = 1")
    end

    test "an unknown dialect is refused rather than quietly treated as JavaScript" do
      assert {:error, _, _} = Formatter.format("const x = 1", :content, dialect: :coffee)
    end
  end

  describe "variable declarations" do
    # `contains_variable_from_ast` matched only `VarDeclKind::Let`, so `const` — the common form
    # in every modern codebase — was invisible. It went unnoticed because the Phoenix `app.js`
    # this was written for declares `let Hooks = {}`.
    test "const, let and var are all found" do
      assert Parser.var_exists?("const Hooks = {}", "Hooks")
      assert Parser.var_exists?("let Hooks = {}", "Hooks")
      assert Parser.var_exists?("var Hooks = {}", "Hooks")
    end

    test "a variable that is not there is still not found" do
      refute Parser.var_exists?("const Other = {}", "Hooks")
    end

    test "it works in TypeScript too" do
      assert Parser.var_exists?("const rsk: Rsk = create()", "rsk", :content, dialect: :ts)
    end
  end
end
