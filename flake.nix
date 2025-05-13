{
  description = "Devshells for maturin development";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    hooks.url = "github:cachix/git-hooks.nix";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    hooks,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};
      in {
        checks = {
          pre-commit = hooks.lib.${system}.run {
            src = ./.;

            settings = {
              rust.check.cargoDeps = pkgs.rustPackages.rustPlatform.importCargoLock {
                lockFile = ./Cargo.lock;
                allowBuiltinFetchGit = true;
              };

              clippy = {
                denyWarnings = true;
                allFeatures = true;
              };
            };

            hooks = {
              commitizen.enable = true;
              black.enable = true;
              isort.enable = true;

              # cargo-check.enable = true;
              rustfmt.enable = true;
              clippy = {
                enable = true;
                packageOverrides.cargo = pkgs.cargo;
                packageOverrides.clippy = pkgs.rustPackages.clippy;
              };
            };
          };
        };
        devShells.default = pkgs.mkShell {
          name = "cosutils";

          inherit (self.checks.${system}.pre-commit) shellHook;
          packages = with pkgs;
            [
              (
                python311.withPackages
                (ps:
                  with ps; [
                    pip
                    virtualenv
                  ])
              )
              maturin
              cargo
              rustc
              rustfmt
              pre-commit
              rustPackages.clippy
            ]
            ++ self.checks.${system}.pre-commit.enabledPackages;
        };
      }
    );
}
