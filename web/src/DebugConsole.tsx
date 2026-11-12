// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

import { useEffect, useRef } from "react";
import type { SessionSnapshot } from "./sessionSnapshot";
import { buildSessionTimeline } from "./sessionSnapshot";

type E = {
  t: string;
  level: "status" | "rotation" | "peer" | "info" | "error";
};

type Filter = "all" | "errors" | "rotation" | "peer" | "status";

type DebugConsoleProps = {
  events: E[];
  filter: Filter;
  onFilterChange: (filter: Filter) => void;
  mode?: "listener" | "dialer";
  port?: number;
  vpnMode?: boolean;
  snapshot?: SessionSnapshot;
};

export function DebugConsole({
  events,
  filter,
  onFilterChange,
  mode,
  port,
  vpnMode,
  snapshot,
}: DebugConsoleProps) {
  const consoleRef = useRef<HTMLDivElement>(null);

  const handleDownloadLogs = () => {
    if (events.length === 0) {
      return;
    }

    const timestamp = new Date().toISOString().replace(/[:.]/g, "-").slice(0, -5);
    const header = `CrypRQ Session Logs
Generated: ${new Date().toLocaleString()}
Mode: ${mode || "N/A"}
Port: ${port || "N/A"}
VPN Mode: ${vpnMode ? "Enabled" : "Disabled"}

--- Log Entries ---

`;

    const logContent = events.map((e) => `[${e.level.toUpperCase()}] ${e.t}`).join("\n");
    const fullContent = header + logContent;

    const blob = new Blob([fullContent], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `cryprq-session-logs-${timestamp}.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const handleDownloadTimeline = () => {
    if (!snapshot) {
      console.warn("Session snapshot not available for timeline export");
      return;
    }

    try {
      const timeline = buildSessionTimeline(snapshot, events);
      const jsonContent = JSON.stringify(timeline, null, 2);
      
      const timestamp = new Date().toISOString().replace(/[:.]/g, "-").slice(0, -5);
      const blob = new Blob([jsonContent], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `cryprq_timeline_${timestamp}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (err) {
      console.error("Failed to export timeline:", err);
    }
  };

  // Filter events based on selected filter
  const filteredEvents = events.filter((e) => {
    if (filter === "all") return true;
    if (filter === "errors") return e.level === "error";
    if (filter === "rotation") return e.level === "rotation";
    if (filter === "peer") return e.level === "peer";
    if (filter === "status") return e.level === "status";
    return true;
  });

  // Auto-scroll to bottom when new events arrive
  useEffect(() => {
    if (consoleRef.current) {
      consoleRef.current.scrollTop = consoleRef.current.scrollHeight;
    }
  }, [events]);

  return (
    <div
      ref={consoleRef}
      style={{
        position: "fixed",
        bottom: 0,
        left: 0,
        right: 0,
        height: 280,
        overflow: "auto",
        background: "#0a0a0a",
        color: "#ddd",
        fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace",
        fontSize: 11,
        borderTop: "2px solid #333",
        padding: "12px 16px",
        lineHeight: 1.5,
        boxShadow: "0 -4px 12px rgba(0,0,0,0.5)",
      }}
    >
      <div
        style={{
          position: "sticky",
          top: 0,
          background: "#0a0a0a",
          paddingBottom: 8,
          marginBottom: 8,
          borderBottom: "1px solid #222",
          zIndex: 10,
        }}
      >
        <div
          style={{
            fontSize: 10,
            fontWeight: 600,
            color: "#666",
            textTransform: "uppercase",
            letterSpacing: 1,
            marginBottom: 8,
          }}
        >
          Debug Console ({filteredEvents.length} of {events.length} events)
        </div>
        <div
          style={{
            display: "flex",
            gap: 8,
            flexWrap: "wrap",
            alignItems: "center",
          }}
        >
          {(["all", "errors", "rotation", "peer", "status"] as Filter[]).map(
            (f) => (
              <button
                key={f}
                onClick={() => onFilterChange(f)}
                style={{
                  padding: "4px 8px",
                  fontSize: 10,
                  background: filter === f ? "#333" : "#1a1a1a",
                  border: "1px solid #444",
                  borderRadius: 4,
                  color: filter === f ? "#fff" : "#888",
                  cursor: "pointer",
                  textTransform: "capitalize",
                }}
              >
                {f}
              </button>
            ),
          )}
          <button
            onClick={handleDownloadLogs}
            disabled={events.length === 0}
            style={{
              padding: "4px 8px",
              fontSize: 10,
              background: events.length === 0 ? "#1a1a1a" : "#333",
              border: "1px solid #444",
              borderRadius: 4,
              color: events.length === 0 ? "#666" : "#fff",
              cursor: events.length === 0 ? "not-allowed" : "pointer",
            }}
          >
            Download Logs
          </button>
          <button
            onClick={handleDownloadTimeline}
            disabled={!snapshot}
            style={{
              padding: "4px 8px",
              fontSize: 10,
              background: !snapshot ? "#1a1a1a" : "#333",
              border: "1px solid #444",
              borderRadius: 4,
              color: !snapshot ? "#666" : "#fff",
              cursor: !snapshot ? "not-allowed" : "pointer",
              marginLeft: "auto",
            }}
          >
            Download Timeline (JSON)
          </button>
        </div>
      </div>
      {filteredEvents.slice(-500).map((e, i) => (
        <div
          key={i}
          style={{
            color:
              e.level === "error"
                ? "#f55"
                : e.level === "rotation"
                  ? "#f90"
                  : e.level === "peer"
                    ? "#59f"
                    : e.level === "status"
                      ? "#8ff"
                      : "#8f8",
            opacity: e.level === "error" ? 1 : 0.85,
            marginBottom: 2,
            padding: "2px 0",
            wordBreak: "break-word",
          }}
        >
          <span style={{ color: "#666", marginRight: 8 }}>
            [{e.level.toUpperCase()}]
          </span>
          {e.t}
        </div>
      ))}
      {filteredEvents.length === 0 && events.length > 0 && (
        <div
          style={{
            color: "#666",
            fontStyle: "italic",
            textAlign: "center",
            padding: "40px 20px",
          }}
        >
          No events match the "{filter}" filter.
        </div>
      )}
      {events.length === 0 && (
        <div
          style={{
            color: "#666",
            fontStyle: "italic",
            textAlign: "center",
            padding: "40px 20px",
          }}
        >
          Waiting for events...
        </div>
      )}
    </div>
  );
}
