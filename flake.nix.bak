{
  description = "CrypRQ - Post-quantum VPN with 5-minute key rotation";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        isLinux = pkgs.stdenv.isLinux;

        rustToolchain = if isLinux then
          pkgs.rust-bin.stable."1.83.0".default.override {
            targets = [ "x86_64-unknown-linux-musl" ];
          }
        else
          pkgs.rust-bin.stable."1.83.0".default;

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ] ++ (if isLinux then [ cargo-zigbuild ] else []);

        buildInputs = with pkgs; [
          openssl
        ] ++ (if isLinux then [ musl ] else []);

      in {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;
          shellHook = ''
            echo "CrypRQ development environment"
            echo "Rust version: $(rustc --version)"
            echo ""
            if ${toString isLinux}; then
              echo "Build reproducible musl binary:"
              echo "  cargo zigbuild --release --target x86_64-unknown-linux-musl"
              echo ""
              echo "Strip and compress:"
              echo "  strip target/x86_64-unknown-linux-musl/release/cryprq"
              echo "  upx --best --lzma target/x86_64-unknown-linux-musl/release/cryprq"
            else
              echo "Build native macOS binary:"
              echo "  cargo build --release"
            fi
          '';
        };

        packages = rec {
          cryprq = pkgs.rustPlatform.buildRustPackage {
            pname = "cryprq";
            version = "0.1.0";
            src = ./.;
            cargoLock = { lockFile = ./Cargo.lock; };
            nativeBuildInputs = nativeBuildInputs;
            buildInputs = buildInputs;
            CARGO_BUILD_TARGET = if isLinux then "x86_64-unknown-linux-musl" else null;
            CARGO_BUILD_RUSTFLAGS = if isLinux then "-C target-feature=+crt-static -C link-arg=-s" else null;
            postInstall = ''
              strip $out/bin/cryprq || true
              ${pkgs.upx}/bin/upx --best --lzma $out/bin/cryprq || true
            '';
            meta = {
              description = "Post-quantum VPN with 5-minute key rotation";
              homepage = "https://github.com/codethor0/cryprq";
              license = pkgs.lib.licenses.gpl3Only;
              maintainers = [];
              platforms = pkgs.lib.platforms.all;
            };
          };
          default = cryprq;
        };
      }
    );
}
