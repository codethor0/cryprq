#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

echo "=== 1.  Fix broken shellHook ======================================="
sed -i.bak '/if ; then/,/fi/d' flake.nix 2>/dev/null || true
grep -q 'shellHook.*=' flake.nix || \
  perl -i -pe 'last if /shellHook/; print "  shellHook = ''\n    echo Cry\n  '';\n" if /devShells.*=/' flake.nix

echo "=== 2.  Add --listen flag to CLI ==================================="
cat > cli/src/main.rs <<'EOF'
use clap::Parser;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "cryprq", about = "Post-Quantum VPN")]
struct Args {
    #[arg(long, help = "Peer address to connect to (multiaddr)")]
    peer: Option<String>,

    #[arg(long, help = "Address to listen on (multiaddr)")]
    listen: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.listen.is_some() == args.peer.is_some() {
        eprintln!("Error: supply exactly one of --listen or --peer");
        process::exit(1);
    }

    if let Some(addr) = args.listen {
        println!("Starting listener on {}", addr);
        node::p2p::start_listener(addr).await?;
    } else {
        println!("Dialing peer {}", args.peer.as_ref().unwrap());
        node::p2p::dial_peer(args.peer.unwrap()).await?;
    }
    Ok(())
}
EOF

echo "=== 3.  Build ======================================================="
nix develop -c cargo build --release -p cryprq

echo
echo "=== 4.  Done – run listener ======================================="
echo "    ./target/release/cryprq --listen /ip4/127.0.0.1/udp/9999/quic-v1"
