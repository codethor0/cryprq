// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// License: MIT (see LICENSE file for details)

import type { DiagnosticSummary, DiagnosticSeverity } from "./diagnostics";

type DiagnosticHintsProps = {
  summary?: DiagnosticSummary;
};

export function DiagnosticHints({ summary }: DiagnosticHintsProps) {
  if (!summary || summary.matchedHints.length === 0) {
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
            marginBottom: 12,
            fontSize: 18,
            fontWeight: 600,
            color: "#fff",
          }}
        >
          Diagnostic Hints
        </h3>
        <p
          style={{
            margin: 0,
            fontSize: 13,
            color: "#888",
            fontStyle: "italic",
          }}
        >
          No specific diagnostic hints detected. Check metrics and logs for more details.
        </p>
      </div>
    );
  }

  const { matchedHints, countsBySeverity } = summary;

  const getSeverityColor = (severity: DiagnosticSeverity): string => {
    switch (severity) {
      case "env_limitation":
        return "#ff8";
      case "product_bug":
        return "#f55";
      case "info":
        return "#59f";
      default:
        return "#888";
    }
  };

  const getSeverityLabel = (severity: DiagnosticSeverity): string => {
    switch (severity) {
      case "env_limitation":
        return "ENV LIMITATION";
      case "product_bug":
        return "PRODUCT SUSPECT";
      case "info":
        return "INFO";
      default:
        return "UNKNOWN";
    }
  };

  const handleCopySummary = () => {
    const summaryText = `# Diagnostic Summary

## Counts by Severity
- ENV LIMITATION: ${countsBySeverity.env_limitation}
- PRODUCT SUSPECT: ${countsBySeverity.product_bug}
- INFO: ${countsBySeverity.info}
- UNKNOWN: ${countsBySeverity.unknown}

## Matched Hints

${matchedHints
  .map(
    (hint) => `### ${hint.title} (${getSeverityLabel(hint.severity)})

${hint.summary}

${hint.details ? `${hint.details}\n` : ""}**Suggested Actions:**
${hint.suggestedActions.map((action) => `- ${action}`).join("\n")}`
  )
  .join("\n\n---\n\n")}
`;

    navigator.clipboard
      .writeText(summaryText)
      .then(() => {
        alert("Diagnostic summary copied to clipboard");
      })
      .catch((err) => {
        console.error("Failed to copy diagnostic summary:", err);
      });
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
        <h3
          style={{
            marginTop: 0,
            marginBottom: 0,
            fontSize: 18,
            fontWeight: 600,
            color: "#fff",
          }}
        >
          Diagnostic Hints
        </h3>
        <button
          onClick={handleCopySummary}
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
          Copy Summary
        </button>
      </div>

      <div
        style={{
          marginBottom: 16,
          padding: 12,
          background: "#0a0a0a",
          borderRadius: 6,
          fontSize: 12,
          color: "#aaa",
        }}
      >
        <strong>Summary:</strong> {countsBySeverity.env_limitation} env limitations,{" "}
        {countsBySeverity.product_bug} product-suspect, {countsBySeverity.info} info,{" "}
        {countsBySeverity.unknown} unknown
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
        {matchedHints.map((hint) => (
          <div
            key={hint.id}
            style={{
              background: "#0a0a0a",
              border: "1px solid #444",
              borderRadius: 8,
              padding: 16,
            }}
          >
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "flex-start",
                marginBottom: 8,
              }}
            >
              <h4
                style={{
                  marginTop: 0,
                  marginBottom: 0,
                  fontSize: 14,
                  fontWeight: 600,
                  color: "#fff",
                }}
              >
                {hint.title}
              </h4>
              <span
                style={{
                  padding: "2px 8px",
                  background: getSeverityColor(hint.severity),
                  color: "#000",
                  borderRadius: 4,
                  fontSize: 10,
                  fontWeight: 600,
                  whiteSpace: "nowrap",
                  marginLeft: 12,
                }}
              >
                {getSeverityLabel(hint.severity)}
              </span>
            </div>
            <p
              style={{
                margin: "0 0 8px 0",
                fontSize: 12,
                color: "#aaa",
                lineHeight: 1.5,
              }}
            >
              {hint.summary}
            </p>
            {hint.details && (
              <p
                style={{
                  margin: "0 0 8px 0",
                  fontSize: 11,
                  color: "#888",
                  lineHeight: 1.5,
                }}
              >
                {hint.details}
              </p>
            )}
            <div style={{ marginTop: 8 }}>
              <div
                style={{
                  fontSize: 11,
                  color: "#666",
                  marginBottom: 4,
                  fontWeight: 600,
                }}
              >
                Suggested Actions:
              </div>
              <ul
                style={{
                  margin: 0,
                  paddingLeft: 20,
                  fontSize: 11,
                  color: "#aaa",
                  lineHeight: 1.6,
                }}
              >
                {hint.suggestedActions.map((action, idx) => (
                  <li key={idx}>{action}</li>
                ))}
              </ul>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

