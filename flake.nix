{
  description = "Tools for automatic generation of hypervisor modules and decoder automation.";

  # Define the inputs for this flake, i.e., its dependencies.
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  # Define the outputs of this flake.
  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
    }:
    # Use flake-utils to provide standard outputs for common systems.
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        # Import nixpkgs for the given system.
        pkgs = import nixpkgs {
          inherit system;
        };

        # Select the OCaml 4.14 package set to match the project's requirements.
        # This ensures compatibility with sail version 0.18.
        ocamlPkgs = pkgs.ocaml-ng.ocamlPackages_4_14;

        # Specify the Rust toolchain using fenix and a toolchain file.
        rustToolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-SJwZ8g0zF2WrKDVmHrVG3pD2RGoQeo24MEXnNx5FyuI=";
        };

        # Override the sail package to fetch and build version 0.18 from GitHub.
        # This is built using the specified OCaml 4.14 compiler.
        sail-0_18 = ocamlPkgs.sail.overrideAttrs (previousAttrs: {
          version = "0.18";

          src = pkgs.fetchFromGitHub {
            owner = "rems-project";
            repo = "sail";
            rev = "0.18";
            hash = "sha256-QvVK7KeAvJ/RfJXXYo6xEGEk5iOmVsZbvzW28MHRFic=";
          };

          # Ensure menhirLib is included, as seen in the sail-riscv example.
          propagatedBuildInputs = previousAttrs.propagatedBuildInputs ++ [ ocamlPkgs.menhirLib ];
        });

        # List of native dependencies required for building the project.
        nativeBuildInputs = [
          pkgs.pkg-config
          pkgs.openssl
        ];

        # List of dependencies for the project itself.
        buildInputs = [
          # Use our custom-built sail package.
          sail-0_18

          # Add the OCaml binding for the GMP library, which dune is looking for.
          ocamlPkgs.zarith

          # System libraries and tools required by sail.
          pkgs.gmp
          pkgs.zlib
          pkgs.m4
          pkgs.findutils
          pkgs.z3 # z3 is often a runtime dependency for sail.
        ];
      in
      {
        # The default package built by `nix build`.
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "ozora";
          version = "0.1.0"; # Set a placeholder version.

          # The source code is the current directory.
          src = ./.;

          # Specify the lock file for reproducible builds.
          cargoLock.lockFile = ./Cargo.lock;

          # Dependencies needed at build time on the host system.
          inherit nativeBuildInputs;

          # Dependencies needed by the package itself.
          inherit buildInputs;
        };

        # The development shell environment provided by `nix develop`.
        devShells.default = pkgs.mkShell {
          name = "ozora-dev-shell";

          # Packages available in the development environment.
          packages =
            nativeBuildInputs
            ++ buildInputs
            ++ [
              # The Rust toolchain (cargo, rustc, etc.).
              rustToolchain

              # Include OCaml and opam for manual package management if needed.
              # We explicitly use the 4.14 versions.
              ocamlPkgs.ocaml
              pkgs.opam

              # Add OCaml development tools to the shell PATH.
              ocamlPkgs.findlib # Provides ocamlfind
              ocamlPkgs.dune_3 # The OCaml build system
            ];
        };
      }
    );
}
