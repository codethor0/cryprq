// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// License: MIT (see LICENSE file for details)

import { useState, useEffect } from "react";
import type { SessionSnapshot, SessionTimeline } from "./sessionSnapshot";
import {
  loadStoredSessions,
  addStoredSession,
  updateSessionLabel,
  deleteStoredSession,
  isLocalStorageAvailable,
  markSessionAsBaseline,
  clearBaselineFlag,
  type StoredSession,
} from "./sessionHistory";

type SessionHistoryPanelProps = {
  currentSnapshot?: SessionSnapshot;
  currentTimeline?: SessionTimeline;
  onSelectForCompare?: (id: string, side: "left" | "right") => void;
};

export function SessionHistoryPanel({
  currentSnapshot,
  currentTimeline,
  onSelectForCompare,
}: SessionHistoryPanelProps) {
  const [sessions, setSessions] = useState<StoredSession[]>([]);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editLabel, setEditLabel] = useState<string>("");
  const [localStorageAvailable, setLocalStorageAvailable] = useState(true);

  useEffect(() => {
    const available = isLocalStorageAvailable();
    setLocalStorageAvailable(available);
    if (available) {
      setSessions(loadStoredSessions());
    }
  }, []);

  const handleSave = () => {
    if (!currentSnapshot || !localStorageAvailable) {
      return;
    }
    const updated = addStoredSession({
      label: `Session ${new Date().toLocaleString()}`,
      snapshot: currentSnapshot,
      timeline: currentTimeline,
    });
    setSessions(updated);
    // Notify parent if callback exists
    if (onSelectForCompare) {
      // Trigger a refresh in parent
      setTimeout(() => {
        setSessions(loadStoredSessions());
      }, 100);
    }
  };

  const handleRename = (id: string, currentLabel: string) => {
    setEditingId(id);
    setEditLabel(currentLabel);
  };

  const handleRenameSave = (id: string) => {
    if (editLabel.trim()) {
      const updated = updateSessionLabel(id, editLabel.trim());
      setSessions(updated);
    }
    setEditingId(null);
    setEditLabel("");
  };

  const handleDelete = (id: string) => {
    if (confirm("Delete this session? This cannot be undone.")) {
      const updated = deleteStoredSession(id);
      setSessions(updated);
    }
  };

  const handleToggleBaseline = (session: StoredSession) => {
    if (session.isBaseline) {
      const updated = clearBaselineFlag(session.id);
      setSessions(updated);
    } else {
      const tag = prompt("Enter baseline tag (optional):", session.label || "baseline");
      if (tag !== null) {
        const updated = markSessionAsBaseline(session.id, tag || undefined);
        setSessions(updated);
      }
    }
  };

  const formatDate = (isoString: string): string => {
    try {
      return new Date(isoString).toLocaleString();
    } catch {
      return isoString;
    }
  };

  if (!localStorageAvailable) {
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
          Session History
        </h3>
        <p
          style={{
            margin: 0,
            fontSize: 13,
            color: "#f55",
          }}
        >
          Session history disabled: localStorage not available.
        </p>
      </div>
    );
  }

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
          Session History ({sessions.length})
        </h3>
        <button
          onClick={handleSave}
          disabled={!currentSnapshot}
          style={{
            padding: "6px 12px",
            background: currentSnapshot ? "#333" : "#1a1a1a",
            border: "1px solid #555",
            borderRadius: 6,
            color: currentSnapshot ? "#fff" : "#666",
            fontSize: 12,
            cursor: currentSnapshot ? "pointer" : "not-allowed",
            fontWeight: 500,
          }}
          onMouseOver={(e) => {
            if (currentSnapshot) {
              e.currentTarget.style.background = "#444";
            }
          }}
          onMouseOut={(e) => {
            if (currentSnapshot) {
              e.currentTarget.style.background = "#333";
            }
          }}
        >
          Save Current Session
        </button>
      </div>

      {sessions.length === 0 ? (
        <p
          style={{
            margin: 0,
            fontSize: 13,
            color: "#888",
            fontStyle: "italic",
          }}
        >
          No saved sessions yet. Save your current session to get started.
        </p>
      ) : (
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
                  Label
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
                  Created
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
                  Actions
                </th>
              </tr>
            </thead>
            <tbody>
              {sessions.map((session) => (
                <tr
                  key={session.id}
                  style={{
                    borderBottom: "1px solid #222",
                  }}
                >
                  <td
                    style={{
                      padding: "10px 12px",
                      color: "#fff",
                    }}
                  >
                    {editingId === session.id ? (
                      <input
                        type="text"
                        value={editLabel}
                        onChange={(e) => setEditLabel(e.target.value)}
                        onBlur={() => handleRenameSave(session.id)}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") {
                            handleRenameSave(session.id);
                          } else if (e.key === "Escape") {
                            setEditingId(null);
                            setEditLabel("");
                          }
                        }}
                        autoFocus
                        style={{
                          padding: "4px 8px",
                          background: "#0a0a0a",
                          border: "1px solid #555",
                          borderRadius: 4,
                          color: "#fff",
                          fontSize: 12,
                          width: "100%",
                        }}
                      />
                    ) : (
                      <span
                        onClick={() => handleRename(session.id, session.label)}
                        style={{
                          cursor: "pointer",
                          textDecoration: "underline",
                          textDecorationStyle: "dotted",
                        }}
                        title="Click to rename"
                      >
                        {session.label}
                      </span>
                    )}
                  </td>
                  <td
                    style={{
                      padding: "10px 12px",
                      color: "#888",
                      fontFamily: "ui-monospace, monospace",
                      fontSize: 11,
                    }}
                  >
                    {formatDate(session.createdAt)}
                  </td>
                  <td
                    style={{
                      padding: "10px 12px",
                    }}
                  >
                    {session.isBaseline ? (
                      <span
                        style={{
                          padding: "2px 6px",
                          background: "#ff8",
                          color: "#000",
                          borderRadius: 3,
                          fontSize: 9,
                          fontWeight: 600,
                        }}
                      >
                        BASELINE
                        {session.baselineTag && `: ${session.baselineTag}`}
                      </span>
                    ) : (
                      <span
                        style={{
                          fontSize: 11,
                          color: "#666",
                        }}
                      >
                        —
                      </span>
                    )}
                  </td>
                  <td
                    style={{
                      padding: "10px 12px",
                    }}
                  >
                    <div
                      style={{
                        display: "flex",
                        gap: 6,
                        flexWrap: "wrap",
                      }}
                    >
                      {onSelectForCompare && (
                        <>
                          <button
                            onClick={() => onSelectForCompare(session.id, "left")}
                            style={{
                              padding: "4px 8px",
                              fontSize: 10,
                              background: "#333",
                              border: "1px solid #555",
                              borderRadius: 4,
                              color: "#fff",
                              cursor: "pointer",
                            }}
                            onMouseOver={(e) => {
                              e.currentTarget.style.background = "#444";
                            }}
                            onMouseOut={(e) => {
                              e.currentTarget.style.background = "#333";
                            }}
                          >
                            Select Left
                          </button>
                          <button
                            onClick={() => onSelectForCompare(session.id, "right")}
                            style={{
                              padding: "4px 8px",
                              fontSize: 10,
                              background: "#333",
                              border: "1px solid #555",
                              borderRadius: 4,
                              color: "#fff",
                              cursor: "pointer",
                            }}
                            onMouseOver={(e) => {
                              e.currentTarget.style.background = "#444";
                            }}
                            onMouseOut={(e) => {
                              e.currentTarget.style.background = "#333";
                            }}
                          >
                            Select Right
                          </button>
                        </>
                      )}
                      <button
                        onClick={() => handleToggleBaseline(session)}
                        style={{
                          padding: "4px 8px",
                          fontSize: 10,
                          background: session.isBaseline ? "#3a3a1a" : "#333",
                          border: "1px solid #555",
                          borderRadius: 4,
                          color: session.isBaseline ? "#ff8" : "#fff",
                          cursor: "pointer",
                        }}
                        onMouseOver={(e) => {
                          e.currentTarget.style.background = session.isBaseline ? "#4a4a2a" : "#444";
                        }}
                        onMouseOut={(e) => {
                          e.currentTarget.style.background = session.isBaseline ? "#3a3a1a" : "#333";
                        }}
                      >
                        {session.isBaseline ? "Remove Baseline" : "Mark Baseline"}
                      </button>
                      <button
                        onClick={() => handleDelete(session.id)}
                        style={{
                          padding: "4px 8px",
                          fontSize: 10,
                          background: "#3a1a1a",
                          border: "1px solid #555",
                          borderRadius: 4,
                          color: "#f55",
                          cursor: "pointer",
                        }}
                        onMouseOver={(e) => {
                          e.currentTarget.style.background = "#4a2a2a";
                        }}
                        onMouseOut={(e) => {
                          e.currentTarget.style.background = "#3a1a1a";
                        }}
                      >
                        Delete
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

