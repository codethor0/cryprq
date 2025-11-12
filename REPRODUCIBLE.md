# Reproducible Builds

CrypRQ supports reproducible builds for Linux (musl), macOS, Docker, and Nix environments.

## Overview

Reproducible builds ensure that the same source code always produces bit-for-bit identical binaries, enabling verification of release artifacts and supply-chain security.

## Build Methods

### Linux (musl)

```bash
## Build statically linked binary with musl
bash scripts/build-musl.sh

## Binary will be in: target/x86_64-unknown-linux-musl/release/cryprq
```

### macOS

```bash
## Build macOS binary
bash scripts/build-macos.sh

## Binary will be in: target/release/cryprq
```

### Docker

```bash
## Build Docker image
docker build -t cryprq-node:latest -f Dockerfile .

## Verify image
docker run --rm cryprq-node:latest --help
```

### Nix

```bash
## Build with Nix
nix build

## Run binary
./result/bin/cryprq --help
```

## Validation

### Automated Validation

```bash
## Run reproducible build validation script
bash scripts/repro-build.sh

## This performs two clean builds and compares hashes
```

### Manual Validation

1. **Build 1**: Clean build, capture binary hash
2. **Build 2**: Clean build again, capture binary hash
3. **Compare**: Hashes should match for true reproducibility

```bash
## Build 1
cargo clean
cargo build --release -p cryprq
shasum -a 256 target/release/cryprq > build1.sha256

## Build 2
cargo clean
cargo build --release -p cryprq
shasum -a 256 target/release/cryprq > build2.sha256

## Compare
diff build1.sha256 build2.sha256
```

## Deterministic Build Settings

The project uses the following settings for reproducible builds:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

## Known Limitations

1. **Timestamps**: Build timestamps may differ between builds
2. **Build Paths**: Absolute paths in debug info may differ
3. **Rust Version**: Must use exact same Rust toolchain version (1.83.0)
4. **Dependencies**: Cargo.lock must be committed and unchanged

## Docker Reproducibility

Docker builds are more reproducible due to:
- Fixed base images
- Isolated build environment
- Deterministic dependency resolution

```bash
## Build with Docker
docker build -t cryprq-node:latest -f Dockerfile .

## Verify reproducibility
docker build -t cryprq-node:v2 -f Dockerfile .
docker diff cryprq-node:latest cryprq-node:v2
```

## Nix Reproducibility

Nix provides the highest level of reproducibility:
- Pure builds (no network access)
- Deterministic dependency resolution
- Isolated build environments

```bash
## Build with Nix
nix build

## Verify
nix-build --check
```

## Release Artifacts

Release bundles include:
- Binary checksums (SHA256)
- SBOM (Software Bill of Materials)
- Build logs
- Reproducibility validation reports

See `scripts/release.sh` for the complete release pipeline.

## References

- [Reproducible Builds Project](https://reproducible-builds.org/)
- [Rust Reproducible Builds RFC](https://github.com/rust-lang/rfcs/blob/master/text/3328-reproducible-builds.md)
- [Nix Manual](https://nixos.org/manual/nix/stable/)
