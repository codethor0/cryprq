// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

type Event = {
  t: string;
  level: "status" | "rotation" | "peer" | "info" | "error";
};

type AdvancedMetricsData = {
  totalConnections: number;
  successfulConnections: number;
  failedConnections: number;
  pingCount: number;
  pingMinMicros?: number;
  pingMaxMicros?: number;
  pingAvgMicros?: number;
  keyRotationCount: number;
  lastEpoch?: number;
  fileTransfersTotal: number;
  fileTransfersSucceeded: number;
  fileTransfersFailed: number;
  errorCount: number;
  lastErrorAt?: string;
  lastErrorMessage?: string;
};

type AdvancedMetricsProps = {
  events: Event[];
  fileTransferState: "idle" | "preparing" | "in_progress" | "completed" | "failed";
  status: {
    connected: boolean;
  };
};

export function AdvancedMetrics({
  events,
  fileTransferState,
  status,
}: AdvancedMetricsProps) {
  // Calculate metrics from events
  const metrics: AdvancedMetricsData = {
    totalConnections: 0,
    successfulConnections: 0,
    failedConnections: 0,
    pingCount: 0,
    keyRotationCount: 0,
    fileTransfersTotal: 0,
    fileTransfersSucceeded: 0,
    fileTransfersFailed: 0,
    errorCount: 0,
  };

  const pingValues: number[] = [];
  let lastError: { time: string; message: string } | null = null;

  // Parse events to calculate metrics
  for (const event of events) {
    const text = event.t || "";

    // Connection attempts
    if (text.match(/Connect initiated|connect/i)) {
      metrics.totalConnections++;
    }

    // Successful connections
    if (
      text.match(/connection_established|Connected to|Inbound connection established/i)
    ) {
      metrics.successfulConnections++;
    }

    // Failed connections
    if (
      text.match(/Connect failed|Connect error|ConnectionClosed|connection closed/i) &&
      !text.match(/Address already in use/i)
    ) {
      metrics.failedConnections++;
    }

    // Ping events
    if (text.match(/Ping event.*result:\s*Ok\(([\d.]+)(µs|ms|s)\)/i)) {
      const pingMatch = text.match(/result:\s*Ok\(([\d.]+)(µs|ms|s)\)/i);
      if (pingMatch) {
        metrics.pingCount++;
        const value = parseFloat(pingMatch[1]);
        const unit = pingMatch[2];
        // Convert to microseconds for consistent comparison
        let micros = value;
        if (unit === "ms") {
          micros = value * 1000;
        } else if (unit === "s") {
          micros = value * 1000000;
        }
        pingValues.push(micros);
      }
    }

    // Key rotation events
    if (text.match(/key_rotation.*status=success/i)) {
      metrics.keyRotationCount++;
      const epochMatch = text.match(/epoch=(\d+)/);
      if (epochMatch) {
        metrics.lastEpoch = parseInt(epochMatch[1]);
      }
    }

    // File transfer events
    if (text.match(/FILE_TRANSFER|file transfer|File transfer/i)) {
      metrics.fileTransfersTotal++;
      if (text.match(/sent successfully|sent securely|completed/i)) {
        metrics.fileTransfersSucceeded++;
      } else if (text.match(/failed|error|Error/i)) {
        metrics.fileTransfersFailed++;
      }
    }

    // Error events
    if (event.level === "error" || text.match(/\[ERROR\]/i)) {
      metrics.errorCount++;
      lastError = {
        time: new Date().toLocaleTimeString(),
        message: text.length > 100 ? text.substring(0, 100) + "..." : text,
      };
    }
  }

  // Calculate ping statistics
  if (pingValues.length > 0) {
    metrics.pingMinMicros = Math.min(...pingValues);
    metrics.pingMaxMicros = Math.max(...pingValues);
    metrics.pingAvgMicros =
      pingValues.reduce((a, b) => a + b, 0) / pingValues.length;
  }

  // Update from current state
  if (status.connected && metrics.successfulConnections === 0) {
    metrics.successfulConnections = 1;
  }

  if (fileTransferState === "completed") {
    metrics.fileTransfersSucceeded++;
  } else if (fileTransferState === "failed") {
    metrics.fileTransfersFailed++;
  }

  if (lastError) {
    metrics.lastErrorAt = lastError.time;
    metrics.lastErrorMessage = lastError.message;
  }

  const formatMicros = (micros?: number): string => {
    if (micros === undefined) return "N/A";
    if (micros < 1000) {
      return `${micros.toFixed(2)}µs`;
    } else if (micros < 1000000) {
      return `${(micros / 1000).toFixed(2)}ms`;
    } else {
      return `${(micros / 1000000).toFixed(2)}s`;
    }
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
        Advanced Metrics
      </h3>

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "auto 1fr",
          gap: "8px 16px",
          color: "#ddd",
        }}
      >
        <div style={{ color: "#888" }}>Connection Attempts:</div>
        <div style={{ color: "#59f" }}>
          {metrics.totalConnections} total ({metrics.successfulConnections}{" "}
          succeeded, {metrics.failedConnections} failed)
        </div>

        <div style={{ color: "#888" }}>Ping Latency:</div>
        <div style={{ color: "#8f8" }}>
          {metrics.pingCount > 0 ? (
            <>
              Min: {formatMicros(metrics.pingMinMicros)}, Max:{" "}
              {formatMicros(metrics.pingMaxMicros)}, Avg:{" "}
              {formatMicros(metrics.pingAvgMicros)} ({metrics.pingCount} samples)
            </>
          ) : (
            "N/A (no ping events)"
          )}
        </div>

        <div style={{ color: "#888" }}>Key Rotation:</div>
        <div style={{ color: "#f90" }}>
          {metrics.keyRotationCount} events
          {metrics.lastEpoch !== undefined
            ? ` (last epoch: ${metrics.lastEpoch})`
            : ""}
        </div>

        <div style={{ color: "#888" }}>File Transfers:</div>
        <div style={{ color: "#59f" }}>
          {metrics.fileTransfersTotal} total ({metrics.fileTransfersSucceeded}{" "}
          succeeded, {metrics.fileTransfersFailed} failed)
        </div>

        <div style={{ color: "#888" }}>Errors:</div>
        <div style={{ color: metrics.errorCount > 0 ? "#f55" : "#8f8" }}>
          {metrics.errorCount} total
          {metrics.lastErrorAt && (
            <>
              {" "}
              (last: {metrics.lastErrorAt}
              {metrics.lastErrorMessage && ` - ${metrics.lastErrorMessage}`})
            </>
          )}
        </div>
      </div>
    </div>
  );
}

