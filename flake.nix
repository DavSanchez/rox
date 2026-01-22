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
            crafting-interpreters-tests = pkgs.writers.writeNuBin "crafting-interpreters-tests" {
              makeWrapperArgs = [
                "--prefix"
                "PATH"
                ":"
                "${lib.makeBinPath [
                  inputs.oldDartNixpkgs.legacyPackages.${system}.dart
                  pkgs.gnumake
                  pkgs.uutils-coreutils-noprefix
                ]}"
              ];
            } (lib.readFile ./scripts/test_runner.nu);
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

              shellHook = ''
                ${config.pre-commit.installationScript}
              '';

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
