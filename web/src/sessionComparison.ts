// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// License: MIT (see LICENSE file for details)

import type { StoredSession } from "./sessionHistory";

export interface SessionComparisonInput {
  left: StoredSession;
  right: StoredSession;
}

export interface SessionComparisonResult {
  configDiff: {
    mode: { left: string; right: string; equal: boolean };
    port: { left: number | string; right: number | string; equal: boolean };
    peerAddress: { left: string; right: string; equal: boolean };
    vpnEnabled: { left: boolean; right: boolean; equal: boolean };
  };
  metricsDiff: {
    handshakes: { left: number; right: number };
    ping: { leftAvg?: number; rightAvg?: number };
    keyRotations: { leftEpoch?: number; rightEpoch?: number };
    fileTransfers: {
      leftCount: number;
      rightCount: number;
      leftLastStatus?: string;
      rightLastStatus?: string;
    };
  };
  diagnosticsDiff: {
    envLimitations: { left: number; right: number };
    productSuspect: { left: number; right: number };
    info: { left: number; right: number };
    unknown: { left: number; right: number };
    commonHintIds: string[];
    leftOnlyHintIds: string[];
    rightOnlyHintIds: string[];
  };
  notes?: string[];
}

export function compareSessions(
  input: SessionComparisonInput
): SessionComparisonResult {
  const { left, right } = input;
  const leftSnap = left.snapshot;
  const rightSnap = right.snapshot;

  // Config diff
  const configDiff = {
    mode: {
      left: leftSnap.mode || "N/A",
      right: rightSnap.mode || "N/A",
      equal: leftSnap.mode === rightSnap.mode,
    },
    port: {
      left: leftSnap.port ?? "N/A",
      right: rightSnap.port ?? "N/A",
      equal: leftSnap.port === rightSnap.port,
    },
    peerAddress: {
      left: leftSnap.peerAddress || "N/A",
      right: rightSnap.peerAddress || "N/A",
      equal: leftSnap.peerAddress === rightSnap.peerAddress,
    },
    vpnEnabled: {
      left: leftSnap.vpnEnabled ?? false,
      right: rightSnap.vpnEnabled ?? false,
      equal: leftSnap.vpnEnabled === rightSnap.vpnEnabled,
    },
  };

  // Metrics diff
  const metricsDiff = {
    handshakes: {
      left: leftSnap.connectionAttemptsSucceeded || 0,
      right: rightSnap.connectionAttemptsSucceeded || 0,
    },
    ping: {
      leftAvg: leftSnap.pingAvgMicros,
      rightAvg: rightSnap.pingAvgMicros,
    },
    keyRotations: {
      leftEpoch: leftSnap.currentEpoch,
      rightEpoch: rightSnap.currentEpoch,
    },
    fileTransfers: {
      leftCount: leftSnap.fileTransferState === "completed" ? 1 : 0,
      rightCount: rightSnap.fileTransferState === "completed" ? 1 : 0,
      leftLastStatus: leftSnap.fileTransferState,
      rightLastStatus: rightSnap.fileTransferState,
    },
  };

  // Diagnostics diff
  const leftDiag = leftSnap.diagnostics;
  const rightDiag = rightSnap.diagnostics;
  const leftHints = leftDiag?.matchedHints || [];
  const rightHints = rightDiag?.matchedHints || [];
  const leftHintIds = new Set(leftHints.map((h) => h.id));
  const rightHintIds = new Set(rightHints.map((h) => h.id));
  const commonHintIds = Array.from(leftHintIds).filter((id) => rightHintIds.has(id));
  const leftOnlyHintIds = Array.from(leftHintIds).filter((id) => !rightHintIds.has(id));
  const rightOnlyHintIds = Array.from(rightHintIds).filter((id) => !leftHintIds.has(id));

  const diagnosticsDiff = {
    envLimitations: {
      left: leftDiag?.countsBySeverity.env_limitation || 0,
      right: rightDiag?.countsBySeverity.env_limitation || 0,
    },
    productSuspect: {
      left: leftDiag?.countsBySeverity.product_bug || 0,
      right: rightDiag?.countsBySeverity.product_bug || 0,
    },
    info: {
      left: leftDiag?.countsBySeverity.info || 0,
      right: rightDiag?.countsBySeverity.info || 0,
    },
    unknown: {
      left: leftDiag?.countsBySeverity.unknown || 0,
      right: rightDiag?.countsBySeverity.unknown || 0,
    },
    commonHintIds,
    leftOnlyHintIds,
    rightOnlyHintIds,
  };

  const notes: string[] = [];
  if (leftSnap.errorCount > 0 || rightSnap.errorCount > 0) {
    notes.push(
      `Left session: ${leftSnap.errorCount} errors; Right session: ${rightSnap.errorCount} errors`
    );
  }

  return {
    configDiff,
    metricsDiff,
    diagnosticsDiff,
    notes: notes.length > 0 ? notes : undefined,
  };
}

