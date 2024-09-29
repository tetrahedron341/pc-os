{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixpkgs-mozilla.url = "github:mozilla/nixpkgs-mozilla";
    utils.url = "github:numtide/flake-utils";
    flake-compat.url = "github:edolstra/flake-compat";
  };

  outputs = { self, nixpkgs, utils, nixpkgs-mozilla, flake-compat }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { overlays = [ nixpkgs-mozilla.overlay ]; inherit system; };
        limine-override = pkgs.limine.override { enableAll=true; };
        rustChannel = pkgs.rustChannelOf {
          rustToolchain = ./rust-toolchain.toml;
          sha256 = "sha256-fciDir+a3mo4kzjN1at6Oo3/l+eGmwV+k/w8SX3FDA4=";
        };
        buildInputs = with pkgs; [ just qemu limine-override jq xorriso OVMF.fd ];
        nativeBuildInputs = (with rustChannel; [ (rust.override {
          extensions = [ "rust-src" ];
        }) ]) 
          ++ (with pkgs; [ rustfmt clippy ]);
      in
      {
        devShells.default = with pkgs; mkShell {
          RUST_SRC_PATH = rustChannel.rust-src;
          OVMF_PATH = "${pkgs.OVMF.fd}/FV/OVMF.fd";
          LIMINE_PREFIX = limine-override;
          inherit buildInputs nativeBuildInputs;
        };
      }
    );
}
