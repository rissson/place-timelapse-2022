{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    futils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, futils } @ inputs:
    let
      inherit (nixpkgs) lib;
      inherit (lib) recursiveUpdate;
      inherit (futils.lib) eachDefaultSystem defaultSystems;

      nixpkgsFor = lib.genAttrs defaultSystems (system: import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlay
        ];
      });
    in
    (eachDefaultSystem (system:
      let
        pkgs = nixpkgsFor.${system};
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            wasm-bindgen-cli wasm-pack trunk

            (rust-bin.fromRustupToolchainFile ./rust-toolchain)
            cargo-audit
            cargo-edit
            cargo-expand
            cargo-flamegraph
            cargo-sort
            cargo-tarpaulin
            git
            pre-commit
          ];

          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };
      }
    ));
}
