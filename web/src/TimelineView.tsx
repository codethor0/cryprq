// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// License: MIT (see LICENSE file for details)

import { useState } from "react";
import type { SessionTimeline } from "./sessionSnapshot";

type TimelineViewProps = {
  timeline?: SessionTimeline;
};

type EventFilter = "all" | "status" | "peer" | "rotation" | "file" | "error" | "unknown";

export function TimelineView({ timeline }: TimelineViewProps) {
  const [filter, setFilter] = useState<EventFilter>("all");

  if (!timeline || timeline.events.length === 0) {
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
          Session Timeline
        </h3>
        <p
          style={{
            margin: 0,
            fontSize: 13,
            color: "#888",
            fontStyle: "italic",
          }}
        >
          No timeline events available yet.
        </p>
      </div>
    );
  }

  const filteredEvents = timeline.events.filter((e) => {
    if (filter === "all") return true;
    return e.kind === filter;
  });

  const displayEvents = filteredEvents.slice(-500);
  const hasMore = filteredEvents.length > 500;

  const formatTime = (timestamp: string): string => {
    try {
      const date = new Date(timestamp);
      if (isNaN(date.getTime())) {
        return timestamp.length > 8 ? timestamp.substring(11, 19) : timestamp;
      }
      return date.toLocaleTimeString();
    } catch {
      return timestamp.length > 8 ? timestamp.substring(11, 19) : timestamp;
    }
  };

  const getKindColor = (kind: string): string => {
    switch (kind) {
      case "error":
        return "#f55";
      case "rotation":
        return "#f90";
      case "peer":
        return "#59f";
      case "status":
        return "#8ff";
      case "file":
        return "#4f4";
      default:
        return "#888";
    }
  };

  const getKindLabel = (kind: string): string => {
    return kind.charAt(0).toUpperCase() + kind.slice(1);
  };

  const filters: EventFilter[] = ["all", "status", "peer", "rotation", "file", "error", "unknown"];

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
        Session Timeline
      </h3>

      <div
        style={{
          display: "flex",
          gap: 8,
          flexWrap: "wrap",
          marginBottom: 16,
          paddingBottom: 12,
          borderBottom: "1px solid #333",
        }}
      >
        {filters.map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            style={{
              padding: "4px 8px",
              fontSize: 11,
              background: filter === f ? "#333" : "#1a1a1a",
              border: "1px solid #444",
              borderRadius: 4,
              color: filter === f ? "#fff" : "#888",
              cursor: "pointer",
              textTransform: "capitalize",
            }}
            onMouseOver={(e) => {
              if (filter !== f) {
                e.currentTarget.style.background = "#222";
              }
            }}
            onMouseOut={(e) => {
              if (filter !== f) {
                e.currentTarget.style.background = "#1a1a1a";
              }
            }}
          >
            {f} ({f === "all" ? timeline.events.length : timeline.events.filter((e) => e.kind === f).length})
          </button>
        ))}
      </div>

      {hasMore && (
        <div
          style={{
            marginBottom: 12,
            padding: 8,
            background: "#0a0a0a",
            borderRadius: 6,
            fontSize: 11,
            color: "#888",
            textAlign: "center",
          }}
        >
          Showing latest 500 of {filteredEvents.length} events
        </div>
      )}

      <div
        style={{
          maxHeight: 400,
          overflowY: "auto",
          border: "1px solid #333",
          borderRadius: 6,
          background: "#0a0a0a",
        }}
      >
        <table
          style={{
            width: "100%",
            borderCollapse: "collapse",
            fontSize: 11,
          }}
        >
          <thead>
            <tr
              style={{
                background: "#1a1a1a",
                borderBottom: "1px solid #333",
                position: "sticky",
                top: 0,
                zIndex: 10,
              }}
            >
              <th
                style={{
                  padding: "8px 12px",
                  textAlign: "left",
                  color: "#aaa",
                  fontWeight: 600,
                  fontSize: 10,
                  textTransform: "uppercase",
                }}
              >
                Time
              </th>
              <th
                style={{
                  padding: "8px 12px",
                  textAlign: "left",
                  color: "#aaa",
                  fontWeight: 600,
                  fontSize: 10,
                  textTransform: "uppercase",
                }}
              >
                Kind
              </th>
              <th
                style={{
                  padding: "8px 12px",
                  textAlign: "left",
                  color: "#aaa",
                  fontWeight: 600,
                  fontSize: 10,
                  textTransform: "uppercase",
                }}
              >
                Message
              </th>
            </tr>
          </thead>
          <tbody>
            {displayEvents.length === 0 ? (
              <tr>
                <td
                  colSpan={3}
                  style={{
                    padding: 40,
                    textAlign: "center",
                    color: "#666",
                    fontStyle: "italic",
                  }}
                >
                  No events match the "{filter}" filter.
                </td>
              </tr>
            ) : (
              displayEvents.map((event, idx) => (
                <tr
                  key={idx}
                  style={{
                    borderBottom: "1px solid #222",
                    background: event.kind === "error" ? "#1a0a0a" : "transparent",
                  }}
                >
                  <td
                    style={{
                      padding: "6px 12px",
                      color: "#888",
                      fontFamily: "ui-monospace, monospace",
                      fontSize: 10,
                    }}
                  >
                    {formatTime(event.timestamp)}
                  </td>
                  <td
                    style={{
                      padding: "6px 12px",
                    }}
                  >
                    <span
                      style={{
                        padding: "2px 6px",
                        background: getKindColor(event.kind),
                        color: "#000",
                        borderRadius: 3,
                        fontSize: 9,
                        fontWeight: 600,
                      }}
                    >
                      {getKindLabel(event.kind)}
                    </span>
                  </td>
                  <td
                    style={{
                      padding: "6px 12px",
                      color: event.kind === "error" ? "#f55" : "#aaa",
                      wordBreak: "break-word",
                    }}
                  >
                    {event.message || event.raw || "<no message>"}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}

