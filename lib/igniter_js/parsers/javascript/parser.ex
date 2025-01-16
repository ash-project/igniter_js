defmodule IgniterJs.Parsers.Javascript.Parser do
  @moduledoc """
  Codemods for JavaScript files.
  """

  alias IgniterJs.Native
  import IgniterJs.Helpers, only: [call_nif_fn: 4]

  @doc """
  Check if a module is imported in the given file or content or content and returns boolean.

  ```elixir
  alias IgniterJs.Parsers.Javascript.Parser
  Parser.module_imported?(js_content, "module")
  Parser.module_imported?(js_content, "module", :content)
  Parser.module_imported?("/path/to/file.js", "module", :path)
  ```
  """
  def module_imported?(file_path_or_content, module, type \\ :content) do
    elem(module_imported(file_path_or_content, module, type), 0) == :ok
  end

  @doc """
  Check if a module is imported in the given file or content or contents and return tuple.

  ```elixir
  alias IgniterJs.Parsers.Javascript.Parser
  Parser.module_imported(js_content, "module")
  Parser.module_imported(js_content, "module", :content)
  Parser.module_imported("/path/to/file.js", "module", :path)
  ```
  """
  def module_imported(file_path_or_content, module, type \\ :content) do
    call_nif_fn(
      file_path_or_content,
      __ENV__.function,
      fn file_content ->
        Native.is_module_imported_from_ast_nif(file_content, module)
      end,
      type
    )
  end

  @doc """
  Insert imports to the given file or content and returns tuple.

  ```elixir
  alias IgniterJs.Parsers.Javascript.Parser
  Parser.insert_imports(js_content, imports_lines)
  Parser.insert_imports(js_content, imports_lines, :content)
  Parser.insert_imports("/path/to/file.js", imports_lines, :path)
  ```
  """
  def insert_imports(file_path_or_content, imports_lines, type \\ :content) do
    call_nif_fn(
      file_path_or_content,
      __ENV__.function,
      fn file_content ->
        Native.insert_import_to_ast_nif(file_content, imports_lines)
      end,
      type
    )
  end

  @doc """
  Remove imports from the given file or content. it accepts a single module or a list of modules.
  It returns a tuple.

  ```elixir
  alias IgniterJs.Parsers.Javascript.Parser
  Parser.remove_imports(js_content, "SomeModule")
  Parser.remove_imports("/path/to/file.js", "SomeModule", :path)
  ```
  """
  def remove_imports(file_path_or_content, module, type \\ :content)

  def remove_imports(file_path_or_content, modules, type) do
    call_nif_fn(
      file_path_or_content,
      __ENV__.function,
      fn file_content ->
        Native.remove_import_from_ast_nif(file_content, modules)
      end,
      type
    )
  end

  @doc """
  Check if a LiveSocket var exists in the given file or content and returns boolean.

  ```elixir
  alias IgniterJs.Parsers.Javascript.Parser
  Parser.exist_live_socket?(js_content)
  Parser.exist_live_socket?(js_content, :content)
  Parser.exist_live_socket?("/path/to/file.js", :path)
  ```
  """
  def exist_live_socket?(file_path_or_content, type \\ :content) do
    elem(exist_live_socket(file_path_or_content, type), 0) == :ok
  end

  @doc """
  Check if a LiveSocket var exists in the given file or content and returns tuple.

  ```elixir
  alias IgniterJs.Parsers.Javascript.Parser
  Parser.exist_live_socket(js_content)
  Parser.exist_live_socket(js_content, :content)
  Parser.exist_live_socket("/path/to/file.js", :path)
  ```
  """
  def exist_live_socket(file_path_or_content, type \\ :content) do
    call_nif_fn(
      file_path_or_content,
      __ENV__.function,
      fn file_content ->
        Native.find_live_socket_node_from_ast_nif(file_content)
      end,
      type
    )
  end

  @doc """
  Check if a specific var exists in the given file or content and returns boolean.

  ```elixir
  alias IgniterJs.Parsers.Javascript.Parser
  Parser.exist_live_socket?(js_content)
  Parser.exist_live_socket?(js_content, :content)
  Parser.exist_live_socket?("/path/to/file.js", :path)
  ```
  """
  def var_exists?(file_path_or_content, type \\ :content) do
    elem(exist_var(file_path_or_content, type), 0) == :ok
  end

  @doc """
  Check if an specific var exists in the given file or content and returns tuple.

  ```elixir
  alias IgniterJs.Parsers.Javascript.Parser
  Parser.exist_live_socket(js_content, var_name)
  Parser.exist_live_socket(js_content, var_name, :content)
  Parser.exist_live_socket("/path/to/file.js", var_name, :path)
  ```
  """
  def exist_var(file_path_or_content, var_name, type \\ :content) do
    call_nif_fn(
      file_path_or_content,
      __ENV__.function,
      fn file_content ->
        Native.contains_variable_from_ast_nif(file_content, var_name)
      end,
      type
    )
  end

  @doc """
  Extend the hook object in the given file or content. It accepts a single object
  or a list of objects.
  It returns a tuple.

  ```elixir
  alias IgniterJs.Parsers.Javascript.Parser
  Parser.extend_hook_object(js_content, "SomeObject")
  Parser.extend_hook_object(js_content, ["SomeObject", "AnotherObject"], :content)
  Parser.extend_hook_object("/path/to/file.js", "SomeObject", :path)
  ```
  """
  def extend_hook_object(file_path_or_content, object_name, type \\ :content)

  def extend_hook_object(file_path_or_content, object_name, type) when is_binary(object_name) do
    extend_hook_object(file_path_or_content, [object_name], type)
  end

  def extend_hook_object(file_path_or_content, objects_names, type) when is_list(objects_names) do
    call_nif_fn(
      file_path_or_content,
      __ENV__.function,
      fn file_content ->
        Native.extend_hook_object_to_ast_nif(file_content, objects_names)
      end,
      type
    )
  end

  @doc """
  Remove objects from the hooks in the given file or content. It accepts a single o
  bject or a list of objects.
  It returns a tuple.

  ```elixir
  alias IgniterJs.Parsers.Javascript.Parser
  Parser.remove_objects_from_hooks(js_content, "SomeObject")
  Parser.remove_objects_from_hooks(js_content, ["SomeObject", "AnotherObject"], :content)
  Parser.remove_objects_from_hooks("/path/to/file.js", "SomeObject", :path)
  ```
  """
  def remove_objects_from_hooks(file_path_or_content, object_name, type \\ :content)

  def remove_objects_from_hooks(file_path_or_content, object_name, type)
      when is_binary(object_name) do
    remove_objects_from_hooks(file_path_or_content, [object_name], type)
  end

  def remove_objects_from_hooks(file_path_or_content, objects_names, type)
      when is_list(objects_names) do
    call_nif_fn(
      file_path_or_content,
      __ENV__.function,
      fn file_content ->
        Native.remove_objects_of_hooks_from_ast_nif(file_content, objects_names)
      end,
      type
    )
  end

  @doc """
  Retrieve statistical information about the JavaScript source code, such as the number of
  functions, classes, debugger statements, imports, try-catch blocks, and throw statements.

  This function accepts either the content of the JavaScript file or the path to the file,
  and returns a tuple with the status, function atom, and the extracted data as a map.

  ## Examples

  ```elixir
  alias IgniterJs.Parsers.Javascript.Parser

  # Analyze a JavaScript source file by providing its content
  Parser.statistics(js_content)

  # Analyze a JavaScript source file by providing its file path
  Parser.statistics("/path/to/file.js", :path)
  ```
  """
  def statistics(file_path_or_content, type \\ :content) do
    {status, fn_atom, {_, data}} =
      call_nif_fn(
        file_path_or_content,
        __ENV__.function,
        fn file_content ->
          Native.statistics_from_ast_nif(file_content)
        end,
        type
      )

    converted = if is_map(data), do: Map.drop(data, [:__struct__]), else: data
    {status, fn_atom, converted}
  end

  @doc """
    Extend a variable of object type in the given file or content by adding additional objects to it,
    based on their names.

    This function ensures that duplicate entries are not added during the process.

    This function accepts:
    - The content or path of the JavaScript file.
    - The name of the variable to be extended.
    - A single object name or a list of object names to be added.
    - The type indicating whether it's content (`:content`) or a path (`:path`).

    It returns a tuple with the status, function atom, and the updated content or an error message
    if the variable could not be found or modified.

    ## Examples

    ```elixir
    alias IgniterJs.Parsers.Javascript.Parser

    objects_names = ["OXCTestHook", "MishkaHooks", "MishkaHooks", "OXCTestHook"]

    Parser.extend_var_object_by_object_names(js_content, "Components", "TestHook")
    Parser.extend_var_object_by_object_names("/path/to/file.js", "Components", objects_names, :path)

    {:error, :extend_var_object_by_object_names, _output} =
      Parser.extend_var_object_by_object_names("None", "Components", objects_names)
    ```
  """
  def extend_var_object_by_object_names(file_path_or_content, var, object_names, type \\ :content)

  def extend_var_object_by_object_names(file_path_or_content, var, object_name, type)
      when is_binary(object_name) do
    extend_var_object_by_object_names(file_path_or_content, var, [object_name], type)
  end

  def extend_var_object_by_object_names(file_path_or_content, var, object_names, type) do
    call_nif_fn(
      file_path_or_content,
      __ENV__.function,
      fn file_content ->
        Native.extend_var_object_property_by_names_to_ast_nif(file_content, var, object_names)
      end,
      type
    )
  end
end
