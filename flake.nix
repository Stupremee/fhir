{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust-bin.stable.latest.default
            rustPlatform.bindgenHook
            cargo-pgrx
            cargo-shear

            # build dependencies
            bison
            flex
            pkg-config
            readline
            zlib
            libxml2
            openssl
            icu
            icu.dev
          ];

          PG_VERSION = "pg18";
        };
      }
    );
}
