# CrypRQ: Post-Quantum, Zero-Trust VPN

CrypRQ is a Rust-based, post-quantum VPN solution that implements ephemeral key rotation and utilizes the Kyber768 cryptographic algorithm for secure handshakes. This project aims to provide a secure and efficient way to establish VPN connections while ensuring that keys are regularly rotated to enhance security.

## Contact & SPDX

- © 2025 Thor Thor  
- Contact: [codethor@gmail.com](mailto:codethor@gmail.com)  
- LinkedIn: [https://www.linkedin.com/in/thor-thor0](https://www.linkedin.com/in/thor-thor0)  
- SPDX-License-Identifier: MIT

## Project Structure

The project is organized as a Rust workspace with the following modules:

- **crypto**: Contains cryptographic functions and types used throughout the project.
- **p2p**: Implements peer-to-peer networking functionalities for establishing connections between VPN nodes.
- **node**: The main application that initializes and runs the VPN node.
- **cli**: A command-line interface for interacting with the VPN node.

## Setup Instructions

To build and run the project, ensure you have the Rust toolchain installed. You can specify the toolchain version using the `rust-toolchain.toml` file.

1. **Clone the repository**:
   ```
   git clone <repository-url>
   cd cryprq
   ```

2. **Build the project**:
   ```
   cargo build --release
   ```

3. **Run the VPN node**:
   ```
   cargo run --release --bin node
   ```

4. **Use the CLI**:
   ```
   cargo run --release --bin cli -- --peer <PEER_ID>
   ```

## Docker Support

The project includes a Dockerfile for building and running the application in a containerized environment. To build the Docker image, run:

```
docker build -t cryprq-node .
```

To run the VPN node in Docker:

```
docker run -d --name vpn1 cryprq-node
```

## Testing

A script is provided to automate the testing of the VPN nodes. You can run the test script as follows:

```
bash scripts/docker_vpn_test.sh
```

This script will build the Docker image, create a network, and start two VPN nodes to test the peer connection functionality.

## Maintain SPDX headers

Run the helper whenever you add new files so every source retains the required Thor Thor SPDX metadata:

```
bash scripts/add-headers.sh
```

## License

This project is licensed under the MIT License by default (see [LICENSE](../LICENSE)). An Apache 2.0 alternative is available in [LICENSE-APACHE](../LICENSE-APACHE); see [DUAL_LICENSE.md](../DUAL_LICENSE.md) for details.