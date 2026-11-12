// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// License: MIT (see LICENSE file for details)

import { useState } from "react";
import type { SessionSnapshot } from "./sessionSnapshot";

type BugReportDraftProps = {
  sessionSnapshot: SessionSnapshot;
};

export function BugReportDraft({ sessionSnapshot }: BugReportDraftProps) {
  const [copied, setCopied] = useState(false);

  const generateMarkdown = (): string => {
    const formatFileSize = (bytes?: number): string => {
      if (!bytes) return "<not recorded>";
      if (bytes < 1024) return `${bytes} B`;
      if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
      return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
    };

    const formatThroughput = (): string => {
      if (
        !sessionSnapshot.fileTransferSizeBytes ||
        !sessionSnapshot.fileTransferStartTime ||
        !sessionSnapshot.fileTransferEndTime
      ) {
        return "<not recorded>";
      }
      const durationMs = sessionSnapshot.fileTransferEndTime - sessionSnapshot.fileTransferStartTime;
      if (durationMs <= 0) return "<not recorded>";
      const throughputMbps = ((sessionSnapshot.fileTransferSizeBytes * 8) / (durationMs / 1000)) / 1000000;
      return `${throughputMbps.toFixed(2)} Mbps`;
    };

    const formatPingLatency = (micros?: number): string => {
      if (micros === undefined) return "<not recorded>";
      if (micros < 1000) return `${micros.toFixed(2)}µs`;
      if (micros < 1000000) return `${(micros / 1000).toFixed(2)}ms`;
      return `${(micros / 1000000).toFixed(2)}s`;
    };

    return `# Bug Report – CrypRQ

## Summary

[Describe the issue in 1–2 sentences.]

## Environment

### CLI Command
\`\`\`
${sessionSnapshot.cliSnippet}
\`\`\`

### Web Tester Configuration
- Mode: ${sessionSnapshot.mode === "listener" ? "Listener" : "Dialer"}
- Port: ${sessionSnapshot.port ?? "<not set>"}
- Peer Address: ${sessionSnapshot.peerAddress ?? "<not set>"}
- VPN Enabled: ${sessionSnapshot.vpnEnabled ? "true" : "false"}

### Browser
${sessionSnapshot.environmentUserAgent}

## Reproduction Steps

1. [Step 1]
2. [Step 2]
3. [Step 3]

## Observed Behavior

- Connection state: ${sessionSnapshot.connectionState === "encrypted-tunnel-active" ? "Encrypted tunnel active" : sessionSnapshot.connectionState === "connecting" ? "Connecting..." : "Idle"}
- Last handshake: ${sessionSnapshot.lastHandshakeDirection ? `${sessionSnapshot.lastHandshakeDirection === "inbound" ? "Inbound (Listener)" : "Outbound (Dialer)"}${sessionSnapshot.lastHandshakeTime ? ` at ${sessionSnapshot.lastHandshakeTime}` : ""}` : "<not recorded>"}
- Connection attempts: ${sessionSnapshot.connectionAttemptsTotal} total (${sessionSnapshot.connectionAttemptsSucceeded} succeeded, ${sessionSnapshot.connectionAttemptsFailed} failed)
${sessionSnapshot.lastErrorMessage ? `- Last error: ${sessionSnapshot.lastErrorMessage}${sessionSnapshot.lastErrorAt ? ` at ${sessionSnapshot.lastErrorAt}` : ""}` : ""}

## Expected Behavior

[What should have happened]

## Metrics

### Ping Latency
${sessionSnapshot.pingCount > 0 ? `- Min: ${formatPingLatency(sessionSnapshot.pingMinMicros)}, Max: ${formatPingLatency(sessionSnapshot.pingMaxMicros)}, Avg: ${formatPingLatency(sessionSnapshot.pingAvgMicros)} (${sessionSnapshot.pingCount} samples)` : "- No ping events recorded"}

### Key Rotation
- Rotation count: ${sessionSnapshot.keyRotationCount}
${sessionSnapshot.currentEpoch !== undefined ? `- Current epoch: ${sessionSnapshot.currentEpoch}` : ""}
${sessionSnapshot.rotationIntervalSecs !== undefined ? `- Rotation interval: ${sessionSnapshot.rotationIntervalSecs}s` : ""}

### File Transfer
- Last transfer status: ${sessionSnapshot.fileTransferState === "idle" ? "Idle" : sessionSnapshot.fileTransferState === "completed" ? "Completed" : sessionSnapshot.fileTransferState === "failed" ? "Failed" : sessionSnapshot.fileTransferState === "in_progress" ? "In progress" : sessionSnapshot.fileTransferState === "preparing" ? "Preparing" : "<not recorded>"}
${sessionSnapshot.fileTransferFileName ? `- File name: ${sessionSnapshot.fileTransferFileName}` : ""}
${sessionSnapshot.fileTransferSizeBytes ? `- File size: ${formatFileSize(sessionSnapshot.fileTransferSizeBytes)}` : ""}
${sessionSnapshot.fileTransferStartTime && sessionSnapshot.fileTransferEndTime ? `- Duration: ${((sessionSnapshot.fileTransferEndTime - sessionSnapshot.fileTransferStartTime) / 1000).toFixed(2)}s` : ""}
${sessionSnapshot.fileTransferState === "completed" ? `- Throughput: ${formatThroughput()}` : ""}

### Errors
- Error count: ${sessionSnapshot.errorCount}
${sessionSnapshot.lastErrorAt ? `- Last error at: ${sessionSnapshot.lastErrorAt}` : ""}

## Attachments

- Output of: \`bash scripts/quick_smoke.sh\`
- Latest support bundle from: \`./scripts/support_bundle.sh\`
- Exported logs from Web Tester ("Download Logs" button)
- Screenshots of Connection Health, Advanced Metrics, and Test Summary panels
`;
  };

  const handleCopy = async () => {
    const markdown = generateMarkdown();
    try {
      if (navigator.clipboard && navigator.clipboard.writeText) {
        await navigator.clipboard.writeText(markdown);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      }
    } catch (err) {
      console.debug("Clipboard API not available:", err);
    }
  };

  const markdown = generateMarkdown();

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
        Bug Report Draft
      </h3>

      <div style={{ marginBottom: "12px", color: "#888", fontSize: "11px" }}>
        This draft is generated from your current session state. Fill in the placeholders and attach the requested files.
      </div>

      <div
        style={{
          display: "flex",
          gap: 8,
          alignItems: "flex-start",
          marginBottom: "8px",
        }}
      >
        <textarea
          readOnly
          value={markdown}
          style={{
            flex: 1,
            padding: "10px 12px",
            background: "#0a0a0a",
            border: "1px solid #444",
            borderRadius: 6,
            color: "#fff",
            fontSize: 11,
            fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace",
            resize: "vertical",
            minHeight: "400px",
            lineHeight: 1.5,
            whiteSpace: "pre-wrap",
            wordBreak: "break-word",
          }}
          onClick={(e) => {
            (e.target as HTMLTextAreaElement).select();
          }}
        />
        <button
          onClick={handleCopy}
          style={{
            padding: "10px 16px",
            background: copied ? "#2a5" : "#333",
            border: "1px solid #555",
            borderRadius: 6,
            color: "#fff",
            fontSize: 12,
            cursor: "pointer",
            fontWeight: 500,
            whiteSpace: "nowrap",
            transition: "background 0.2s",
          }}
        >
          {copied ? "Copied!" : "Copy Draft"}
        </button>
      </div>

      {copied && (
        <div
          style={{
            fontSize: 11,
            color: "#2a5",
            marginTop: 8,
          }}
        >
          Bug report draft copied to clipboard
        </div>
      )}
    </div>
  );
}

