// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// License: MIT (see LICENSE file for details)

import { useState, useEffect } from "react";
import type { SessionSnapshot, SessionTimeline } from "./sessionSnapshot";
import type { StoredSession } from "./sessionHistory";
import { getBaselines } from "./sessionHistory";
import { buildValidationPacketMarkdown, type ValidationPacketContext } from "./validationPacket";
import { buildScenarioPlaybookMarkdown, type ScenarioPlaybookContext } from "./sessionSnapshot";
import { evaluateRegression, type RegressionEvaluation } from "./regressionGuard";

type ValidationPacketPanelProps = {
  currentSnapshot?: SessionSnapshot;
  currentTimeline?: SessionTimeline;
  storedSessions: StoredSession[];
};

export function ValidationPacketPanel({
  currentSnapshot,
  currentTimeline,
  storedSessions,
}: ValidationPacketPanelProps) {
  const [sourceType, setSourceType] = useState<"current" | "stored">("current");
  const [selectedSessionId, setSelectedSessionId] = useState<string>("");
  const [selectedBaselineId, setSelectedBaselineId] = useState<string>("");
  const [includePlaybook, setIncludePlaybook] = useState<boolean>(true);
  const [includeBugReport, setIncludeBugReport] = useState<boolean>(false);
  const [label, setLabel] = useState<string>("");
  const [notes, setNotes] = useState<string>("");
  const [packetMarkdown, setPacketMarkdown] = useState<string>("");

  const baselines = getBaselines();
  const selectedSession = selectedSessionId
    ? storedSessions.find((s) => s.id === selectedSessionId)
    : undefined;
  const selectedBaseline = selectedBaselineId
    ? storedSessions.find((s) => s.id === selectedBaselineId)
    : undefined;

  useEffect(() => {
    updatePacket();
  }, [
    sourceType,
    selectedSessionId,
    selectedBaselineId,
    includePlaybook,
    includeBugReport,
    label,
    notes,
    currentSnapshot,
    currentTimeline,
    storedSessions,
  ]);

  const updatePacket = () => {
    let snapshot: SessionSnapshot | undefined;
    let timeline: SessionTimeline | undefined;
    let sessionLabel: string | undefined;

    if (sourceType === "current") {
      snapshot = currentSnapshot;
      timeline = currentTimeline;
      sessionLabel = label || `Current Session @ ${new Date().toLocaleString()}`;
    } else if (selectedSession) {
      snapshot = selectedSession.snapshot;
      timeline = selectedSession.timeline;
      sessionLabel = label || selectedSession.label;
    }

    if (!snapshot) {
      setPacketMarkdown("");
      return;
    }

    // Build regression evaluation if baseline is selected
    let regression: RegressionEvaluation | undefined;
    let baselineSnapshot: SessionSnapshot | undefined;
    let baselineLabel: string | undefined;
    let baselineCreatedAt: string | undefined;

    if (selectedBaseline) {
      baselineSnapshot = selectedBaseline.snapshot;
      baselineLabel = selectedBaseline.baselineTag || selectedBaseline.label;
      baselineCreatedAt = selectedBaseline.createdAt;
      regression = evaluateRegression(baselineSnapshot, snapshot, {
        baselineLabel,
        currentLabel: sessionLabel,
      });
    }

    // Build scenario playbook if requested
    let scenarioPlaybookMarkdown: string | undefined;
    if (includePlaybook && snapshot) {
      const playbookCtx: ScenarioPlaybookContext = {
        snapshot,
        timeline,
        label: sessionLabel,
        sourceSessionId: sourceType === "stored" ? selectedSessionId : undefined,
      };
      scenarioPlaybookMarkdown = buildScenarioPlaybookMarkdown(playbookCtx);
    }

    // Build validation packet context
    const ctx: ValidationPacketContext = {
      snapshot,
      timeline,
      baselineSnapshot,
      baselineLabel,
      baselineCreatedAt,
      regression,
      scenarioPlaybookMarkdown,
      bugReportDraftMarkdown: includeBugReport ? undefined : undefined, // TODO: wire in actual bug report draft if available
      label: sessionLabel,
      sourceType,
      notes: notes || undefined,
    };

    const markdown = buildValidationPacketMarkdown(ctx);
    setPacketMarkdown(markdown);
  };

  const handleCopy = () => {
    if (packetMarkdown) {
      navigator.clipboard
        .writeText(packetMarkdown)
        .then(() => {
          alert("Validation Packet copied to clipboard");
        })
        .catch((err) => {
          console.error("Failed to copy packet:", err);
          alert("Failed to copy packet to clipboard");
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
            Validation Packet (All-In-One Session Report)
          </h3>
          <p
            style={{
              margin: 0,
              fontSize: 12,
              color: "#888",
            }}
          >
            Generate a comprehensive Markdown document combining configuration, diagnostics, regression status, scenario playbook, and metrics
          </p>
        </div>
        <button
          onClick={handleCopy}
          disabled={!hasUsableSource || !packetMarkdown}
          style={{
            padding: "6px 12px",
            background: hasUsableSource && packetMarkdown ? "#333" : "#1a1a1a",
            border: "1px solid #555",
            borderRadius: 6,
            color: hasUsableSource && packetMarkdown ? "#fff" : "#666",
            fontSize: 12,
            cursor: hasUsableSource && packetMarkdown ? "pointer" : "not-allowed",
            fontWeight: 500,
          }}
          onMouseOver={(e) => {
            if (hasUsableSource && packetMarkdown) {
              e.currentTarget.style.background = "#444";
            }
          }}
          onMouseOut={(e) => {
            if (hasUsableSource && packetMarkdown) {
              e.currentTarget.style.background = "#333";
            }
          }}
        >
          Copy Validation Packet (Markdown)
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
            Source Session
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
              Select Stored Session
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
            Baseline (Optional)
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
            <option value="">-- No baseline selected --</option>
            {baselines.map((s) => (
              <option key={s.id} value={s.id}>
                {s.baselineTag || s.label} ({new Date(s.createdAt).toLocaleString()})
              </option>
            ))}
          </select>
        </div>

        <div>
          <label
            style={{
              display: "flex",
              alignItems: "center",
              gap: 8,
              fontSize: 12,
              color: "#fff",
              cursor: "pointer",
            }}
          >
            <input
              type="checkbox"
              checked={includePlaybook}
              onChange={(e) => setIncludePlaybook(e.target.checked)}
              style={{ cursor: "pointer" }}
            />
            Attach Scenario Playbook
          </label>
        </div>

        <div>
          <label
            style={{
              display: "flex",
              alignItems: "center",
              gap: 8,
              fontSize: 12,
              color: "#fff",
              cursor: "pointer",
            }}
          >
            <input
              type="checkbox"
              checked={includeBugReport}
              onChange={(e) => setIncludeBugReport(e.target.checked)}
              style={{ cursor: "pointer" }}
            />
            Attach Bug Report Draft (if available)
          </label>
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
            Packet Label (optional)
          </label>
          <input
            type="text"
            value={label}
            onChange={(e) => setLabel(e.target.value)}
            placeholder="e.g., 1.0.3-pre-release-smoke-run"
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

        <div>
          <label
            style={{
              display: "block",
              fontSize: 12,
              color: "#aaa",
              marginBottom: 6,
            }}
          >
            Notes (optional)
          </label>
          <textarea
            value={notes}
            onChange={(e) => setNotes(e.target.value)}
            placeholder="Additional context or observations..."
            rows={3}
            style={{
              width: "100%",
              padding: "6px 8px",
              background: "#0a0a0a",
              border: "1px solid #555",
              borderRadius: 4,
              color: "#fff",
              fontSize: 12,
              resize: "vertical",
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
            Validation Packet Preview
          </label>
          <textarea
            readOnly
            value={packetMarkdown}
            style={{
              width: "100%",
              minHeight: 500,
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

