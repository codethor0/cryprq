#!/usr/bin/env bash
# fix-libp2p-map.sh  –  patch the map() calls that break the build
# Works on GNU sed (Linux) and BSD sed (macOS).
# Run from the repo root.
set -euo pipefail
FILE=p2p/src/lib.rs
if sed --version >/dev/null 2>&1; then
  SED_I=(sed -i)
else
  SED_I=(sed -i '')
fi
# Replace the two bad lines
"${SED_I[@]}" \
  -e 's/let boxed_transport = libp2p::Transport::map(transport.boxed(), .*/let boxed_transport = libp2p::Transport::map(transport.boxed(), |(peer, muxer), _point| (peer, libp2p::core::muxing::StreamMuxerBox::new(muxer)));/' \
  "$FILE"
echo "✅  Patched $FILE – build again with cargo build --release -p cryprq"
