defmodule IgniterJs.Native do
  @moduledoc false
  # use Rustler, otp_app: :igniter_js, crate: "igniter_js"

  mix_config = Mix.Project.config()
  version = mix_config[:version]
  github_url = mix_config[:package][:links]["GitHub"]

  use RustlerPrecompiled,
    otp_app: :igniter_js,
    crate: "igniter_js",
    base_url: "#{github_url}/releases/download/v#{version}",
    version: version,
    targets: ~w(
        aarch64-apple-darwin
        aarch64-unknown-linux-gnu
        aarch64-unknown-linux-musl
        riscv64gc-unknown-linux-gnu
        x86_64-apple-darwin
        x86_64-pc-windows-gnu
        x86_64-pc-windows-msvc
        x86_64-unknown-freebsd
        x86_64-unknown-linux-gnu
        x86_64-unknown-linux-musl
      ),
    force_build: System.get_env("IGNITERJS_BUILD") in ["1", "true"]

  # When your NIF is loaded, it will override this function.
  def is_module_imported_from_ast_nif(_file_content, _module_name), do: error()

  def insert_import_to_ast_nif(_file_content, _import_lines), do: error()

  def remove_import_from_ast_nif(_file_content, _modules), do: error()

  def find_live_socket_node_from_ast_nif(_file_content), do: error()

  def extend_hook_object_to_ast_nif(_file_content, _names), do: error()

  def remove_objects_of_hooks_from_ast_nif(_file_content, _object_names), do: error()

  defp error, do: :erlang.nif_error(:nif_not_loaded)
end
