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

type Status = {
  connected: boolean;
  encryption: "libp2p transport encryption (test mode)" | "None";
  peerId?: string;
  connectedPeer?: string;
  keyEpoch?: number;
  rotationInterval?: number;
  mode?: "listener" | "dialer";
};

type Event = {
  t: string;
  level: "status" | "rotation" | "peer" | "info" | "error";
};

type FileTransferMetrics = {
  state: "idle" | "preparing" | "in_progress" | "completed" | "failed";
  fileName?: string;
  fileSize?: number;
  startTime?: number | null;
  endTime?: number | null;
};

type TestSummaryProps = {
  events: Event[];
  connectionHealth: ConnectionHealthData;
  status: Status;
  fileTransfer?: FileTransferMetrics;
};

export function TestSummary({
  events,
  connectionHealth,
  status,
  fileTransfer,
}: TestSummaryProps) {
  const errorCount = events.filter((e) => e.level === "error").length;

  const getFileTransferSummary = (): string => {
    if (!fileTransfer || fileTransfer.state === "idle") {
      return "N/A";
    }
    if (fileTransfer.state === "completed" && fileTransfer.endTime && fileTransfer.startTime) {
      const durationMs = fileTransfer.endTime - fileTransfer.startTime;
      const durationSec = (durationMs / 1000).toFixed(2);
      const sizeMB = fileTransfer.fileSize ? (fileTransfer.fileSize / (1024 * 1024)).toFixed(2) : "?";
      const throughput = fileTransfer.fileSize && durationMs > 0
        ? ((fileTransfer.fileSize * 8) / (durationMs / 1000) / 1000000).toFixed(2)
        : "?";
      return `${sizeMB} MB in ${durationSec}s (~${throughput} Mbps) - Completed`;
    }
    if (fileTransfer.state === "failed") {
      return "Failed";
    }
    if (fileTransfer.state === "in_progress" || fileTransfer.state === "preparing") {
      return "In progress...";
    }
    return fileTransfer.state;
  };

  const getLastRunStatus = (): string => {
    if (status.connected) {
      return "Connected (encrypted tunnel active)";
    }
    if (connectionHealth.connectionState === "connecting") {
      return "Connecting...";
    }
    if (connectionHealth.connectionState === "encrypted-tunnel-active") {
      return "Encrypted tunnel active";
    }
    if (errorCount > 0) {
      return "Failed to connect";
    }
    return "Idle";
  };

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
        Test Summary
      </h3>

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "auto 1fr",
          gap: "8px 16px",
          color: "#ddd",
        }}
      >
        <div style={{ color: "#888" }}>Last Run Status:</div>
        <div
          style={{
            color: status.connected
              ? "#4f4"
              : errorCount > 0
                ? "#f55"
                : connectionHealth.connectionState === "connecting"
                  ? "#ff8"
                  : "#888",
          }}
        >
          {getLastRunStatus()}
        </div>

        <div style={{ color: "#888" }}>Last Handshake:</div>
        <div style={{ color: "#59f" }}>
          {connectionHealth.lastHandshakeDirection
            ? `${connectionHealth.lastHandshakeDirection === "inbound" ? "Inbound (Listener)" : "Outbound (Dialer)"}${connectionHealth.lastHandshakeTime ? ` at ${connectionHealth.lastHandshakeTime}` : ""}`
            : "N/A"}
        </div>

        <div style={{ color: "#888" }}>Last Ping:</div>
        <div style={{ color: "#8f8" }}>
          {connectionHealth.lastPingLatency || "N/A"}
        </div>

        <div style={{ color: "#888" }}>Key Rotation:</div>
        <div style={{ color: "#f90" }}>
          {connectionHealth.currentEpoch !== undefined
            ? `Epoch ${connectionHealth.currentEpoch}${connectionHealth.rotationIntervalSecs !== undefined ? ` (every ${connectionHealth.rotationIntervalSecs}s)` : ""}`
            : "N/A"}
        </div>

        <div style={{ color: "#888" }}>Errors This Session:</div>
        <div style={{ color: errorCount > 0 ? "#f55" : "#8f8" }}>
          {errorCount}
        </div>

        <div style={{ color: "#888" }}>Last File Transfer:</div>
        <div
          style={{
            color:
              fileTransfer?.state === "completed"
                ? "#4f4"
                : fileTransfer?.state === "failed"
                  ? "#f55"
                  : fileTransfer?.state === "in_progress" || fileTransfer?.state === "preparing"
                    ? "#ff8"
                    : "#888",
          }}
        >
          {getFileTransferSummary()}
        </div>
      </div>
    </div>
  );
}

