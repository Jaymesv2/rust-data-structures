{
  description = "Simple Rust Data Structures";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };
      #rustVersion = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      rustVersion = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.complete); # or `toolchain.minimal`
      rustPlatform = pkgs.makeRustPlatform {
        cargo = rustVersion;
	rustc = rustVersion;
      };
    in {
      devShell = pkgs.mkShell {
        buildInputs =
          [ 
      pkgs.clang
	    pkgs.ograc 
      pkgs.mold
	    (rustVersion.override { extensions = [ "rust-src" ]; }) 
	  ];
      };
    });
}
