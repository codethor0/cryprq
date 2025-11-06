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
        
        rustToolchain = pkgs.rust-bin.stable."1.83.0".default.override {
          targets = [ "x86_64-unknown-linux-musl" ];
        };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          cargo-zigbuild
          pkg-config
        ];

        buildInputs = with pkgs; [
          openssl
          musl
        ];

      in {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;
          
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = "${pkgs.zigpkgs.master}/bin/zig";
          
          shellHook = ''
            echo "CrypRQ development environment"
            echo "Rust version: $(rustc --version)"
            echo ""
            echo "Build reproducible musl binary:"
            echo "  cargo zigbuild --release --target x86_64-unknown-linux-musl"
            echo ""
            echo "Strip and compress:"
            echo "  strip target/x86_64-unknown-linux-musl/release/cryprq"
            echo "  upx --best --lzma target/x86_64-unknown-linux-musl/release/cryprq"
          '';
        };

        packages = rec {
          cryprq-musl = pkgs.rustPlatform.buildRustPackage {
            pname = "cryprq";
            version = "0.1.0";
            
            src = ./.;
            
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            
            nativeBuildInputs = nativeBuildInputs;
            buildInputs = buildInputs;
            
            CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
            CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static -C link-arg=-s";
            
            postInstall = ''
              strip $out/bin/cryprq
              ${pkgs.upx}/bin/upx --best --lzma $out/bin/cryprq || true
            '';
            
            meta = with pkgs.lib; {
              description = "Post-quantum VPN with 5-minute key rotation";
              homepage = "https://github.com/codethor0/cryprq";
              license = licenses.gpl3Only;
              maintainers = [];
              platforms = platforms.linux;
            };
          };
          
          default = cryprq-musl;
        };
      }
    );
}
