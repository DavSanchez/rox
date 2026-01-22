{
  description = "My implementations of Lox, a programming language from the book 'Crafting Interpreters' by Robert Nystrom, as a bytecode virtual machine";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    oldDartNixpkgs.url = "github:nixos/nixpkgs/8cad3dbe48029cb9def5cdb2409a6c80d3acfe2e"; # Dart 2.19.6
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    flake-parts.url = "github:hercules-ci/flake-parts";
    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crafting-interpreters = {
      url = "github:munificent/craftinginterpreters";
      flake = false;
    };
  };

  outputs =
    inputs@{
      flake-parts,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } (
      # top@{ config, withSystem, moduleWithSystem, ... }: # Unused for now
      _: {
        imports = with inputs; [
          git-hooks.flakeModule
          treefmt-nix.flakeModule
        ];

        systems = [
          "x86_64-linux"
          "aarch64-linux"
          "aarch64-darwin"
        ];

        perSystem =
          {
            pkgs,
            config,
            inputs',
            self',
            system,
            ...
          }:
          let
            inherit (pkgs) lib;
            crafting-interpreters-tests = pkgs.writeShellApplication {
              name = "run-crafting-interpreters-tests";

              runtimeInputs = [
                inputs.oldDartNixpkgs.legacyPackages.${system}.dart
                pkgs.gnumake
                pkgs.uutils-coreutils-noprefix
              ];

              text = ''
                # Mappings from chapter number to test suite name.
                declare -A chapter_map
                chapter_map["14"]="chap14_chunks"
                chapter_map["15"]="chap15_virtual"
                chapter_map["16"]="chap16_scanning"
                chapter_map["17"]="chap17_compiling"
                chapter_map["18"]="chap18_types"
                chapter_map["19"]="chap19_strings"
                chapter_map["20"]="chap20_hash"
                chapter_map["21"]="chap21_global"
                chapter_map["22"]="chap22_local"
                chapter_map["23"]="chap23_jumping"
                chapter_map["24"]="chap24_calls"
                chapter_map["25"]="chap25_closures"
                chapter_map["26"]="chap26_garbage"
                chapter_map["27"]="chap27_classes"
                chapter_map["28"]="chap28_methods"
                chapter_map["29"]="chap29_superclasses"
                chapter_map["30"]="chap30_optimization"

                # Ordered chapters for default execution
                ordered_chapters=(
                  "chap14_chunks"
                  "chap15_virtual"
                  "chap16_scanning"
                  "chap17_compiling"
                  "chap18_types"
                  "chap19_strings"
                  "chap20_hash"
                  "chap21_global"
                  "chap22_local"
                  "chap23_jumping"
                  "chap24_calls"
                  "chap25_closures"
                  "chap26_garbage"
                  "chap27_classes"
                  "chap28_methods"
                  "chap29_superclasses"
                  "chap30_optimization"
                )

                targets=()
                if [ "$#" -eq 0 ]; then
                  targets=("''${ordered_chapters[@]}")
                else
                  for arg in "$@"; do
                    val="''${chapter_map[$arg]:-}"
                    if [ -n "$val" ]; then
                      targets+=("$val")
                    else
                      echo "Error: Invalid chapter number provided: $arg"
                      exit 1
                    fi
                  done
                fi

                tmp_dir=$(mktemp -d)
                # Cleanup on exit
                trap 'rm -rf "$tmp_dir"' EXIT

                echo "Copying ${inputs.crafting-interpreters} to temporary directory $tmp_dir..."
                cp --no-preserve=all -r "${inputs.crafting-interpreters}/." "$tmp_dir"

                cd "$tmp_dir/tool"
                dart pub get
                cd "$tmp_dir"

                for target in "''${targets[@]}"; do
                  echo "--------------------------------------------------------------------------------"
                  echo "Running $target..."
                  echo "--------------------------------------------------------------------------------"
                  dart tool/bin/test.dart "$target" --interpreter "${lib.getExe rox}" --arguments "--$target"
                done
              '';
            };
            rustToolchain = inputs'.fenix.packages.latest.toolchain;
            craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;
            src = craneLib.cleanCargoSource ./.;
            advisory-db = inputs.advisory-db;

            # Common arguments can be set here to avoid repeating them later
            commonArgs = {
              inherit src;
              strictDeps = true;

              buildInputs = [
                # Add additional build inputs here
              ]
              ++ lib.optionals pkgs.stdenv.isDarwin [
                # Additional darwin specific inputs can be set here
                pkgs.libiconv
              ];

              # Additional environment variables can be set directly
              # MY_CUSTOM_VAR = "some value";
            };

            # Build *just* the cargo dependencies, so we can reuse
            # all of that work (e.g. via cachix) when running in CI
            cargoArtifacts = craneLib.buildDepsOnly commonArgs;

            # Build the actual crate itself, reusing the dependency
            # artifacts from above.
            rox = craneLib.buildPackage (
              commonArgs
              // {
                inherit cargoArtifacts;
              }
            );
          in
          {
            # If one wishes to add config or overlay (e.g. cross-compilation?)
            # It's possible to override the `pkgs` argument passed into `perSystem`.
            # Though perhaps `withSystem` suits that use case better.
            # _module.args.pkgs = import inputs.nixpkgs {
            #   inherit system;
            #   overlays = [
            #     inputs.fenix.overlays.default
            #   ];
            # };
            checks = {
              inherit rox;
              # Run clippy (and deny all warnings) on the crate source,
              # again, reusing the dependency artifacts from above.
              #
              # Note that this is done as a separate derivation so that
              # we can block the CI if there are issues here, but not
              # prevent downstream consumers from building our crate by itself.
              rox-clippy = craneLib.cargoClippy (
                commonArgs
                // {
                  inherit cargoArtifacts;
                  cargoClippyExtraArgs = "--all-targets -- --deny warnings";
                }
              );

              rox-doc = craneLib.cargoDoc (
                commonArgs
                // {
                  inherit cargoArtifacts;
                  # This can be commented out or tweaked as necessary, e.g. set to
                  # `--deny rustdoc::broken-intra-doc-links` to only enforce that lint
                  env.RUSTDOCFLAGS = "--deny warnings";
                }
              );

              # Check formatting
              rox-fmt = craneLib.cargoFmt {
                inherit src;
              };

              rox-toml-fmt = craneLib.taploFmt {
                src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
                # taplo arguments can be further customized below as needed
                taploExtraArgs = "--config ./taplo.toml";
              };

              # Audit dependencies
              rox-audit = craneLib.cargoAudit {
                inherit src advisory-db;
              };

              # Audit licenses
              rox-deny = craneLib.cargoDeny {
                inherit src;
              };

              # Run tests with cargo-nextest
              # Consider setting `doCheck = false` on `my-crate` if you do not want
              # the tests to run twice
              rox-nextest = craneLib.cargoNextest (
                commonArgs
                // {
                  inherit cargoArtifacts;
                  partitions = 1;
                  partitionType = "count";
                  cargoNextestPartitionsExtraArgs = "--no-tests=pass";
                }
              );

            };

            packages = {
              default = rox;
            };

            apps = {
              default = {
                type = "app";
                program = lib.getExe rox;
              };
              rox-crafting-interpreters-tests = {
                type = "app";
                program = lib.getExe crafting-interpreters-tests;
                meta.description = "Run the Crafting Interpreters test suite. Usage: nix run .#tests -- [chapters...]";
              };
            };

            devShells.default = craneLib.devShell {
              # Inherit inputs from checks.
              inherit (self') checks;

              # Additional dev-shell environment variables can be set directly
              # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

              # Extra inputs can be added here; cargo and rustc are provided by default.
              packages = [
                # pkgs.ripgrep
              ];
            };

            pre-commit = {
              settings = {
                hooks = {
                  # Formatters
                  treefmt = {
                    enable = true;
                    packageOverrides.treefmt = config.treefmt.build.wrapper;
                  };
                  actionlint.enable = true;
                  convco.enable = true;
                  gitlint.enable = true;
                  check-merge-conflicts.enable = true;
                  # `git-hooks` defines its own entries for
                  # `clippy`, `rustfmt` and `taplo`, but we
                  # also defined them with `crane` above, so
                  # we can just use them here.
                  clippy.enable = false;
                  crane-clippy = {
                    name = "crane clippy";
                    package = self'.checks.rox-clippy;
                    entry = "";
                  };
                  rustfmt.enable = false;
                  crane-fmt = {
                    name = "crane rustfmt";
                    package = self'.checks.rox-fmt;
                    entry = "";
                  };
                  taplo.enable = false;
                  crane-toml-fmt = {
                    name = "crane taplo fmt";
                    package = self'.checks.rox-toml-fmt;
                    entry = "";
                  };
                };
              };
            };

            treefmt = {
              programs = {
                nixfmt.enable = true;
                rustfmt = {
                  enable = true;
                  package = rustToolchain;
                };
                taplo = {
                  enable = true;
                  package = pkgs.taplo;
                };
              };
            };
          };
      }
    );
}
