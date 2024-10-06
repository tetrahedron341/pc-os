{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    utils.url = "github:numtide/flake-utils";
    flake-compat.url = "github:edolstra/flake-compat";
  };

  outputs = { self, nixpkgs, utils, rust-overlay, flake-compat }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { overlays = [ rust-overlay.overlays.default ]; inherit system; };
        limine-override = pkgs.limine.override { enableAll=true; };
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        buildInputs = with pkgs; [ just qemu limine-override jq xorriso OVMF.fd ];
        nativeBuildInputs = with pkgs; [ rustfmt clippy rustToolchain ];
      in
      {
        devShells.default = with pkgs; mkShell {
          OVMF_PATH = "${pkgs.OVMF.fd}/FV/OVMF.fd";
          LIMINE_PREFIX = limine-override;
          inherit buildInputs nativeBuildInputs;
        };
      }
    );
}
