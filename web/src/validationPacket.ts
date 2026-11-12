// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// License: MIT (see LICENSE file for details)

import type { SessionSnapshot, SessionTimeline } from "./sessionSnapshot";
import type { RegressionEvaluation } from "./regressionGuard";

export interface ValidationPacketContext {
  snapshot: SessionSnapshot;
  timeline?: SessionTimeline;
  baselineSnapshot?: SessionSnapshot;
  baselineLabel?: string;
  baselineCreatedAt?: string;
  regression?: RegressionEvaluation;
  scenarioPlaybookMarkdown?: string;
  bugReportDraftMarkdown?: string;
  label?: string;
  sourceType?: "current" | "stored";
  notes?: string;
}

export function buildValidationPacketMarkdown(ctx: ValidationPacketContext): string {
  const lines: string[] = [];
  const { snapshot, timeline, baselineSnapshot, baselineLabel, baselineCreatedAt, regression, scenarioPlaybookMarkdown, bugReportDraftMarkdown, label, sourceType, notes } = ctx;

  // Title & Summary
  lines.push(`# Validation Packet – ${label || "Unnamed Session"}`);
  lines.push("");
  lines.push("## Summary");
  lines.push("");
  lines.push(`- **Session Type:** ${sourceType === "stored" ? "Stored Session" : "Current Session"}`);
  if (snapshot.lastHandshakeTime) {
    lines.push(`- **Last Handshake:** ${snapshot.lastHandshakeTime}`);
  }
  if (baselineLabel || regression) {
    lines.push(`- **Baseline:** ${baselineLabel || "N/A"}`);
    if (baselineCreatedAt) {
      lines.push(`- **Baseline Created:** ${new Date(baselineCreatedAt).toLocaleString()}`);
    }
    if (regression) {
      lines.push(`- **Regression Status:** ${regression.status}`);
    }
  }
  if (notes) {
    lines.push("");
    lines.push("### Notes");
    lines.push(notes);
  }
  lines.push("");

  // Environment & Configuration
  lines.push("## Environment & Configuration");
  lines.push("");
  lines.push("### Connection Settings");
  lines.push(`- **Mode:** ${snapshot.mode === "listener" ? "Listener" : "Dialer"}`);
  lines.push(`- **Port:** ${snapshot.port ?? "N/A"}`);
  if (snapshot.mode === "dialer" && snapshot.peerAddress) {
    lines.push(`- **Peer Address:** ${snapshot.peerAddress}`);
  }
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

  // Connection & Metrics Overview
  lines.push("## Connection & Metrics Overview");
  lines.push("");
  lines.push("### Connection State");
  lines.push(`- **State:** ${snapshot.connectionState || "N/A"}`);
  if (snapshot.lastHandshakeDirection) {
    lines.push(`- **Last Handshake Direction:** ${snapshot.lastHandshakeDirection}`);
  }
  if (snapshot.lastHandshakeTime) {
    lines.push(`- **Last Handshake Time:** ${snapshot.lastHandshakeTime}`);
  }
  if (snapshot.currentEpoch) {
    lines.push(`- **Current Key Rotation Epoch:** ${snapshot.currentEpoch}`);
  }
  if (snapshot.rotationIntervalSecs) {
    lines.push(`- **Rotation Interval:** ${snapshot.rotationIntervalSecs} seconds`);
  }
  lines.push("");

  lines.push("### Connection Metrics");
  lines.push(`- **Total Connection Attempts:** ${snapshot.connectionAttemptsTotal || 0}`);
  lines.push(`- **Successful Connections:** ${snapshot.connectionAttemptsSucceeded || 0}`);
  lines.push(`- **Failed Connections:** ${snapshot.connectionAttemptsFailed || 0}`);
  lines.push(`- **Key Rotations Observed:** ${snapshot.keyRotationCount || 0}`);
  lines.push("");

  lines.push("### Latency Statistics");
  if (snapshot.pingCount > 0) {
    lines.push(`- **Ping Count:** ${snapshot.pingCount}`);
    if (snapshot.pingAvgMicros) {
      lines.push(`- **Average Latency:** ${snapshot.pingAvgMicros.toFixed(2)} µs`);
    }
    if (snapshot.pingMinMicros) {
      lines.push(`- **Minimum Latency:** ${snapshot.pingMinMicros.toFixed(2)} µs`);
    }
    if (snapshot.pingMaxMicros) {
      lines.push(`- **Maximum Latency:** ${snapshot.pingMaxMicros.toFixed(2)} µs`);
    }
  } else {
    lines.push("- **Ping Count:** 0 (no latency data available)");
  }
  lines.push("");

  lines.push("### Error Summary");
  lines.push(`- **Total Errors:** ${snapshot.errorCount || 0}`);
  if (snapshot.lastErrorAt && snapshot.lastErrorMessage) {
    lines.push(`- **Last Error:** ${snapshot.lastErrorAt} - ${snapshot.lastErrorMessage}`);
  }
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
      if (snapshot.fileTransferSizeBytes) {
        const throughput = (snapshot.fileTransferSizeBytes / ((snapshot.fileTransferEndTime - snapshot.fileTransferStartTime) / 1000)) / 1024;
        lines.push(`- **Throughput:** ${throughput.toFixed(2)} KB/s`);
      }
    }
    lines.push("");
  }

  // Diagnostics Summary
  lines.push("## Diagnostics Summary");
  lines.push("");
  if (snapshot.diagnostics) {
    const diag = snapshot.diagnostics;
    lines.push("### Diagnostic Counts");
    lines.push(`- **PRODUCT SUSPECT:** ${diag.countsBySeverity.product_bug || 0}`);
    lines.push(`- **ENV LIMITATION:** ${diag.countsBySeverity.env_limitation || 0}`);
    lines.push(`- **INFO:** ${diag.countsBySeverity.info || 0}`);
    lines.push(`- **UNKNOWN:** ${diag.countsBySeverity.unknown || 0}`);
    lines.push("");

    if (diag.matchedHints.length > 0) {
      lines.push("### Key Diagnostic Hints");
      lines.push("");
      for (const hint of diag.matchedHints) {
        const severityLabel = hint.severity === "env_limitation" ? "ENV LIMITATION" :
          hint.severity === "product_bug" ? "PRODUCT SUSPECT" :
          hint.severity === "info" ? "INFO" : "UNKNOWN";
        lines.push(`- **[${severityLabel}]** ${hint.title}`);
        lines.push(`  - ${hint.summary}`);
        if (hint.suggestedActions.length > 0) {
          lines.push(`  - Suggested actions: ${hint.suggestedActions.join("; ")}`);
        }
      }
      lines.push("");
    }
  } else {
    lines.push("No diagnostic hints detected in this session.");
    lines.push("");
  }

  // Regression Guard Summary
  lines.push("## Regression Guard Summary");
  lines.push("");
  if (regression && baselineSnapshot) {
    lines.push(`- **Overall Status:** ${regression.status}`);
    lines.push("");
    if (regression.signals.length > 0) {
      lines.push("### Regression Signals");
      lines.push("");
      for (const signal of regression.signals) {
        lines.push(`- **[${signal.severity}]** ${signal.id}: ${signal.message}`);
        if (signal.details) {
          lines.push(`  - ${signal.details}`);
        }
      }
      lines.push("");
    } else {
      lines.push("No regression signals detected. Session matches baseline expectations.");
      lines.push("");
    }

    lines.push("### Metric Comparison (Baseline vs Current)");
    lines.push("");
    lines.push("| Metric | Baseline | Current |");
    lines.push("|--------|----------|---------|");
    lines.push(
      `| Connection State | ${baselineSnapshot.connectionState || "N/A"} | ${snapshot.connectionState || "N/A"} |`
    );
    lines.push(
      `| Successful Connections | ${baselineSnapshot.connectionAttemptsSucceeded || 0} | ${snapshot.connectionAttemptsSucceeded || 0} |`
    );
    lines.push(
      `| Avg Ping Latency (µs) | ${baselineSnapshot.pingAvgMicros?.toFixed(2) || "N/A"} | ${snapshot.pingAvgMicros?.toFixed(2) || "N/A"} |`
    );
    lines.push(
      `| Key Rotations | ${baselineSnapshot.keyRotationCount || 0} | ${snapshot.keyRotationCount || 0} |`
    );
    lines.push(
      `| File Transfer Status | ${baselineSnapshot.fileTransferState || "N/A"} | ${snapshot.fileTransferState || "N/A"} |`
    );
    lines.push(
      `| Error Count | ${baselineSnapshot.errorCount || 0} | ${snapshot.errorCount || 0} |`
    );
    lines.push("");
  } else {
    lines.push("No baseline comparison included in this packet.");
    lines.push("");
  }

  // Scenario Playbook
  lines.push("## Scenario Playbook");
  lines.push("");
  if (scenarioPlaybookMarkdown) {
    // Strip leading # if present to avoid double headers
    const playbookContent = scenarioPlaybookMarkdown.replace(/^#\s+.*$/m, "");
    lines.push(playbookContent);
    lines.push("");
  } else {
    lines.push("Scenario Playbook not attached. You can generate one from the Web Tester UI.");
    lines.push("");
  }

  // Bug Report Draft
  if (bugReportDraftMarkdown) {
    lines.push("## Bug Report Draft (Optional)");
    lines.push("");
    lines.push(bugReportDraftMarkdown);
    lines.push("");
  } else {
    lines.push("## Bug Report Draft");
    lines.push("");
    lines.push("Bug Report Draft not attached. You can generate one from the Web Tester UI.");
    lines.push("");
  }

  // Timeline & Artifacts References
  lines.push("## Timeline & Artifacts References");
  lines.push("");
  if (timeline) {
    const eventCount = timeline.events.length;
    const errorCount = timeline.events.filter((e) => e.kind === "error").length;
    lines.push(`- **Total Events:** ${eventCount}`);
    lines.push(`- **Error Events:** ${errorCount}`);
    if (timeline.events.length > 0) {
      const firstEvent = timeline.events[0];
      const lastEvent = timeline.events[timeline.events.length - 1];
      if (firstEvent.timestamp && lastEvent.timestamp) {
        lines.push(`- **Time Span:** ${firstEvent.timestamp} to ${lastEvent.timestamp}`);
      }
    }
    lines.push("");
  } else {
    lines.push("- **Timeline:** Not available in this packet");
    lines.push("");
  }

  lines.push("### Additional Artifacts");
  lines.push("");
  lines.push("This Validation Packet is a summary. For detailed analysis, consider also including:");
  lines.push("");
  lines.push("- **Timeline JSON Export:** Download from Debug Console for event-level analysis");
  lines.push("- **Raw Logs Export:** Download logs from Debug Console for full log history");
  lines.push("- **Support Bundle Tarball:** Run `scripts/support_bundle.sh` for comprehensive system diagnostics");
  lines.push("- **CLI Validation Run Report:** Run `scripts/validation_run_report.sh` for automated probe outputs");
  lines.push("");

  return lines.join("\n");
}

