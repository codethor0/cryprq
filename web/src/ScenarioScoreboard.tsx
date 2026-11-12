// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// License: MIT (see LICENSE file for details)

import type { ScenarioRecord, ScenarioId } from "./guidedScenarios";
import { SCENARIO_EXPECTATIONS } from "./guidedScenarios";

type ScenarioScoreboardProps = {
  scenarios: Map<ScenarioId, ScenarioRecord>;
  onStartScenario?: (id: ScenarioId) => void;
};

export function ScenarioScoreboard({
  scenarios,
  onStartScenario,
}: ScenarioScoreboardProps) {
  const getStatusColor = (status: string): string => {
    switch (status) {
      case "passed":
        return "#4f4";
      case "failed":
        return "#f55";
      case "running":
        return "#ff8";
      default:
        return "#888";
    }
  };

  const getStatusLabel = (status: string): string => {
    switch (status) {
      case "passed":
        return "PASSED";
      case "failed":
        return "FAILED";
      case "running":
        return "RUNNING";
      default:
        return "NOT RUN";
    }
  };

  const formatTimestamp = (timestamp?: string): string => {
    if (!timestamp) return "—";
    try {
      const date = new Date(timestamp);
      return date.toLocaleTimeString();
    } catch {
      return timestamp;
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
        Scenario Scoreboard
      </h3>

      <div
        style={{
          overflowX: "auto",
          border: "1px solid #333",
          borderRadius: 6,
          background: "#0a0a0a",
        }}
      >
        <table
          style={{
            width: "100%",
            borderCollapse: "collapse",
            fontSize: 12,
          }}
        >
          <thead>
            <tr
              style={{
                background: "#1a1a1a",
                borderBottom: "1px solid #333",
              }}
            >
              <th
                style={{
                  padding: "10px 12px",
                  textAlign: "left",
                  color: "#aaa",
                  fontWeight: 600,
                  fontSize: 11,
                  textTransform: "uppercase",
                }}
              >
                Scenario
              </th>
              <th
                style={{
                  padding: "10px 12px",
                  textAlign: "left",
                  color: "#aaa",
                  fontWeight: 600,
                  fontSize: 11,
                  textTransform: "uppercase",
                }}
              >
                Status
              </th>
              <th
                style={{
                  padding: "10px 12px",
                  textAlign: "left",
                  color: "#aaa",
                  fontWeight: 600,
                  fontSize: 11,
                  textTransform: "uppercase",
                }}
              >
                Last Run
              </th>
              <th
                style={{
                  padding: "10px 12px",
                  textAlign: "left",
                  color: "#aaa",
                  fontWeight: 600,
                  fontSize: 11,
                  textTransform: "uppercase",
                }}
              >
                Details
              </th>
              {onStartScenario && (
                <th
                  style={{
                    padding: "10px 12px",
                    textAlign: "left",
                    color: "#aaa",
                    fontWeight: 600,
                    fontSize: 11,
                    textTransform: "uppercase",
                  }}
                >
                  Action
                </th>
              )}
            </tr>
          </thead>
          <tbody>
            {SCENARIO_EXPECTATIONS.map((expectation) => {
              const record = scenarios.get(expectation.id) || {
                id: expectation.id,
                status: "not_run" as const,
              };

              return (
                <tr
                  key={expectation.id}
                  style={{
                    borderBottom: "1px solid #222",
                  }}
                >
                  <td
                    style={{
                      padding: "10px 12px",
                      color: "#fff",
                      fontWeight: 500,
                    }}
                  >
                    <div style={{ marginBottom: 4 }}>{expectation.title}</div>
                    <div
                      style={{
                        fontSize: 10,
                        color: "#888",
                        lineHeight: 1.4,
                      }}
                    >
                      {expectation.description}
                    </div>
                    {expectation.classificationNotes && (
                      <div
                        style={{
                          fontSize: 9,
                          color: "#666",
                          marginTop: 4,
                          fontStyle: "italic",
                        }}
                      >
                        {expectation.classificationNotes}
                      </div>
                    )}
                  </td>
                  <td style={{ padding: "10px 12px" }}>
                    <span
                      style={{
                        padding: "4px 8px",
                        background: getStatusColor(record.status),
                        color: "#000",
                        borderRadius: 4,
                        fontSize: 10,
                        fontWeight: 600,
                      }}
                    >
                      {getStatusLabel(record.status)}
                    </span>
                  </td>
                  <td
                    style={{
                      padding: "10px 12px",
                      color: "#888",
                      fontFamily: "ui-monospace, monospace",
                      fontSize: 11,
                    }}
                  >
                    {formatTimestamp(record.lastRunAt)}
                  </td>
                  <td
                    style={{
                      padding: "10px 12px",
                      color: "#aaa",
                      fontSize: 11,
                      maxWidth: 300,
                      wordBreak: "break-word",
                    }}
                  >
                    {record.details || "—"}
                  </td>
                  {onStartScenario && (
                    <td style={{ padding: "10px 12px" }}>
                      <button
                        onClick={() => onStartScenario(expectation.id)}
                        disabled={record.status === "running"}
                        style={{
                          padding: "4px 8px",
                          fontSize: 10,
                          background:
                            record.status === "running" ? "#1a1a1a" : "#333",
                          border: "1px solid #555",
                          borderRadius: 4,
                          color:
                            record.status === "running" ? "#666" : "#fff",
                          cursor:
                            record.status === "running"
                              ? "not-allowed"
                              : "pointer",
                          fontWeight: 500,
                        }}
                        onMouseOver={(e) => {
                          if (record.status !== "running") {
                            e.currentTarget.style.background = "#444";
                          }
                        }}
                        onMouseOut={(e) => {
                          if (record.status !== "running") {
                            e.currentTarget.style.background = "#333";
                          }
                        }}
                      >
                        {record.status === "running" ? "Running..." : "Start"}
                      </button>
                    </td>
                  )}
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}

