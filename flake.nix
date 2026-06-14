{
  description = "emunes: an NES emulator written in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
    git-hooks,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [rust-overlay.overlays.default];
      };

      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = ["rust-src" "rust-analyzer"];
      };

      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

      src = craneLib.cleanCargoSource ./.;
      commonArgs = {
        inherit src;
        strictDeps = true;
      };
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      emunes = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
        });

      pre-commit-check = git-hooks.lib.${system}.run {
        src = ./.;
        hooks.rustfmt = {
          enable = true;
          packageOverrides = {
            cargo = rustToolchain;
            rustfmt = rustToolchain;
          };
        };
      };
    in {
      checks = {
        inherit emunes;

        emunes-test = craneLib.cargoTest (commonArgs
          // {
            inherit cargoArtifacts;
          });

        emunes-fmt = craneLib.cargoFmt {inherit src;};

        inherit pre-commit-check;
      };

      packages.default = emunes;

      devShells.default = craneLib.devShell {
        inherit (pre-commit-check) shellHook;
        inputsFrom = [emunes];
        packages = [rustToolchain];
      };
    });
}
