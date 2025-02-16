defmodule IgniterJs.MixProject do
  use Mix.Project
  @version "0.4.3"
  @source_url "https://github.com/ash-project/igniter_js"

  @description """
  Javascript codemods, powered by a high-performance Rust parser integrated via NIFs
  """

  def project do
    [
      app: :igniter_js,
      version: @version,
      elixir: "~> 1.14",
      package: package(),
      aliases: aliases(),
      elixirc_paths: elixirc_paths(Mix.env()),
      start_permanent: Mix.env() == :prod,
      docs: docs(),
      deps: deps(),
      description: @description,
      package: package(),
      source_url: @source_url,
      homepage_url: @source_url
    ]
  end

  defp package() do
    [
      name: :igniter_js,
      files: ~w[
          lib
          native/igniter_js/src
          native/igniter_js/Cargo.*
          native/igniter_js/README.md
          native/igniter_js/.cargo
          checksum-*.exs
          .formatter.exs
          mix.exs
          LICENSE
          README*
        ],
      maintainers: ["Zach Daniel", "Shahryar Tavakkoli"],
      licenses: ["MIT"],
      links: %{
        "GitHub" => @source_url,
        "Discord" => "https://discord.gg/HTHRaaVPUc",
        "Website" => "https://ash-hq.org",
        "Forum" => "https://elixirforum.com/c/ash-framework-forum/",
        "Changelog" => "#{@source_url}/blob/main/CHANGELOG.md"
      }
    ]
  end

  defp elixirc_paths(:test), do: ["lib", "test/support"]
  defp elixirc_paths(_), do: ["lib"]

  defp docs do
    [
      main: "readme",
      source_ref: "v#{@version}",
      logo: "logos/igniter-logo.png",
      extra_section: "GUIDES",
      before_closing_head_tag: fn type ->
        if type == :html do
          """
          <script>
            if (location.hostname === "hexdocs.pm") {
              var script = document.createElement("script");
              script.src = "https://plausible.io/js/script.js";
              script.setAttribute("defer", "defer")
              script.setAttribute("data-domain", "ashhexdocs")
              document.head.appendChild(script);
            }
          </script>
          """
        end
      end,
      extras: [
        {"README.md", title: "Home"},
        "CHANGELOG.md"
      ],
      groups_for_extras: [
        Tutorials: ~r'documentation/tutorials',
        "How To": ~r'documentation/how_to',
        Topics: ~r'documentation/topics',
        DSLs: ~r'documentation/dsls',
        "About IgniterJs": [
          "CHANGELOG.md"
        ]
      ]
      # groups_for_modules: [
      # ]
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger],
      mod: {IgniterJs.Application, []}
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      {:rustler, "~> 0.36.1", optional: true},
      {:rustler_precompiled, "~> 0.8"},
      {:ex_doc, "~> 0.35", only: [:dev, :test], runtime: false},
      {:ex_check, "~> 0.12", only: [:dev, :test]},
      {:credo, ">= 0.0.0", only: [:dev, :test], runtime: false},
      {:dialyxir, ">= 0.0.0", only: [:dev, :test], runtime: false},
      {:sobelow, ">= 0.0.0", only: [:dev, :test], runtime: false},
      {:git_ops, "~> 2.5", only: [:dev, :test]},
      {:mix_audit, ">= 0.0.0", only: [:dev, :test], runtime: false}
    ]
  end

  defp aliases do
    [
      sobelow: "sobelow --skip",
      credo: "credo --strict"
    ]
  end
end
