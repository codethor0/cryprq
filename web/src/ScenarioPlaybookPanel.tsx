// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// License: MIT (see LICENSE file for details)

import { useState, useEffect } from "react";
import type { SessionSnapshot, SessionTimeline } from "./sessionSnapshot";
import { buildScenarioPlaybookMarkdown, type ScenarioPlaybookContext } from "./sessionSnapshot";
import type { StoredSession } from "./sessionHistory";

type ScenarioPlaybookPanelProps = {
  currentSnapshot?: SessionSnapshot;
  currentTimeline?: SessionTimeline;
  storedSessions?: StoredSession[];
};

export function ScenarioPlaybookPanel({
  currentSnapshot,
  currentTimeline,
  storedSessions = [],
}: ScenarioPlaybookPanelProps) {
  const [sourceType, setSourceType] = useState<"current" | "stored">("current");
  const [selectedSessionId, setSelectedSessionId] = useState<string>("");
  const [label, setLabel] = useState<string>("");
  const [playbookMarkdown, setPlaybookMarkdown] = useState<string>("");

  useEffect(() => {
    updatePlaybook();
  }, [sourceType, selectedSessionId, label, currentSnapshot, currentTimeline, storedSessions]);

  const updatePlaybook = () => {
    let snapshot: SessionSnapshot | undefined;
    let timeline: SessionTimeline | undefined;
    let sessionLabel: string | undefined;
    let sourceSessionId: string | undefined;

    if (sourceType === "current") {
      snapshot = currentSnapshot;
      timeline = currentTimeline;
      sessionLabel = label || `Current Session @ ${new Date().toLocaleString()}`;
    } else if (selectedSessionId) {
      const stored = storedSessions.find((s) => s.id === selectedSessionId);
      if (stored) {
        snapshot = stored.snapshot;
        timeline = stored.timeline;
        sessionLabel = label || stored.label;
        sourceSessionId = stored.id;
      }
    }

    if (snapshot) {
      const ctx: ScenarioPlaybookContext = {
        snapshot,
        timeline,
        label: sessionLabel,
        sourceSessionId,
      };
      const markdown = buildScenarioPlaybookMarkdown(ctx);
      setPlaybookMarkdown(markdown);
    } else {
      setPlaybookMarkdown("");
    }
  };

  const handleCopy = () => {
    if (playbookMarkdown) {
      navigator.clipboard
        .writeText(playbookMarkdown)
        .then(() => {
          alert("Scenario playbook copied to clipboard");
        })
        .catch((err) => {
          console.error("Failed to copy playbook:", err);
          alert("Failed to copy playbook to clipboard");
        });
    }
  };

  const hasUsableSource = currentSnapshot || (sourceType === "stored" && selectedSessionId);

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
            Scenario Playbook
          </h3>
          <p
            style={{
              margin: 0,
              fontSize: 12,
              color: "#888",
            }}
          >
            Generate a reproducible scenario document from current or saved sessions
          </p>
        </div>
        <button
          onClick={handleCopy}
          disabled={!hasUsableSource || !playbookMarkdown}
          style={{
            padding: "6px 12px",
            background: hasUsableSource && playbookMarkdown ? "#333" : "#1a1a1a",
            border: "1px solid #555",
            borderRadius: 6,
            color: hasUsableSource && playbookMarkdown ? "#fff" : "#666",
            fontSize: 12,
            cursor: hasUsableSource && playbookMarkdown ? "pointer" : "not-allowed",
            fontWeight: 500,
          }}
          onMouseOver={(e) => {
            if (hasUsableSource && playbookMarkdown) {
              e.currentTarget.style.background = "#444";
            }
          }}
          onMouseOut={(e) => {
            if (hasUsableSource && playbookMarkdown) {
              e.currentTarget.style.background = "#333";
            }
          }}
        >
          Copy Playbook (Markdown)
        </button>
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
            Source
          </label>
          <div
            style={{
              display: "flex",
              gap: 16,
            }}
          >
            <label
              style={{
                display: "flex",
                alignItems: "center",
                gap: 6,
                fontSize: 12,
                color: "#fff",
                cursor: "pointer",
              }}
            >
              <input
                type="radio"
                checked={sourceType === "current"}
                onChange={() => setSourceType("current")}
                style={{ cursor: "pointer" }}
              />
              Use current session
            </label>
            <label
              style={{
                display: "flex",
                alignItems: "center",
                gap: 6,
                fontSize: 12,
                color: "#fff",
                cursor: "pointer",
              }}
            >
              <input
                type="radio"
                checked={sourceType === "stored"}
                onChange={() => setSourceType("stored")}
                disabled={storedSessions.length === 0}
                style={{ cursor: storedSessions.length > 0 ? "pointer" : "not-allowed" }}
              />
              Use stored session
            </label>
          </div>
        </div>

        {sourceType === "stored" && (
          <div>
            <label
              style={{
                display: "block",
                fontSize: 12,
                color: "#aaa",
                marginBottom: 6,
              }}
            >
              Select Session
            </label>
            <select
              value={selectedSessionId}
              onChange={(e) => setSelectedSessionId(e.target.value)}
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
              <option value="">-- Select a session --</option>
              {storedSessions.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.label} ({new Date(s.createdAt).toLocaleString()})
                </option>
              ))}
            </select>
          </div>
        )}

        <div>
          <label
            style={{
              display: "block",
              fontSize: 12,
              color: "#aaa",
              marginBottom: 6,
            }}
          >
            Scenario Label (optional)
          </label>
          <input
            type="text"
            value={label}
            onChange={(e) => setLabel(e.target.value)}
            placeholder={sourceType === "current" ? "Current Session @ ..." : "Enter label..."}
            style={{
              width: "100%",
              padding: "6px 8px",
              background: "#0a0a0a",
              border: "1px solid #555",
              borderRadius: 4,
              color: "#fff",
              fontSize: 12,
            }}
          />
        </div>
      </div>

      {!hasUsableSource ? (
        <p
          style={{
            margin: 0,
            fontSize: 13,
            color: "#888",
            fontStyle: "italic",
            textAlign: "center",
            padding: 40,
          }}
        >
          No session data available yet – run a test and/or save a session first.
        </p>
      ) : (
        <div>
          <label
            style={{
              display: "block",
              fontSize: 12,
              color: "#aaa",
              marginBottom: 6,
            }}
          >
            Playbook Preview
          </label>
          <textarea
            readOnly
            value={playbookMarkdown}
            style={{
              width: "100%",
              minHeight: 400,
              padding: 12,
              background: "#0a0a0a",
              border: "1px solid #555",
              borderRadius: 6,
              color: "#aaa",
              fontSize: 11,
              fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace",
              lineHeight: 1.6,
              resize: "vertical",
            }}
          />
        </div>
      )}
    </div>
  );
}

