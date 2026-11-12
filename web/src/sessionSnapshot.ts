// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// License: MIT (see LICENSE file for details)

import { classifyLogEvents, type DiagnosticSummary } from "./diagnostics";

export type SessionSnapshot = {
  // Configuration
  mode: "listener" | "dialer";
  port: number | null;
  peerAddress: string | null;
  vpnEnabled: boolean;

  // Connection state
  connectionState: "idle" | "connecting" | "encrypted-tunnel-active";
  lastHandshakeDirection?: "inbound" | "outbound";
  lastHandshakeTime?: string;
  lastPingLatency?: string;
  currentEpoch?: number;
  rotationIntervalSecs?: number;

  // Advanced metrics
  connectionAttemptsTotal: number;
  connectionAttemptsSucceeded: number;
  connectionAttemptsFailed: number;
  pingCount: number;
  pingMinMicros?: number;
  pingMaxMicros?: number;
  pingAvgMicros?: number;
  keyRotationCount: number;
  errorCount: number;
  lastErrorAt?: string;
  lastErrorMessage?: string;

  // File transfer
  fileTransferState: "idle" | "preparing" | "in_progress" | "completed" | "failed";
  fileTransferFileName?: string;
  fileTransferSizeBytes?: number;
  fileTransferStartTime?: number | null;
  fileTransferEndTime?: number | null;

  // CLI snippet
  cliSnippet: string;

  // Environment
  environmentUserAgent: string;

  // Diagnostics
  diagnostics?: DiagnosticSummary;
};

type BuildSnapshotInput = {
  mode: "listener" | "dialer";
  port: number;
  peer: string;
  vpnMode: boolean;
  connectionHealth: {
    connectionState: "idle" | "connecting" | "encrypted-tunnel-active";
    lastHandshakeDirection?: "inbound" | "outbound";
    lastHandshakeTime?: string;
    lastPingLatency?: string;
    currentEpoch?: number;
    rotationIntervalSecs?: number;
  };
  events: Array<{ t: string; level: "status" | "rotation" | "peer" | "info" | "error" }>;
  status: {
    connected: boolean;
  };
  fileTransferState: "idle" | "preparing" | "in_progress" | "completed" | "failed";
  fileTransferFileName: string;
  fileTransferSize: number;
  fileTransferStartTime: number | null;
  fileTransferEndTime: number | null;
};

