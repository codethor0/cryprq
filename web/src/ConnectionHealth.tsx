// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

type ConnectionHealthData = {
  connectionState: "idle" | "connecting" | "encrypted-tunnel-active";
  lastHandshakeDirection?: "inbound" | "outbound";
  lastHandshakeTime?: string;
  lastPingLatency?: string;
  currentEpoch?: number;
  rotationIntervalSecs?: number;
};

export function ConnectionHealth({ health }: { health: ConnectionHealthData }) {
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
        Connection Health
      </h3>

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "auto 1fr",
          gap: "8px 16px",
          color: "#ddd",
        }}
      >
        <div style={{ color: "#888" }}>Connection State:</div>
        <div
          style={{
            color:
              health.connectionState === "encrypted-tunnel-active"
                ? "#4f4"
                : health.connectionState === "connecting"
                  ? "#ff8"
                  : "#888",
          }}
        >
          {health.connectionState === "encrypted-tunnel-active"
            ? "Encrypted tunnel active"
            : health.connectionState === "connecting"
              ? "Connecting..."
              : "Idle"}
        </div>

        <div style={{ color: "#888" }}>Last Handshake:</div>
        <div style={{ color: "#59f" }}>
          {health.lastHandshakeDirection
            ? `${health.lastHandshakeDirection === "inbound" ? "Inbound (Listener)" : "Outbound (Dialer)"}${health.lastHandshakeTime ? ` at ${health.lastHandshakeTime}` : ""}`
            : "N/A"}
        </div>

        <div style={{ color: "#888" }}>Last Ping Latency:</div>
        <div style={{ color: "#8f8" }}>
          {health.lastPingLatency || "N/A"}
        </div>

        <div style={{ color: "#888" }}>Current Epoch:</div>
        <div style={{ color: "#f90" }}>
          {health.currentEpoch !== undefined ? `Epoch ${health.currentEpoch}` : "N/A"}
        </div>

        <div style={{ color: "#888" }}>Rotation Interval:</div>
        <div style={{ color: "#f90" }}>
          {health.rotationIntervalSecs !== undefined
            ? `Every ${health.rotationIntervalSecs}s`
            : "N/A"}
        </div>
      </div>
    </div>
  );
}

