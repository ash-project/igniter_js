defmodule IgniterJS.Parsers.Javascript.Parser do
  alias IgniterJS.Native
  import IgniterJS.Helpers, only: [call_nif_fn: 4]

  @doc """
  Check if a module is imported in the given file or content or content and returns boolean.

  ```elixir
  alias IgniterJS.Parsers.Javascript.Parser
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
  alias IgniterJS.Parsers.Javascript.Parser
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
  alias IgniterJS.Parsers.Javascript.Parser
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
  alias IgniterJS.Parsers.Javascript.Parser
  Parser.remove_imports(js_content, "SomeModule")
  Parser.remove_imports(js_content, ["SomeModule", "AnotherModule"], :content)
  Parser.remove_imports("/path/to/file.js", "SomeModule", :path)
  ```
  """
  def remove_imports(file_path_or_content, module, type \\ :content)

  def remove_imports(file_path_or_content, module, type) when is_binary(module) do
    remove_imports(file_path_or_content, [module], type)
  end

  def remove_imports(file_path_or_content, modules, type) when is_list(modules) do
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
  alias IgniterJS.Parsers.Javascript.Parser
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
  alias IgniterJS.Parsers.Javascript.Parser
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
  Extend the hook object in the given file or content. It accepts a single object
  or a list of objects.
  It returns a tuple.

  ```elixir
  alias IgniterJS.Parsers.Javascript.Parser
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
  alias IgniterJS.Parsers.Javascript.Parser
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
end
