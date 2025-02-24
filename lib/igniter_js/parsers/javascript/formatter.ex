defmodule IgniterJs.Parsers.Javascript.Formatter do
  @moduledoc """
  This module provides functions to check if a JavaScript file is formatted
  and to format JavaScript files. It interacts with the Native NIF (Native
  Implemented Functions) for performing the actual formatting and checking.
  """

  alias IgniterJs.Native
  import IgniterJs.Helpers, only: [call_nif_fn: 4]

  @doc """
  Checks if the provided JavaScript content or file is formatted.

  This function returns `true` if the content is formatted, and `false`
  otherwise. The type can either be `:content` (default) or `:path`.

  ## Parameters

    - `file_path_or_content`: The JavaScript file path or content to check.
    - `type`: The type of the input, either `:content` or `:path`.

  ## Examples

      iex> IgniterJs.Parsers.Javascript.Formatter.is_formatted?("path/to/file.js")
      true

  """
  def is_formatted?(file_path_or_content, type \\ :content) do
    elem(is_formatted(file_path_or_content, type), 0) == :ok
  end

  @doc """
  Checks if the provided JavaScript content or file is formatted.

  This function returns the status of the formatting check.

  ## Parameters

    - `file_path_or_content`: The JavaScript file path or content to check.
    - `type`: The type of the input, either `:content` or `:path`.

  ## Examples

      iex> IgniterJs.Parsers.Javascript.Formatter.is_formatted("path/to/file.js")
      {:ok, :is_formatted, true}

  """
  def is_formatted(file_path_or_content, type \\ :content) do
    call_nif_fn(
      file_path_or_content,
      __ENV__.function,
      fn file_content ->
        Native.is_js_formatted_nif(file_content)
      end,
      type
    )
  end

  def format(file_path_or_content, type \\ :content) do
    call_nif_fn(
      file_path_or_content,
      __ENV__.function,
      fn file_content ->
        Native.format_js_nif(file_content)
      end,
      type
    )
  end
end
