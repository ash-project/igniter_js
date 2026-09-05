# SPDX-FileCopyrightText: 2024 Shahryar Tavakkoli
# SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs/contributors>
#
# SPDX-License-Identifier: MIT

defmodule IgniterJs.Helpers do
  @moduledoc """
  A module that contains helper functions for IgniterJs. For example it helps to normalize the
  output of the NIFs, read and validate the file, and call the NIF function with the
  given file path or content.
  """

  @doc """
  Normalize the output of the NIFs. It is a macro and returns a tuple with the first
  element as the output, the second element as the caller function name, and the
  third element as the status.

  ```elixir
  require IgniterJs.Helpers
  normalize_output({:ok, :fun_atom, result}, __ENV__.function)
  normalize_output({:error, :fun_atom, result}, __ENV__.function)
  ```
  """
  defmacro normalize_output(output, caller_function) do
    quote do
      {elem(unquote(output), 0), elem(unquote(caller_function), 0), elem(unquote(output), 2)}
    end
  end

  @doc """
  Read and validate the file. It returns the file content if the file exists and the
  extension is `.js` or `.ts`, otherwise, it returns an error tuple.

  ```elixir
  read_and_validate_file("/path/to/file.js")
  ```
  """
  # sobelow_skip ["Traversal.FileModule"]
  def read_and_validate_file(file_path) do
    with true <- File.exists?(file_path),
         true <- Path.extname(file_path) in [".js", ".ts"],
         {:ok, file_content} <- File.read(file_path) do
      {:ok, file_content}
    else
      {:error, reason} -> {:error, reason}
      _ -> {:error, "Invalid file path or format."}
    end
  end

  @doc """
  Call the NIF function with the given file path or content and return the result.
  It helps to change the function name as atom based on its caller function.

  ```elixir
  call_nif_fn("/path/to/file.js", __ENV__.function, fn content -> content end, :path)
  call_nif_fn("file content", __ENV__.function, fn content -> content end)
  call_nif_fn("file content", __ENV__.function, fn content -> content end, :content)
  ```
  """

  def call_nif_fn(file_path, caller_function, processing_fn, type \\ :content)

  def call_nif_fn(file_content, caller_function, processing_fn, :content) do
    processing_fn.(file_content)
    |> normalize_output(caller_function)
  end

  def call_nif_fn(file_path, caller_function, processing_fn, :path) do
    case read_and_validate_file(file_path) do
      {:ok, file_content} ->
        processing_fn.(file_content)
        |> normalize_output(caller_function)

      reason ->
        Tuple.insert_at(reason, 1, :none)
        |> normalize_output(caller_function)
    end
  end

  @doc """
  Which dialect a source should be parsed as: `"js"`, `"jsx"`, `"ts"` or `"tsx"`.

  Resolution order:

    1. an explicit `:dialect` option -- `dialect: :tsx`
    2. the file extension, when `type` is `:path`
    3. `"js"`

  Step 3 is what keeps this backward compatible. A caller passing content and saying nothing gets
  exactly the parser configuration this library used before dialects existed, so anything that
  parsed before parses the same way. Step 2 is new behaviour, but it can only affect files that
  did not parse at all previously: reading `app.tsx` and parsing it as JavaScript never produced
  an answer worth keeping.

      iex> IgniterJs.Helpers.dialect_for("x.tsx", :path, [])
      "tsx"

      iex> IgniterJs.Helpers.dialect_for("const a = 1", :content, [])
      "js"

      iex> IgniterJs.Helpers.dialect_for("x.js", :path, dialect: :tsx)
      "tsx"
  """
  @spec dialect_for(String.t(), :content | :path, keyword()) :: String.t()
  def dialect_for(file_path_or_content, type, opts) do
    case Keyword.get(opts, :dialect) do
      nil -> if type == :path, do: dialect_of_path(file_path_or_content), else: "js"
      dialect -> to_string(dialect)
    end
  end

  @doc """
  The dialect implied by a path's extension, defaulting to `"js"`.

      iex> IgniterJs.Helpers.dialect_of_path("vite.config.ts")
      "ts"

      iex> IgniterJs.Helpers.dialect_of_path("Makefile")
      "js"
  """
  @spec dialect_of_path(String.t()) :: String.t()
  def dialect_of_path(path) do
    case path |> Path.extname() |> String.downcase() do
      ext when ext in [".ts", ".mts", ".cts"] -> "ts"
      ".tsx" -> "tsx"
      ".jsx" -> "jsx"
      _ -> "js"
    end
  end
end