export function formatComparisonMarkdown(
  input: SessionComparisonInput,
  result: SessionComparisonResult
): string {
  const { left, right } = input;
  const lines: string[] = [];

  lines.push("# Session Comparison");
  lines.push("");
  lines.push(`**Baseline (Left):** ${left.label}`);
  lines.push(`- Created: ${new Date(left.createdAt).toLocaleString()}`);
  lines.push(`- ID: ${left.id}`);
  lines.push("");
  lines.push(`**Candidate (Right):** ${right.label}`);
  lines.push(`- Created: ${new Date(right.createdAt).toLocaleString()}`);
  lines.push(`- ID: ${right.id}`);
  lines.push("");
  lines.push("## Configuration Differences");
  lines.push("");
  lines.push("| Field | Baseline | Candidate | Equal |");
  lines.push("|-------|----------|-----------|-------|");
  lines.push(
    `| Mode | ${result.configDiff.mode.left} | ${result.configDiff.mode.right} | ${result.configDiff.mode.equal ? "Yes" : "**No**"} |`
  );
  lines.push(
    `| Port | ${result.configDiff.port.left} | ${result.configDiff.port.right} | ${result.configDiff.port.equal ? "Yes" : "**No**"} |`
  );
  lines.push(
    `| Peer Address | ${result.configDiff.peerAddress.left} | ${result.configDiff.peerAddress.right} | ${result.configDiff.peerAddress.equal ? "Yes" : "**No**"} |`
  );
  lines.push(
    `| VPN Enabled | ${result.configDiff.vpnEnabled.left} | ${result.configDiff.vpnEnabled.right} | ${result.configDiff.vpnEnabled.equal ? "Yes" : "**No**"} |`
  );
  lines.push("");
  lines.push("## Metrics Differences");
  lines.push("");
  lines.push("| Metric | Baseline | Candidate |");
  lines.push("|--------|----------|-----------|");
  lines.push(
    `| Successful Handshakes | ${result.metricsDiff.handshakes.left} | ${result.metricsDiff.handshakes.right} |`
  );
  lines.push(
    `| Ping Avg (µs) | ${result.metricsDiff.ping.leftAvg ?? "N/A"} | ${result.metricsDiff.ping.rightAvg ?? "N/A"} |`
  );
  lines.push(
    `| Key Rotation Epoch | ${result.metricsDiff.keyRotations.leftEpoch ?? "N/A"} | ${result.metricsDiff.keyRotations.rightEpoch ?? "N/A"} |`
  );
  lines.push(
    `| File Transfer Status | ${result.metricsDiff.fileTransfers.leftLastStatus ?? "N/A"} | ${result.metricsDiff.fileTransfers.rightLastStatus ?? "N/A"} |`
  );
  lines.push("");
  lines.push("## Diagnostics Differences");
  lines.push("");
  lines.push("| Severity | Baseline | Candidate |");
  lines.push("|---------|----------|-----------|");
  lines.push(
    `| ENV LIMITATION | ${result.diagnosticsDiff.envLimitations.left} | ${result.diagnosticsDiff.envLimitations.right} |`
  );
  lines.push(
    `| PRODUCT SUSPECT | ${result.diagnosticsDiff.productSuspect.left} | ${result.diagnosticsDiff.productSuspect.right} |`
  );
  lines.push(
    `| INFO | ${result.diagnosticsDiff.info.left} | ${result.diagnosticsDiff.info.right} |`
  );
  lines.push(
    `| UNKNOWN | ${result.diagnosticsDiff.unknown.left} | ${result.diagnosticsDiff.unknown.right} |`
  );
  lines.push("");
  if (result.diagnosticsDiff.commonHintIds.length > 0) {
    lines.push("### Common Diagnostic Hints");
    lines.push("");
    result.diagnosticsDiff.commonHintIds.forEach((id) => {
      lines.push(`- ${id}`);
    });
    lines.push("");
  }
  if (result.diagnosticsDiff.leftOnlyHintIds.length > 0) {
    lines.push("### Hints Only in Baseline");
    lines.push("");
    result.diagnosticsDiff.leftOnlyHintIds.forEach((id) => {
      lines.push(`- ${id}`);
    });
    lines.push("");
  }
  if (result.diagnosticsDiff.rightOnlyHintIds.length > 0) {
    lines.push("### Hints Only in Candidate");
    lines.push("");
    result.diagnosticsDiff.rightOnlyHintIds.forEach((id) => {
      lines.push(`- ${id}`);
    });
    lines.push("");
  }
  if (result.notes && result.notes.length > 0) {
    lines.push("## Notes");
    lines.push("");
    result.notes.forEach((note) => {
      lines.push(`- ${note}`);
    });
  }

  return lines.join("\n");
}

