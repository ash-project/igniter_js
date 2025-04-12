import Config

if Mix.env() == :dev do
  config :git_ops,
    mix_project: IgniterJs.MixProject,
    changelog_file: "CHANGELOG.md",
    repository_url: "https://github.com/ash-project/igniter_js",
    # Instructs the tool to manage your mix version in your `mix.exs` file
    # See below for more information
    manage_mix_version?: true,
    # Instructs the tool to manage the version in your README.md
    # Pass in `true` to use `"README.md"` or a string to customize
    manage_readme_version: [
      "README.md"
    ],
    version_tag_prefix: "v"
end

config :pythonx, :uv_init,
  pyproject_toml: """
  [project]
  name = "igniter_py"
  version = "0.4.4"
  requires-python = "==3.13.*"
  dependencies = [
    "tinycss2==1.4.0",
    "css_tools==0.1.0"
  ]
  [tool.uv.sources]
  css_tools = { path = "#{File.cwd!()}/plibs/css_tools/dist/css_tools-0.1.0-py3-none-any.whl" }
  """