export function buildSessionSnapshot(input: BuildSnapshotInput): SessionSnapshot {
  const { mode, port, peer, vpnMode, connectionHealth, events, status, fileTransferState, fileTransferFileName, fileTransferSize, fileTransferStartTime, fileTransferEndTime } = input;

  // Generate CLI snippet (same logic as CLISnippet component)
  const generateCLISnippet = (): string => {
    if (mode === "listener") {
      const base = `cryprq --listen /ip4/0.0.0.0/udp/${port}/quic-v1`;
      return vpnMode ? `${base} --vpn` : base;
    } else {
      if (!peer || peer.trim() === "") {
        return `cryprq --peer <PEER_ADDRESS>${vpnMode ? " --vpn" : ""}`;
      }
      const base = `cryprq --peer ${peer}`;
      return vpnMode ? `${base} --vpn` : base;
    }
  };

  // Calculate metrics from events (same logic as AdvancedMetrics)
  let totalConnections = 0;
  let successfulConnections = 0;
  let failedConnections = 0;
  let pingCount = 0;
  const pingValues: number[] = [];
  let keyRotationCount = 0;
  let errorCount = 0;
  let lastError: { time: string; message: string } | null = null;

  for (const event of events) {
    const text = event.t || "";

    if (text.match(/Connect initiated|connect/i)) {
      totalConnections++;
    }

    if (text.match(/connection_established|Connected to|Inbound connection established/i)) {
      successfulConnections++;
    }

    if (text.match(/Connect failed|Connect error|ConnectionClosed|connection closed/i) && !text.match(/Address already in use/i)) {
      failedConnections++;
    }

    if (text.match(/Ping event.*result:\s*Ok\(([\d.]+)(µs|ms|s)\)/i)) {
      const pingMatch = text.match(/result:\s*Ok\(([\d.]+)(µs|ms|s)\)/i);
      if (pingMatch) {
        pingCount++;
        const value = parseFloat(pingMatch[1]);
        const unit = pingMatch[2];
        let micros = value;
        if (unit === "ms") {
          micros = value * 1000;
        } else if (unit === "s") {
          micros = value * 1000000;
        }
        pingValues.push(micros);
      }
    }

    if (text.match(/key_rotation.*status=success/i)) {
      keyRotationCount++;
    }

    if (event.level === "error" || text.match(/\[ERROR\]/i)) {
      errorCount++;
      lastError = {
        time: new Date().toLocaleTimeString(),
        message: text.length > 100 ? text.substring(0, 100) + "..." : text,
      };
    }
  }

  // Update from current state
  if (status.connected && successfulConnections === 0) {
    successfulConnections = 1;
  }

  // Calculate ping statistics
  let pingMinMicros: number | undefined;
  let pingMaxMicros: number | undefined;
  let pingAvgMicros: number | undefined;
  if (pingValues.length > 0) {
    pingMinMicros = Math.min(...pingValues);
    pingMaxMicros = Math.max(...pingValues);
    pingAvgMicros = pingValues.reduce((a, b) => a + b, 0) / pingValues.length;
  }

  // Get user agent
  const userAgent = typeof navigator !== "undefined" && navigator.userAgent ? navigator.userAgent : "unknown";

  // Classify log events for diagnostics
  const diagnostics = (() => {
    try {
      const rawEvents = events.map((e) => ({ raw: e.t || "", t: e.t || "" }));
      return classifyLogEvents(rawEvents);
    } catch {
      return undefined;
    }
  })();

  return {
    mode,
    port,
    peerAddress: peer || null,
    vpnEnabled: vpnMode,
    connectionState: connectionHealth.connectionState,
    lastHandshakeDirection: connectionHealth.lastHandshakeDirection,
    lastHandshakeTime: connectionHealth.lastHandshakeTime,
    lastPingLatency: connectionHealth.lastPingLatency,
    currentEpoch: connectionHealth.currentEpoch,
    rotationIntervalSecs: connectionHealth.rotationIntervalSecs,
    connectionAttemptsTotal: totalConnections,
    connectionAttemptsSucceeded: successfulConnections,
    connectionAttemptsFailed: failedConnections,
    pingCount,
    pingMinMicros,
    pingMaxMicros,
    pingAvgMicros,
    keyRotationCount,
    errorCount,
    lastErrorAt: lastError?.time,
    lastErrorMessage: lastError?.message,
    fileTransferState,
    fileTransferFileName: fileTransferFileName || undefined,
    fileTransferSizeBytes: fileTransferSize > 0 ? fileTransferSize : undefined,
    fileTransferStartTime: fileTransferStartTime || undefined,
    fileTransferEndTime: fileTransferEndTime || undefined,
    cliSnippet: generateCLISnippet(),
    environmentUserAgent: userAgent,
    diagnostics,
  };
}

export interface SessionEvent {
  timestamp: string;
  kind: string;
  message?: string;
  raw?: string;
}

export interface SessionTimeline {
  snapshot: SessionSnapshot;
  events: SessionEvent[];
  generatedAt: string;
  version: string;
}

export function buildSessionTimeline(
  snapshot: SessionSnapshot,
  rawEvents: Array<{ t: string; level: "status" | "rotation" | "peer" | "info" | "error" }>
): SessionTimeline {
  const events: SessionEvent[] = [];
  const now = new Date().toISOString();

  for (const event of rawEvents) {
    const text = event.t || "";
    
    // Infer kind from level and content
    let kind = "unknown";
    if (event.level === "status") {
      kind = "status";
    } else if (event.level === "rotation") {
      kind = "rotation";
    } else if (event.level === "peer") {
      kind = "peer";
    } else if (event.level === "error") {
      kind = "error";
    } else if (event.level === "info") {
      kind = "info";
    }
    
    // Try to extract timestamp from log line (best effort)
    let timestamp = now;
    const timestampMatch = text.match(/(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2})/);
    if (timestampMatch) {
      timestamp = timestampMatch[1].replace(" ", "T") + "Z";
    }
    
    // Extract message (first 200 chars of text)
    const message = text.length > 200 ? text.substring(0, 200) + "..." : text;
    
    events.push({
      timestamp,
      kind,
      message: message || undefined,
      raw: text || undefined,
    });
  }

  return {
    snapshot,
    events,
    generatedAt: now,
    version: "1",
  };
}

export interface ScenarioPlaybookContext {
  snapshot: SessionSnapshot;
  timeline?: SessionTimeline;
  label?: string;
  sourceSessionId?: string;
}

