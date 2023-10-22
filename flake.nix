{
  description = "Precursor script image generator";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
	pkgs = import nixpkgs {
	  inherit system overlays;
	};
        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "llvm-tools-preview" ];
        };
	rustPlatform = pkgs.makeRustPlatform {
	  rustc = rust;
	  cargo = rust;
	};
      in with pkgs; {
        devShell = mkShell {
	  buildInputs = [
	    rust
            diffutils
            rust-analyzer
            cargo-nextest
            grcov
            cargo-llvm-cov
	  ];

	  shellHook = ''
	    export RUST_BACKTRACE=1
	  '';
	};

	defaultPackage = rustPlatform.buildRustPackage {
	  pname = "precursor-gen";
	  version = "0.1.0";

	  src = builtins.filterSource
	    (path: type: type != "symlink" && baseNameOf path != "target")
            ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };
	};
  });
}
