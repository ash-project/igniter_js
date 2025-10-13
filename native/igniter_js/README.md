<!--
SPDX-FileCopyrightText: 2024 Shahryar Tavakkoli

SPDX-License-Identifier: MIT
-->

# NIF for Elixir.IgniterJs.Native

## To build the NIF module:

- Your NIF will now build along with your project.

## To load the NIF:

```elixir
defmodule IgniterJs.Native do
  use Rustler, otp_app: :igniter_js, crate: "igniter_js"

  # When your NIF is loaded, it will override this function.
  def add(_a, _b), do: :erlang.nif_error(:nif_not_loaded)
end
```

## Examples

[This](https://github.com/rusterlium/NifIo) is a complete example of a NIF written in Rust.
