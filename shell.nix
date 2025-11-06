{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    pkg-config
    openssl
  ];
  
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  
  shellHook = ''
    export CARGO_HOME=$PWD/.cargo
    export RUSTUP_HOME=$PWD/.rustup
    echo "CrypRQ reproducible build environment"
    echo "Rust version: $(rustc --version)"
    echo "Run: cargo build --release --locked"
  '';
}
