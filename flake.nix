{
  description = "Devshells for maturin development";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};
      in {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            (
              python311.withPackages
              (ps:
                with ps; [
                  pip
                  virtualenv
                  maturin
                ])
            )
            cargo
            rustc
            rustfmt
            pre-commit
            rustPackages.clippy
          ];
        };
      }
    );
}
