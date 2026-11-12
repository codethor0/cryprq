// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// License: MIT (see LICENSE file for details)

import type { SessionSnapshot } from "./sessionSnapshot";

export type RegressionStatus = "PASS" | "WARN" | "FAIL";

export interface RegressionSignal {
  id: string;
  severity: "INFO" | "WARN" | "FAIL";
  message: string;
  details?: string;
}

export interface RegressionEvaluation {
  status: RegressionStatus;
  signals: RegressionSignal[];
  baselineLabel?: string;
  currentLabel?: string;
  baselineCreatedAt?: string;
  notes?: string;
}

export function evaluateRegression(
  baseline: SessionSnapshot,
  current: SessionSnapshot,
  opts?: { baselineLabel?: string; currentLabel?: string }
): RegressionEvaluation {
  const signals: RegressionSignal[] = [];

  // 1. Diagnostics delta
  const baselineDiag = baseline.diagnostics;
  const currentDiag = current.diagnostics;

  if (baselineDiag && currentDiag) {
    const baselineProductSuspect = baselineDiag.countsBySeverity.product_bug || 0;
    const currentProductSuspect = currentDiag.countsBySeverity.product_bug || 0;

    if (baselineProductSuspect === 0 && currentProductSuspect > 0) {
      signals.push({
        id: "diagnostics_product_suspect_new",
        severity: "FAIL",
        message: "Current session shows PRODUCT SUSPECT diagnostics not present in baseline",
        details: `Baseline: ${baselineProductSuspect} PRODUCT SUSPECT hints; Current: ${currentProductSuspect} PRODUCT SUSPECT hints`,
      });
    }

    const baselineEnvLim = baselineDiag.countsBySeverity.env_limitation || 0;
    const currentEnvLim = currentDiag.countsBySeverity.env_limitation || 0;

    if (currentEnvLim > baselineEnvLim && currentProductSuspect === 0) {
      signals.push({
        id: "diagnostics_env_limitation_increase",
        severity: "INFO",
        message: "Current session has more ENV LIMITATION hints than baseline",
        details: `This is likely environmental (missing privileges, network restrictions, etc.) and not a product bug. Baseline: ${baselineEnvLim} ENV LIMITATION; Current: ${currentEnvLim} ENV LIMITATION`,
      });
    }
  }

  // 2. Connection health & errors
  const baselineConnected =
    baseline.connectionState === "encrypted-tunnel-active" ||
    baseline.connectionAttemptsSucceeded > 0;
  const currentConnected =
    current.connectionState === "encrypted-tunnel-active" ||
    current.connectionAttemptsSucceeded > 0;

  if (baselineConnected && !currentConnected) {
    signals.push({
      id: "connection_state_regressed",
      severity: "FAIL",
      message: "Current session did not reach connected state that baseline achieved",
      details: `Baseline: ${baseline.connectionState} (${baseline.connectionAttemptsSucceeded} successful connections); Current: ${current.connectionState} (${current.connectionAttemptsSucceeded} successful connections)`,
    });
  }

  const baselineErrors = baseline.errorCount || 0;
  const currentErrors = current.errorCount || 0;

  if (baselineErrors === 0 && currentErrors > 0) {
    const severity = (currentDiag?.countsBySeverity.product_bug || 0) > 0 ? "FAIL" : "WARN";
    signals.push({
      id: "error_count_increase",
      severity,
      message: `Current session has ${currentErrors} error(s) while baseline had none`,
      details: severity === "WARN" ? "Review diagnostics to determine if this is environmental or a product issue" : "PRODUCT SUSPECT diagnostics detected",
    });
  }

  // 3. Latency comparison
  if (baseline.pingAvgMicros && current.pingAvgMicros && baseline.pingAvgMicros > 0) {
    const ratio = current.pingAvgMicros / baseline.pingAvgMicros;
    if (ratio >= 2.0) {
      signals.push({
        id: "latency_increase_warn",
        severity: "WARN",
        message: "Current session average latency is significantly higher than baseline",
        details: `Baseline avg: ${baseline.pingAvgMicros.toFixed(2)} µs; Current avg: ${current.pingAvgMicros.toFixed(2)} µs (${ratio.toFixed(2)}x slower). Network noise may be a factor.`,
      });
    }
  }

  // 4. File transfer behavior
  if (baseline.fileTransferState === "completed" && current.fileTransferState === "failed") {
    signals.push({
      id: "file_transfer_regressed",
      severity: "FAIL",
      message: "File transfer succeeded in baseline but failed in current session",
      details: `Baseline: ${baseline.fileTransferState}; Current: ${current.fileTransferState}`,
    });
  }

  if (
    baseline.fileTransferState === "completed" &&
    current.fileTransferState === "completed" &&
    baseline.fileTransferStartTime &&
    baseline.fileTransferEndTime &&
    current.fileTransferStartTime &&
    current.fileTransferEndTime
  ) {
    const baselineDuration =
      (baseline.fileTransferEndTime - baseline.fileTransferStartTime) / 1000;
    const currentDuration =
      (current.fileTransferEndTime - current.fileTransferStartTime) / 1000;

    if (baselineDuration > 0 && currentDuration > 0) {
      const baselineThroughput =
        (baseline.fileTransferSizeBytes || 0) / baselineDuration;
      const currentThroughput = (current.fileTransferSizeBytes || 0) / currentDuration;

      if (baselineThroughput > 0 && currentThroughput < baselineThroughput * 0.5) {
        signals.push({
          id: "throughput_drop_warn",
          severity: "WARN",
          message: "File transfer throughput dropped significantly compared to baseline",
          details: `Baseline: ${(baselineThroughput / 1024).toFixed(2)} KB/s; Current: ${(currentThroughput / 1024).toFixed(2)} KB/s. Performance variation may be environmental.`,
        });
      }
    }
  }

  // Aggregation: determine overall status
  let status: RegressionStatus = "PASS";
  if (signals.some((s) => s.severity === "FAIL")) {
    status = "FAIL";
  } else if (signals.some((s) => s.severity === "WARN")) {
    status = "WARN";
  }

  return {
    status,
    signals,
    baselineLabel: opts?.baselineLabel,
    currentLabel: opts?.currentLabel || "Current session",
  };
}

