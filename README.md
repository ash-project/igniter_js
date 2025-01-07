<img src="https://github.com/ash-project/igniter/blob/main/logos/igniter-logo-small.png?raw=true#gh-light-mode-only" alt="Logo Light" width="250">
<img src="https://github.com/ash-project/igniter/blob/main/logos/igniter-logo-small.png?raw=true#gh-dark-mode-only" alt="Logo Dark" width="250">

[![CI](https://github.com/ash-project/igniter_js/actions/workflows/elixir.yml/badge.svg)](https://github.com/ash-project/igniter_js/actions/workflows/elixir.yml)
[![Hex version badge](https://img.shields.io/hexpm/v/igniter_js.svg)](https://hex.pm/packages/igniter_js)
[![Hexdocs badge](https://img.shields.io/badge/docs-hexdocs-purple)](https://hexdocs.pm/igniter_js)

# IgniterJs

IgniterJs is javascript patching functionality for [Igniter](https://hexdocs.pm/igniter)

## Installation

IgniterJs can be added to an existing elixir project by adding it to your dependencies:

```elixir
{:igniter_js, "~> 0.1", only: [:dev, :test]}
```

## Status

We are still working on getting this ready for an initial release.

The initial codemods will be limited to specific transformations. This is not intended to
be a toolkit (yet) for writing any arbitrary transformation like `Igniter` is for `Elixir`.
We will likely provide a way to do this by the user providing rust code and using our tools
to hook it up to igniter.
