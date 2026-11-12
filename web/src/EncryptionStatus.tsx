// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

type Status = {
  connected: boolean;
  encryption: "libp2p transport encryption (test mode)" | "None";
  peerId?: string;
  connectedPeer?: string;
  keyEpoch?: number;
  rotationInterval?: number;
  mode?: "listener" | "dialer";
};

export function EncryptionStatus({ status }: { status: Status }) {
  return (
    <div
      style={{
        background: "#1a1a1a",
        border: "1px solid #333",
        borderRadius: "8px",
        padding: "16px",
        marginTop: "16px",
        fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace",
        fontSize: "12px",
      }}
    >
      <h3 style={{ marginTop: 0, marginBottom: "12px", color: "#fff" }}>
        Encryption Status
      </h3>

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "auto 1fr",
          gap: "8px 16px",
          color: "#ddd",
        }}
      >
        <div style={{ color: "#888" }}>Connection:</div>
        <div
          style={{
            color: status.connected
              ? "#4f4"
              : status.mode &&
                  status.encryption === "libp2p transport encryption (test mode)"
                ? "#ff8" // Encryption active = yellow
                : status.mode === "listener" && status.peerId
                  ? "#ff8"
                  : status.peerId && !status.connected
                    ? "#ff8"
                    : "#f55",
          }}
        >
          {status.connected
            ? "[ACTIVE] Encrypted Tunnel Active"
            : status.mode === "listener" && status.peerId
              ? "[WAITING] Listening (encryption active, waiting for peer)"
              : status.mode === "dialer" && status.peerId
                ? "[CONNECTING] Connecting (encryption active)..."
                : status.peerId
                  ? "[ESTABLISHING] Encryption Active (establishing connection...)"
                  : status.mode === "listener" &&
                      status.encryption === "libp2p transport encryption (test mode)"
                    ? "[STARTING] Starting (encryption active)..."
                    : status.mode === "dialer" &&
                        status.encryption ===
                          "libp2p transport encryption (test mode)"
                      ? "[CONNECTING] Connecting (encryption active)..."
                      : status.mode === "dialer"
                        ? "[CONNECTING] Connecting..."
                        : status.mode === "listener"
                          ? "[STARTING] Starting..."
                          : status.encryption ===
                              "libp2p transport encryption (test mode)"
                            ? "[READY] Encryption Active (ready to connect)..."
                            : "[DISCONNECTED] Disconnected"}
        </div>

        <div style={{ color: "#888" }}>Encryption:</div>
        <div style={{ color: "#59f" }}>{status.encryption}</div>

        {status.peerId && (
          <>
            <div style={{ color: "#888" }}>Local Peer ID:</div>
            <div style={{ color: "#8f8", wordBreak: "break-all" }}>
              {status.peerId}
            </div>
          </>
        )}

        {status.connectedPeer && (
          <>
            <div style={{ color: "#888" }}>Connected Peer:</div>
            <div style={{ color: "#4f4", wordBreak: "break-all" }}>
              {status.connectedPeer}
            </div>
          </>
        )}

        {status.keyEpoch && (
          <>
            <div style={{ color: "#888" }}>Key Epoch:</div>
            <div style={{ color: "#f90" }}>Epoch {status.keyEpoch}</div>
          </>
        )}

        {status.rotationInterval && (
          <>
            <div style={{ color: "#888" }}>Key Rotation:</div>
            <div style={{ color: "#f90" }}>
              Every {status.rotationInterval}s
            </div>
          </>
        )}

        {status.mode && (
          <>
            <div style={{ color: "#888" }}>Mode:</div>
            <div style={{ color: "#59f", textTransform: "capitalize" }}>
              {status.mode}
            </div>
          </>
        )}
      </div>

      <div
        style={{
          marginTop: "12px",
          padding: "8px",
          background: "#0a0a0a",
          borderRadius: "4px",
          fontSize: "11px",
          color: "#888",
          lineHeight: "1.5",
        }}
      >
        <strong style={{ color: "#fff" }}>Encryption Method:</strong> libp2p
        transport encryption (QUIC/TLS). This preview runs the legacy test-mode
        path: the CrypRQ ML-KEM + X25519 handshake is not run and peers are not
        authenticated.
        <br />
        <br />
        <strong style={{ color: "#59f" }}>Observed events:</strong>
        <br />
        {status.keyEpoch ? (
          <span style={{ color: "#4f4" }}>
            Local rotation timer fired (epoch {status.keyEpoch}); these keys do
            not protect the traffic of this preview
          </span>
        ) : (
          <span style={{ color: "#888" }}>
            Waiting for key rotation event...
          </span>
        )}
        <br />
        {status.peerId ? (
          <span style={{ color: "#4f4" }}>
            Peer ID generated (libp2p transport identity)
          </span>
        ) : (
          <span style={{ color: "#888" }}>
            Waiting for peer ID generation...
          </span>
        )}
        <br />
        <br />
        <strong style={{ color: "#59f" }}>P2P Tunnel:</strong>{" "}
        {status.encryption === "libp2p transport encryption (test mode)"
          ? "[ACTIVE]"
          : "[INACTIVE]"}{" "}
        Traffic between the two test peers is protected by the libp2p
        transport only. Authenticated hybrid tunnels are provided by the CLI
        (<code>cryprq --vpn</code> and file transfer with pinned identities),
        not by this preview.
        <br />
        <strong style={{ color: "#ff8" }}>System-Wide VPN:</strong> [NOTE]
        Requires Network Extension framework on macOS. The encrypted tunnel
        between peers is active, but routing all system/browser traffic requires
        macOS Network Extension (NEPacketTunnelProvider).
        <br />
        <br />
        See{" "}
        <code style={{ color: "#59f" }}>
          docs/SYSTEM_VPN_IMPLEMENTATION.md
        </code>{" "}
        for implementation details.
      </div>
    </div>
  );
}