export function buildScenarioPlaybookMarkdown(ctx: ScenarioPlaybookContext): string {
  const { snapshot, timeline, label, sourceSessionId } = ctx;
  const lines: string[] = [];

  // Title
  const playbookLabel = label || `Session ${new Date().toLocaleString()}`;
  lines.push(`# Scenario Playbook: ${playbookLabel}`);
  lines.push("");

  // Environment & Configuration
  lines.push("## Environment & Configuration");
  lines.push("");
  lines.push("### Connection Mode");
  lines.push(`- **Mode:** ${snapshot.mode || "N/A"}`);
  lines.push(`- **Port:** ${snapshot.port ?? "N/A"}`);
  lines.push(`- **Peer Address:** ${snapshot.peerAddress || "N/A"}`);
  lines.push(`- **VPN Enabled:** ${snapshot.vpnEnabled ? "Yes" : "No"}`);
  lines.push("");

  lines.push("### CLI Command");
  lines.push("```bash");
  lines.push(snapshot.cliSnippet || "# <not recorded>");
  lines.push("```");
  lines.push("");

  lines.push("### Environment");
  lines.push(`- **User Agent:** ${snapshot.environmentUserAgent || "<not recorded>"}`);
  lines.push("");

  // Steps to Reproduce
  lines.push("## Steps to Reproduce");
  lines.push("");
  lines.push("1. Open CrypRQ Web Tester (http://localhost:8787)");
  lines.push(`2. Set mode to **${snapshot.mode === "listener" ? "Listener" : "Dialer"}**`);
  if (snapshot.port) {
    lines.push(`3. Set port to **${snapshot.port}**`);
  }
  if (snapshot.mode === "dialer" && snapshot.peerAddress) {
    lines.push(`4. Set peer address to **${snapshot.peerAddress}**`);
  }
  if (snapshot.vpnEnabled) {
    lines.push("5. Enable VPN mode toggle");
  }
  lines.push("6. Click **Connect**");
  if (snapshot.fileTransferState === "completed" || snapshot.fileTransferState === "in_progress") {
    lines.push("7. Send a file via the file transfer interface");
  }
  lines.push("8. Monitor connection status, logs, and metrics");
  lines.push("");

  // Expected Signals
  lines.push("## Expected Signals");
  lines.push("");
  lines.push("### Connection State");
  lines.push(`- **Final State:** ${snapshot.connectionState || "<not recorded>"}`);
  if (snapshot.lastHandshakeTime) {
    lines.push(`- **Last Handshake:** ${snapshot.lastHandshakeTime} (${snapshot.lastHandshakeDirection || "unknown"} direction)`);
  }
  if (snapshot.currentEpoch) {
    lines.push(`- **Key Rotation Epoch:** ${snapshot.currentEpoch}`);
  }
  if (snapshot.rotationIntervalSecs) {
    lines.push(`- **Rotation Interval:** ${snapshot.rotationIntervalSecs} seconds`);
  }
  lines.push("");

  lines.push("### Connection Metrics");
  lines.push(`- **Total Connection Attempts:** ${snapshot.connectionAttemptsTotal || 0}`);
  lines.push(`- **Successful Connections:** ${snapshot.connectionAttemptsSucceeded || 0}`);
  lines.push(`- **Failed Connections:** ${snapshot.connectionAttemptsFailed || 0}`);
  if (snapshot.pingCount > 0) {
    lines.push(`- **Ping Count:** ${snapshot.pingCount}`);
    if (snapshot.pingAvgMicros) {
      lines.push(`- **Ping Average:** ${snapshot.pingAvgMicros.toFixed(2)} µs`);
    }
    if (snapshot.pingMinMicros) {
      lines.push(`- **Ping Minimum:** ${snapshot.pingMinMicros.toFixed(2)} µs`);
    }
    if (snapshot.pingMaxMicros) {
      lines.push(`- **Ping Maximum:** ${snapshot.pingMaxMicros.toFixed(2)} µs`);
    }
  }
  lines.push(`- **Key Rotations Observed:** ${snapshot.keyRotationCount || 0}`);
  lines.push("");

  if (snapshot.fileTransferState !== "idle") {
    lines.push("### File Transfer");
    lines.push(`- **State:** ${snapshot.fileTransferState}`);
    if (snapshot.fileTransferFileName) {
      lines.push(`- **File Name:** ${snapshot.fileTransferFileName}`);
    }
    if (snapshot.fileTransferSizeBytes) {
      lines.push(`- **File Size:** ${snapshot.fileTransferSizeBytes} bytes`);
    }
    if (snapshot.fileTransferStartTime && snapshot.fileTransferEndTime) {
      const duration = ((snapshot.fileTransferEndTime - snapshot.fileTransferStartTime) / 1000).toFixed(2);
      lines.push(`- **Transfer Duration:** ${duration} seconds`);
    }
    lines.push("");
  }

  // Expected log patterns (from timeline if available)
  if (timeline && timeline.events.length > 0) {
    lines.push("### Expected Log Patterns");
    lines.push("");
    const hasHandshake = timeline.events.some((e) => e.kind === "status" && e.raw?.match(/handshake_complete/i));
    const hasRotation = timeline.events.some((e) => e.kind === "rotation");
    const hasConnection = timeline.events.some((e) => e.kind === "status" && e.raw?.match(/connection_established/i));
    const hasError = timeline.events.some((e) => e.kind === "error");

    if (hasHandshake) {
      lines.push("- `event=handshake_complete ... encryption=libp2p-transport cryprq_handshake=none status=ready`");
    }
    if (hasConnection) {
      lines.push("- `event=connection_established ... encryption_active=true`");
    }
    if (hasRotation) {
      lines.push("- `event=key_rotation status=success ... interval_secs=300`");
    }
    if (hasError) {
      lines.push("- Error events may appear (see Diagnostics section)");
    }
    lines.push("");
  }

  // Diagnostics & Classification
  lines.push("## Diagnostics & Classification");
  lines.push("");
  if (snapshot.diagnostics) {
    const diag = snapshot.diagnostics;
    lines.push("### Diagnostic Summary");
    lines.push(`- **ENV LIMITATION:** ${diag.countsBySeverity.env_limitation || 0}`);
    lines.push(`- **PRODUCT SUSPECT:** ${diag.countsBySeverity.product_bug || 0}`);
    lines.push(`- **INFO:** ${diag.countsBySeverity.info || 0}`);
    lines.push(`- **UNKNOWN:** ${diag.countsBySeverity.unknown || 0}`);
    lines.push("");

    if (diag.matchedHints.length > 0) {
      lines.push("### Diagnostic Hints");
      lines.push("");
      for (const hint of diag.matchedHints) {
        const severityLabel = hint.severity === "env_limitation" ? "ENV LIMITATION" :
          hint.severity === "product_bug" ? "PRODUCT SUSPECT" :
          hint.severity === "info" ? "INFO" : "UNKNOWN";
        lines.push(`- **${hint.title}** (${severityLabel})`);
        lines.push(`  - ${hint.summary}`);
        if (hint.suggestedActions.length > 0) {
          lines.push(`  - Suggested actions: ${hint.suggestedActions.join("; ")}`);
        }
        lines.push("");
      }
    }

    if (diag.countsBySeverity.env_limitation > 0 && diag.countsBySeverity.product_bug === 0) {
      lines.push("**Classification:** This scenario appears to be **environmental** (ENV LIMITATION only).");
      lines.push("The observed issues are likely due to environment constraints (missing privileges, network restrictions, etc.) and not product bugs.");
      lines.push("");
    } else if (diag.countsBySeverity.product_bug > 0) {
      lines.push("**Classification:** This scenario may indicate a **product issue** (PRODUCT SUSPECT hints detected).");
      lines.push("Review the diagnostic hints above and consider filing a bug report with this playbook.");
      lines.push("");
    }
  } else {
    lines.push("No diagnostic hints detected in this session.");
    lines.push("");
  }

  // Error Summary
  if (snapshot.errorCount > 0) {
    lines.push("### Error Summary");
    lines.push(`- **Total Errors:** ${snapshot.errorCount}`);
    if (snapshot.lastErrorAt && snapshot.lastErrorMessage) {
      lines.push(`- **Last Error:** ${snapshot.lastErrorAt} - ${snapshot.lastErrorMessage}`);
    }
    lines.push("");
  }

  // Notes & Variants
  lines.push("## Notes & Variants");
  lines.push("");
  if (sourceSessionId) {
    lines.push(`- **Source Session ID:** ${sourceSessionId}`);
    if (label) {
      lines.push(`- **Source Session Label:** ${label}`);
    }
    lines.push("");
  }
  lines.push("- Note 1: <add manual notes here>");
  lines.push("- Note 2: <add manual notes here>");
  lines.push("");
  lines.push("## Additional Resources");
  lines.push("");
  lines.push("- Use Session Comparison to compare this scenario with another session");
  lines.push("- Export Timeline JSON for detailed event-level analysis");
  lines.push("- Generate Support Bundle (`scripts/support_bundle.sh`) for comprehensive diagnostics");
  lines.push("- See `docs/WEB_TESTER_GUIDE.md` for detailed Web Tester usage");
  lines.push("");

  return lines.join("\n");
}

