{
  description = "Tools for automatic generation of hypervisor modules and decoder automation.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        ocamlPkgs = pkgs.ocaml-ng.ocamlPackages_4_14;

        rustToolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-SJwZ8g0zF2WrKDVmHrVG3pD2RGoQeo24MEXnNx5FyuI=";
        };

        sail-0_18 = ocamlPkgs.sail.overrideAttrs (previousAttrs: {
          version = "0.18";
          src = pkgs.fetchFromGitHub {
            owner = "rems-project";
            repo = "sail";
            rev = "0.18";
            hash = "sha256-QvVK7KeAvJ/RfJXXYo6xEGEk5iOmVsZbvzW28MHRFic=";
          };
          propagatedBuildInputs = previousAttrs.propagatedBuildInputs ++ [ ocamlPkgs.menhirLib ];
        });

        ocaml-gmp = ocamlPkgs.buildDunePackage {
          pname = "gmp";
          version = "6.3.0";

          src = pkgs.fetchFromGitHub {
            owner = "mirage";
            repo = "ocaml-gmp";
            rev = "a4ea288e27f00100bb62351ad61f9d0c984e2903";
            hash = "sha256-oq9fRpgcfLoZjhDq2rq7hyjdIkG6NLOfTm/SR5ciyxU=";
          };

          nativeBuildInputs = [
            pkgs.gmp
            pkgs.m4
            pkgs.file
          ];
        };

      in
      {

        devShells.default = pkgs.mkShell {
          name = "ozora-dev-shell";

          nativeBuildInputs = [
            pkgs.pkg-config
            pkgs.openssl
            ocamlPkgs.ocaml
            ocamlPkgs.findlib
            ocamlPkgs.dune_3
            ocaml-gmp
          ];

          buildInputs = [
            sail-0_18
            pkgs.gmp
            pkgs.zlib
            pkgs.m4
            pkgs.findutils
            pkgs.z3
          ];

          packages = [
            rustToolchain
            pkgs.opam
          ];

          shellHook = ''
            export SAIL_DIR="${sail-0_18}/share/sail"
          '';
        };
      }
    );
}
