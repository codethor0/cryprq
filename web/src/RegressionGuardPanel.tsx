// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// License: MIT (see LICENSE file for details)

import { useState, useEffect } from "react";
import type { SessionSnapshot, SessionTimeline } from "./sessionSnapshot";
import type { StoredSession } from "./sessionHistory";
import { getBaselines } from "./sessionHistory";
import {
  evaluateRegression,
  buildRegressionReportMarkdown,
  type RegressionEvaluation,
} from "./regressionGuard";

type RegressionGuardPanelProps = {
  currentSnapshot?: SessionSnapshot;
  currentTimeline?: SessionTimeline;
  storedSessions: StoredSession[];
};

export function RegressionGuardPanel({
  currentSnapshot,
  storedSessions,
}: RegressionGuardPanelProps) {
  const [selectedBaselineId, setSelectedBaselineId] = useState<string>("");
  const [evaluation, setEvaluation] = useState<RegressionEvaluation | null>(null);

  const baselines = getBaselines();
  const selectedBaseline = selectedBaselineId
    ? storedSessions.find((s) => s.id === selectedBaselineId)
    : undefined;

  useEffect(() => {
    if (selectedBaseline && currentSnapshot) {
      const evalResult = evaluateRegression(selectedBaseline.snapshot, currentSnapshot, {
        baselineLabel: selectedBaseline.baselineTag || selectedBaseline.label,
        currentLabel: "Current session",
      });
      setEvaluation(evalResult);
    } else {
      setEvaluation(null);
    }
  }, [selectedBaselineId, currentSnapshot, storedSessions]);

  const handleCopyReport = () => {
    if (evaluation && selectedBaseline && currentSnapshot) {
      const markdown = buildRegressionReportMarkdown(
        evaluation,
        selectedBaseline.snapshot,
        currentSnapshot
      );
      navigator.clipboard
        .writeText(markdown)
        .then(() => {
          alert("Regression Guard Report copied to clipboard");
        })
        .catch((err) => {
          console.error("Failed to copy report:", err);
          alert("Failed to copy report to clipboard");
        });
    }
  };

  const getStatusColor = (status: string): string => {
    switch (status) {
      case "PASS":
        return "#4f4";
      case "WARN":
        return "#ff8";
      case "FAIL":
        return "#f55";
      default:
        return "#888";
    }
  };

  return (
    <div
      style={{
        background: "#1a1a1a",
        border: "1px solid #333",
        borderRadius: 12,
        padding: 20,
        marginBottom: 20,
        boxShadow: "0 4px 12px rgba(0,0,0,0.3)",
      }}
    >
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: 16,
        }}
      >
        <div>
          <h3
            style={{
              marginTop: 0,
              marginBottom: 4,
              fontSize: 18,
              fontWeight: 600,
              color: "#fff",
            }}
          >
            Regression Guard (Baseline vs Current)
          </h3>
          <p
            style={{
              margin: 0,
              fontSize: 12,
              color: "#888",
            }}
          >
            Compare current session against a golden baseline and classify PASS/WARN/FAIL
          </p>
        </div>
        {evaluation && (
          <button
            onClick={handleCopyReport}
            style={{
              padding: "6px 12px",
              background: "#333",
              border: "1px solid #555",
              borderRadius: 6,
              color: "#fff",
              fontSize: 12,
              cursor: "pointer",
              fontWeight: 500,
            }}
            onMouseOver={(e) => {
              e.currentTarget.style.background = "#444";
            }}
            onMouseOut={(e) => {
              e.currentTarget.style.background = "#333";
            }}
          >
            Copy Report (Markdown)
          </button>
        )}
      </div>

      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: 12,
          marginBottom: 16,
        }}
      >
        <div>
          <label
            style={{
              display: "block",
              fontSize: 12,
              color: "#aaa",
              marginBottom: 6,
            }}
          >
            Select Baseline
          </label>
          <select
            value={selectedBaselineId}
            onChange={(e) => setSelectedBaselineId(e.target.value)}
            style={{
              width: "100%",
              padding: "6px 8px",
              background: "#0a0a0a",
              border: "1px solid #555",
              borderRadius: 4,
              color: "#fff",
              fontSize: 12,
            }}
          >
            <option value="">-- Select a baseline --</option>
            {baselines.map((s) => (
              <option key={s.id} value={s.id}>
                {s.baselineTag || s.label} ({new Date(s.createdAt).toLocaleString()})
              </option>
            ))}
          </select>
        </div>

        {baselines.length === 0 && (
          <p
            style={{
              margin: 0,
              fontSize: 12,
              color: "#888",
              fontStyle: "italic",
            }}
          >
            No baselines defined yet. Mark one of your saved sessions as a baseline in Session History.
          </p>
        )}

        {!currentSnapshot && (
          <p
            style={{
              margin: 0,
              fontSize: 12,
              color: "#888",
              fontStyle: "italic",
            }}
          >
            No active session snapshot available. Run a test first.
          </p>
        )}
      </div>

      {evaluation && (
        <div>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 12,
              marginBottom: 16,
              padding: 12,
              background: "#0a0a0a",
              borderRadius: 6,
            }}
          >
            <span
              style={{
                fontSize: 12,
                color: "#aaa",
                fontWeight: 600,
              }}
            >
              Status:
            </span>
            <span
              style={{
                padding: "4px 12px",
                background: getStatusColor(evaluation.status),
                color: "#000",
                borderRadius: 4,
                fontSize: 14,
                fontWeight: 600,
              }}
            >
              {evaluation.status}
            </span>
          </div>

          {evaluation.signals.length > 0 && (
            <div
              style={{
                background: "#0a0a0a",
                border: "1px solid #444",
                borderRadius: 8,
                padding: 16,
                marginBottom: 16,
              }}
            >
              <h4
                style={{
                  marginTop: 0,
                  marginBottom: 12,
                  fontSize: 14,
                  fontWeight: 600,
                  color: "#fff",
                }}
              >
                Regression Signals
              </h4>
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: 8,
                }}
              >
                {evaluation.signals.map((signal, idx) => (
                  <div
                    key={idx}
                    style={{
                      padding: 8,
                      background: "#1a1a1a",
                      borderRadius: 4,
                      borderLeft: `3px solid ${
                        signal.severity === "FAIL"
                          ? "#f55"
                          : signal.severity === "WARN"
                            ? "#ff8"
                            : "#59f"
                      }`,
                    }}
                  >
                    <div
                      style={{
                        display: "flex",
                        alignItems: "center",
                        gap: 8,
                        marginBottom: 4,
                      }}
                    >
                      <span
                        style={{
                          padding: "2px 6px",
                          background:
                            signal.severity === "FAIL"
                              ? "#f55"
                              : signal.severity === "WARN"
                                ? "#ff8"
                                : "#59f",
                          color: "#000",
                          borderRadius: 3,
                          fontSize: 9,
                          fontWeight: 600,
                        }}
                      >
                        {signal.severity}
                      </span>
                      <span
                        style={{
                          fontSize: 11,
                          color: "#aaa",
                          fontFamily: "ui-monospace, monospace",
                        }}
                      >
                        {signal.id}
                      </span>
                    </div>
                    <div
                      style={{
                        fontSize: 12,
                        color: "#fff",
                        marginBottom: signal.details ? 4 : 0,
                      }}
                    >
                      {signal.message}
                    </div>
                    {signal.details && (
                      <div
                        style={{
                          fontSize: 11,
                          color: "#888",
                          marginTop: 4,
                        }}
                      >
                        {signal.details}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}

          {evaluation.signals.length === 0 && (
            <div
              style={{
                padding: 16,
                background: "#0a0a0a",
                borderRadius: 6,
                textAlign: "center",
                fontSize: 12,
                color: "#4f4",
              }}
            >
              No regression signals detected. Session matches baseline expectations.
            </div>
          )}

          <div
            style={{
              display: "grid",
              gridTemplateColumns: "1fr 1fr",
              gap: 12,
              marginTop: 16,
            }}
          >
            <div
              style={{
                background: "#0a0a0a",
                border: "1px solid #444",
                borderRadius: 8,
                padding: 12,
              }}
            >
              <h4
                style={{
                  marginTop: 0,
                  marginBottom: 8,
                  fontSize: 12,
                  fontWeight: 600,
                  color: "#fff",
                }}
              >
                Baseline Metrics
              </h4>
              <div style={{ fontSize: 11, color: "#aaa", lineHeight: 1.6 }}>
                <div>
                  <strong>State:</strong> {selectedBaseline?.snapshot.connectionState || "N/A"}
                </div>
                <div>
                  <strong>Connections:</strong>{" "}
                  {selectedBaseline?.snapshot.connectionAttemptsSucceeded || 0}
                </div>
                <div>
                  <strong>Ping Avg:</strong>{" "}
                  {selectedBaseline?.snapshot.pingAvgMicros?.toFixed(2) || "N/A"} µs
                </div>
                <div>
                  <strong>Errors:</strong> {selectedBaseline?.snapshot.errorCount || 0}
                </div>
              </div>
            </div>
            <div
              style={{
                background: "#0a0a0a",
                border: "1px solid #444",
                borderRadius: 8,
                padding: 12,
              }}
            >
              <h4
                style={{
                  marginTop: 0,
                  marginBottom: 8,
                  fontSize: 12,
                  fontWeight: 600,
                  color: "#fff",
                }}
              >
                Current Metrics
              </h4>
              <div style={{ fontSize: 11, color: "#aaa", lineHeight: 1.6 }}>
                <div>
                  <strong>State:</strong> {currentSnapshot?.connectionState || "N/A"}
                </div>
                <div>
                  <strong>Connections:</strong> {currentSnapshot?.connectionAttemptsSucceeded || 0}
                </div>
                <div>
                  <strong>Ping Avg:</strong>{" "}
                  {currentSnapshot?.pingAvgMicros?.toFixed(2) || "N/A"} µs
                </div>
                <div>
                  <strong>Errors:</strong> {currentSnapshot?.errorCount || 0}
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

