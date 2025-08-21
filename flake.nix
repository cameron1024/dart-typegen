{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain (
          p:
            p.rust-bin.stable.latest.default.override {
              targets = ["wasm32-wasip1"];
            }
        );

        mkFilter = ext: path: _type: builtins.match ".*${ext}" path != null;

        dart-typegen = craneLib.buildPackage {
          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              false
              || (mkFilter "dart" path type)
              || (mkFilter "kdl" path type)
              || (mkFilter "snap" path type) # cargo-insta snapshots - needed for tests
              || (craneLib.filterCargoSources path type);
          };
          strictDeps = true;
          doCheck = false;
        };
      in {
        checks = {
          inherit dart-typegen;
        };

        packages.default = dart-typegen;

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = with pkgs; [
            cargo-insta
            mdbook
            mdbook-alerts
          ];
        };
      }
    );
}
