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
        pkgs = import nixpkgs { inherit system; };
        # Import nixpkgs with the rust-overlay applied.
        rust_toolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./zbs/rust-toolchain.toml;
          sha256 = "sha256-SJwZ8g0zF2WrKDVmHrVG3pD2RGoQeo24MEXnNx5FyuI=";
        };

        # List of native dependencies required for building the project.
        # Based on the requirements and opam list provided.
        nativeBuildInputs = with pkgs; [
          # For building Rust crates that link against C libraries.
          pkg-config

          # For crates that depend on OpenSSL.
          openssl
        ];

        # List of dependencies for the project itself.
        buildInputs = with pkgs; [
          # The primary requirement. This package should pull in most of the
          # necessary OCaml dependencies like lem, ott, etc.
          sail

          # System libraries identified from `conf-*` packages in opam.
          gmp
          zlib
          m4
          findutils
        ];
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "ozora";
          version = "0.1.0";

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
              rust_toolchain

              # Include OCaml and opam for manual package management if needed.
              # The `sail` package already provides a specific OCaml version.
              ocaml
              opam
            ];
        };
      }
    );
}
