// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// License: MIT (see LICENSE file for details)

import { useState, useEffect } from "react";
import type { StoredSession } from "./sessionHistory";
import { compareSessions, formatComparisonMarkdown, type SessionComparisonResult } from "./sessionComparison";

type SessionComparePanelProps = {
  sessions: StoredSession[];
  leftSelectedId?: string;
  rightSelectedId?: string;
  onLeftSelect?: (id: string) => void;
  onRightSelect?: (id: string) => void;
};

export function SessionComparePanel({
  sessions,
  leftSelectedId,
  rightSelectedId,
  onLeftSelect,
  onRightSelect,
}: SessionComparePanelProps) {
  const [comparison, setComparison] = useState<SessionComparisonResult | null>(null);

  const leftSession = leftSelectedId ? sessions.find((s) => s.id === leftSelectedId) : undefined;
  const rightSession = rightSelectedId ? sessions.find((s) => s.id === rightSelectedId) : undefined;

  useEffect(() => {
    if (leftSession && rightSession) {
      const result = compareSessions({ left: leftSession, right: rightSession });
      setComparison(result);
    } else {
      setComparison(null);
    }
  }, [leftSelectedId, rightSelectedId, sessions]);

  const handleCopyMarkdown = () => {
    if (leftSession && rightSession && comparison) {
      const markdown = formatComparisonMarkdown(
        { left: leftSession, right: rightSession },
        comparison
      );
      navigator.clipboard
        .writeText(markdown)
        .then(() => {
          alert("Comparison copied to clipboard");
        })
        .catch((err) => {
          console.error("Failed to copy comparison:", err);
        });
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
      <h3
        style={{
          marginTop: 0,
          marginBottom: 16,
          fontSize: 18,
          fontWeight: 600,
          color: "#fff",
        }}
      >
        Session Comparison
      </h3>

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr",
          gap: 16,
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
            Baseline Session (Left)
          </label>
          <select
            value={leftSelectedId || ""}
            onChange={(e) => onLeftSelect?.(e.target.value)}
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
            <option value="">-- Select --</option>
            {sessions.map((s) => (
              <option key={s.id} value={s.id}>
                {s.label}
              </option>
            ))}
          </select>
        </div>
        <div>
          <label
            style={{
              display: "block",
              fontSize: 12,
              color: "#aaa",
              marginBottom: 6,
            }}
          >
            Candidate Session (Right)
          </label>
          <select
            value={rightSelectedId || ""}
            onChange={(e) => onRightSelect?.(e.target.value)}
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
            <option value="">-- Select --</option>
            {sessions.map((s) => (
              <option key={s.id} value={s.id}>
                {s.label}
              </option>
            ))}
          </select>
        </div>
      </div>

      {comparison && (
        <>
          <div
            style={{
              marginBottom: 16,
              display: "flex",
              justifyContent: "flex-end",
            }}
          >
            <button
              onClick={handleCopyMarkdown}
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
              Copy Comparison (Markdown)
            </button>
          </div>

          <div
            style={{
              display: "grid",
              gridTemplateColumns: "1fr 1fr",
              gap: 16,
            }}
          >
            {/* Config Diff */}
            <div
              style={{
                background: "#0a0a0a",
                border: "1px solid #444",
                borderRadius: 8,
                padding: 16,
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
                Configuration
              </h4>
              <div style={{ fontSize: 11, color: "#aaa", lineHeight: 1.8 }}>
                <div>
                  <strong>Mode:</strong> {comparison.configDiff.mode.left} →{" "}
                  {comparison.configDiff.mode.right}{" "}
                  {!comparison.configDiff.mode.equal && (
                    <span style={{ color: "#f55" }}>(Changed)</span>
                  )}
                </div>
                <div>
                  <strong>Port:</strong> {comparison.configDiff.port.left} →{" "}
                  {comparison.configDiff.port.right}{" "}
                  {!comparison.configDiff.port.equal && (
                    <span style={{ color: "#f55" }}>(Changed)</span>
                  )}
                </div>
                <div>
                  <strong>Peer:</strong> {comparison.configDiff.peerAddress.left} →{" "}
                  {comparison.configDiff.peerAddress.right}{" "}
                  {!comparison.configDiff.peerAddress.equal && (
                    <span style={{ color: "#f55" }}>(Changed)</span>
                  )}
                </div>
                <div>
                  <strong>VPN:</strong> {String(comparison.configDiff.vpnEnabled.left)} →{" "}
                  {String(comparison.configDiff.vpnEnabled.right)}{" "}
                  {!comparison.configDiff.vpnEnabled.equal && (
                    <span style={{ color: "#f55" }}>(Changed)</span>
                  )}
                </div>
              </div>
            </div>

            {/* Metrics Diff */}
            <div
              style={{
                background: "#0a0a0a",
                border: "1px solid #444",
                borderRadius: 8,
                padding: 16,
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
                Metrics
              </h4>
              <div style={{ fontSize: 11, color: "#aaa", lineHeight: 1.8 }}>
                <div>
                  <strong>Handshakes:</strong> {comparison.metricsDiff.handshakes.left} →{" "}
                  {comparison.metricsDiff.handshakes.right}
                </div>
                <div>
                  <strong>Ping Avg:</strong>{" "}
                  {comparison.metricsDiff.ping.leftAvg ?? "N/A"} →{" "}
                  {comparison.metricsDiff.ping.rightAvg ?? "N/A"}
                </div>
                <div>
                  <strong>Key Rotation Epoch:</strong>{" "}
                  {comparison.metricsDiff.keyRotations.leftEpoch ?? "N/A"} →{" "}
                  {comparison.metricsDiff.keyRotations.rightEpoch ?? "N/A"}
                </div>
                <div>
                  <strong>File Transfer:</strong>{" "}
                  {comparison.metricsDiff.fileTransfers.leftLastStatus ?? "N/A"} →{" "}
                  {comparison.metricsDiff.fileTransfers.rightLastStatus ?? "N/A"}
                </div>
              </div>
            </div>

            {/* Diagnostics Diff */}
            <div
              style={{
                background: "#0a0a0a",
                border: "1px solid #444",
                borderRadius: 8,
                padding: 16,
                gridColumn: "1 / -1",
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
                Diagnostics
              </h4>
              <div
                style={{
                  display: "grid",
                  gridTemplateColumns: "1fr 1fr",
                  gap: 16,
                }}
              >
                <div style={{ fontSize: 11, color: "#aaa", lineHeight: 1.8 }}>
                  <div>
                    <strong>ENV LIMITATION:</strong> {comparison.diagnosticsDiff.envLimitations.left} →{" "}
                    {comparison.diagnosticsDiff.envLimitations.right}
                  </div>
                  <div>
                    <strong>PRODUCT SUSPECT:</strong> {comparison.diagnosticsDiff.productSuspect.left} →{" "}
                    {comparison.diagnosticsDiff.productSuspect.right}
                  </div>
                  <div>
                    <strong>INFO:</strong> {comparison.diagnosticsDiff.info.left} →{" "}
                    {comparison.diagnosticsDiff.info.right}
                  </div>
                  <div>
                    <strong>UNKNOWN:</strong> {comparison.diagnosticsDiff.unknown.left} →{" "}
                    {comparison.diagnosticsDiff.unknown.right}
                  </div>
                </div>
                <div style={{ fontSize: 11, color: "#aaa", lineHeight: 1.8 }}>
                  {comparison.diagnosticsDiff.commonHintIds.length > 0 && (
                    <div>
                      <strong>Common Hints:</strong> {comparison.diagnosticsDiff.commonHintIds.length}
                    </div>
                  )}
                  {comparison.diagnosticsDiff.leftOnlyHintIds.length > 0 && (
                    <div>
                      <strong>Left Only:</strong> {comparison.diagnosticsDiff.leftOnlyHintIds.length}
                    </div>
                  )}
                  {comparison.diagnosticsDiff.rightOnlyHintIds.length > 0 && (
                    <div>
                      <strong>Right Only:</strong> {comparison.diagnosticsDiff.rightOnlyHintIds.length}
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>
        </>
      )}

      {!leftSession || !rightSession ? (
        <p
          style={{
            margin: "16px 0 0 0",
            fontSize: 13,
            color: "#888",
            fontStyle: "italic",
            textAlign: "center",
          }}
        >
          Select two sessions to compare
        </p>
      ) : null}
    </div>
  );
}