export function buildRegressionReportMarkdown(
  evaluation: RegressionEvaluation,
  baselineSnapshot: SessionSnapshot,
  currentSnapshot: SessionSnapshot
): string {
  const lines: string[] = [];

  lines.push("# Regression Guard Report");
  lines.push("");

  // Summary
  lines.push("## Summary");
  lines.push("");
  lines.push(`- **Baseline:** ${evaluation.baselineLabel || "Unnamed baseline"}`);
  if (evaluation.baselineCreatedAt) {
    lines.push(`- **Baseline Created:** ${new Date(evaluation.baselineCreatedAt).toLocaleString()}`);
  }
  lines.push(`- **Current Session:** ${evaluation.currentLabel || "Current session"}`);
  lines.push(`- **Overall Status:** ${evaluation.status}`);
  lines.push("");

  // Signals
  lines.push("## Regression Signals");
  lines.push("");
  if (evaluation.signals.length === 0) {
    lines.push("No regression signals detected. Session matches baseline expectations.");
    lines.push("");
  } else {
    for (const signal of evaluation.signals) {
      lines.push(`- **[${signal.severity}]** ${signal.id}: ${signal.message}`);
      if (signal.details) {
        lines.push(`  - ${signal.details}`);
      }
    }
    lines.push("");
  }

  // Key metric comparison
  lines.push("## Key Metric Comparison");
  lines.push("");
  lines.push("| Metric | Baseline | Current |");
  lines.push("|--------|----------|---------|");
  lines.push(
    `| Connection State | ${baselineSnapshot.connectionState || "N/A"} | ${currentSnapshot.connectionState || "N/A"} |`
  );
  lines.push(
    `| Successful Connections | ${baselineSnapshot.connectionAttemptsSucceeded || 0} | ${currentSnapshot.connectionAttemptsSucceeded || 0} |`
  );
  lines.push(
    `| Avg Ping Latency (µs) | ${baselineSnapshot.pingAvgMicros?.toFixed(2) || "N/A"} | ${currentSnapshot.pingAvgMicros?.toFixed(2) || "N/A"} |`
  );
  lines.push(
    `| Key Rotations Observed | ${baselineSnapshot.keyRotationCount || 0} | ${currentSnapshot.keyRotationCount || 0} |`
  );
  lines.push(
    `| File Transfer Status | ${baselineSnapshot.fileTransferState || "N/A"} | ${currentSnapshot.fileTransferState || "N/A"} |`
  );
  lines.push(
    `| Error Count | ${baselineSnapshot.errorCount || 0} | ${currentSnapshot.errorCount || 0} |`
  );
  lines.push("");

  // Diagnostics summary
  lines.push("## Diagnostics Summary");
  lines.push("");
  lines.push("| Severity | Baseline | Current |");
  lines.push("|----------|----------|---------|");
  const baselineDiag = baselineSnapshot.diagnostics;
  const currentDiag = currentSnapshot.diagnostics;
  lines.push(
    `| ENV LIMITATION | ${baselineDiag?.countsBySeverity.env_limitation || 0} | ${currentDiag?.countsBySeverity.env_limitation || 0} |`
  );
  lines.push(
    `| PRODUCT SUSPECT | ${baselineDiag?.countsBySeverity.product_bug || 0} | ${currentDiag?.countsBySeverity.product_bug || 0} |`
  );
  lines.push(
    `| INFO | ${baselineDiag?.countsBySeverity.info || 0} | ${currentDiag?.countsBySeverity.info || 0} |`
  );
  lines.push(
    `| UNKNOWN | ${baselineDiag?.countsBySeverity.unknown || 0} | ${currentDiag?.countsBySeverity.unknown || 0} |`
  );
  lines.push("");

  // References
  lines.push("## Additional Resources");
  lines.push("");
  lines.push("- Pair this report with:");
  lines.push("  - Scenario Playbook (for reproducible steps)");
  lines.push("  - Bug Report Draft (for detailed session information)");
  lines.push("  - Timeline JSON export (for event-level analysis)");
  lines.push("  - Support bundle tarball (`scripts/support_bundle.sh`)");
  lines.push("  - Validation run report (`scripts/validation_run_report.sh`)");
  lines.push("");

  if (evaluation.notes) {
    lines.push("## Notes");
    lines.push("");
    lines.push(evaluation.notes);
    lines.push("");
  }

  return lines.join("\n");
}

