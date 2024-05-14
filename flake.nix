{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:nmattia/naersk";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = inputs:
    inputs.flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import inputs.nixpkgs { inherit system; };
        naersk-lib = inputs.naersk.lib."${system}";
      in
      rec {
        formatter = pkgs.nixpkgs-fmt;

        packages.elm-duet = naersk-lib.buildPackage {
          root = ./.;
        };
        defaultPackage = packages.elm-duet;

        devShell = pkgs.mkShell {
          packages = [
            pkgs.cargo
            pkgs.clippy
            pkgs.libiconv
            pkgs.rust-analyzer
            pkgs.rustc
            pkgs.rustfmt

            # formatters
            pkgs.typos

            # formatters
            pkgs.nodePackages.npm
            pkgs.nodejs
          ];
        };
      }
    );
}
