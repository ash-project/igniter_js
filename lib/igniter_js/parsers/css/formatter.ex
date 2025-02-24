defmodule IgniterJs.Parsers.CSS.Formatter do
  @moduledoc """
  This module provides functions to check if a CSS file is formatted
  and to format CSS files. It interacts with the Native NIF (Native
  Implemented Functions) for performing the actual formatting and checking.
  """

  # We set version of biemojs based on https://github.com/brioche-dev/brioche/pull/184

  alias IgniterJs.Native
  import IgniterJs.Helpers, only: [call_nif_fn: 4]

  @doc """
  Checks if the provided CSS content or file is formatted.

  This function returns `true` if the content is formatted, and `false`
  otherwise. The type can either be `:content` (default) or `:path`.

  ## Parameters

    - `file_path_or_content`: The CSS file path or content to check.
    - `type`: The type of the input, either `:content` or `:path`.

  ## Examples

      iex> IgniterJs.Parsers.Javascript.Formatter.is_formatted?("path/to/file.js")
      true

  """
  def is_formatted?(file_path_or_content, type \\ :content) do
    elem(is_formatted(file_path_or_content, type), 0) == :ok
  end

  @doc """
  Checks if the provided CSS content or file is formatted.

  This function returns the status of the formatting check.

  ## Parameters

    - `file_path_or_content`: The CSS file path or content to check.
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
        Native.is_css_formatted_nif(file_content)
      end,
      type
    )
  end

  def format(file_path_or_content, type \\ :content) do
    call_nif_fn(
      file_path_or_content,
      __ENV__.function,
      fn file_content ->
        Native.format_css_nif(file_content)
      end,
      type
    )
  end
end
