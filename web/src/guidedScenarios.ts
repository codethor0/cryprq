// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// License: MIT (see LICENSE file for details)

export type ScenarioId = "local_loopback_p2p" | "two_tab_web" | "vpn_demo" | "p2p_file_transfer";

export interface ScenarioExpectation {
  id: ScenarioId;
  title: string;
  description: string;
  expectedPatterns: RegExp[];
  forbiddenErrorPatterns?: RegExp[];
  timeoutMs: number;
  classificationNotes?: string;
}

export const SCENARIO_EXPECTATIONS: ScenarioExpectation[] = [
  {
    id: "local_loopback_p2p",
    title: "Local Loopback P2P Test",
    description: "Two-tab test: Listener in Tab 1, Dialer in Tab 2 using 127.0.0.1",
    expectedPatterns: [
      /event=listener_ready|listener_ready|status=accepting_connections/i,
      /event=handshake_complete|handshake_complete|encryption=libp2p-transport/i,
      /event=connection_established|connection_established|encryption_active=true/i,
    ],
    forbiddenErrorPatterns: [
      /HandshakeTimedOut|Dial error.*Transport.*HandshakeTimedOut/i,
    ],
    timeoutMs: 30000,
    classificationNotes: "Expects listener_ready, handshake_complete, and connection_established events",
  },
  {
    id: "two_tab_web",
    title: "Two-Tab Web Test",
    description: "Tab 1 as Listener, Tab 2 as Dialer connecting to Tab 1",
    expectedPatterns: [
      /event=listener_ready|listener_ready|status=accepting_connections/i,
      /event=handshake_complete|handshake_complete|encryption=libp2p-transport/i,
      /event=connection_established|connection_established|encryption_active=true/i,
    ],
    forbiddenErrorPatterns: [
      /HandshakeTimedOut|Dial error.*Transport.*HandshakeTimedOut/i,
    ],
    timeoutMs: 30000,
    classificationNotes: "Same as local loopback but via web UI in two tabs",
  },
  {
    id: "vpn_demo",
    title: "VPN Demo (Error Handling)",
    description: "VPN mode requires admin/root + TUN. Demonstrates proper error handling when TUN is unavailable",
    expectedPatterns: [
      /VPN MODE ENABLED|VPN.*enabled|--vpn/i,
      /Failed to create TUN|requires root\/admin privileges|TUN.*not available/i,
    ],
    forbiddenErrorPatterns: [],
    timeoutMs: 10000,
    classificationNotes: "VPN ENV demo (TUN failure expected and classified as ENV LIMITATION)",
  },
  {
    id: "p2p_file_transfer",
    title: "P2P File Transfer",
    description: "File transfer over encrypted P2P tunnel",
    expectedPatterns: [
      /event=connection_established|connection_established|encryption_active=true/i,
      /file.*transfer|File.*transfer|transfer.*start/i,
      /transfer.*complete|transfer.*success|file.*sent/i,
    ],
    forbiddenErrorPatterns: [
      /transfer.*failed|file.*error|transfer.*error/i,
    ],
    timeoutMs: 60000,
    classificationNotes: "Expects connection establishment and file transfer completion",
  },
];

export type ScenarioStatus = "not_run" | "running" | "passed" | "failed";

export interface ScenarioRecord {
  id: ScenarioId;
  status: ScenarioStatus;
  lastRunAt?: string;
  details?: string;
  startTime?: number;
  matchedPatterns?: Set<string>;
}

export function findScenarioExpectation(id: ScenarioId): ScenarioExpectation | undefined {
  return SCENARIO_EXPECTATIONS.find((s) => s.id === id);
}

export function evaluateScenario(
  record: ScenarioRecord,
  expectation: ScenarioExpectation,
  events: Array<{ t: string; level?: string }>,
  currentTime: number
): { status: ScenarioStatus; details: string } {
  if (record.status !== "running" || !record.startTime) {
    return { status: record.status, details: record.details || "" };
  }

  const elapsed = currentTime - record.startTime;
  if (elapsed > expectation.timeoutMs) {
    return {
      status: "failed",
      details: `Timeout waiting for expected events (${expectation.timeoutMs}ms elapsed)`,
    };
  }

  const matchedPatterns = new Set<string>();
  let hasForbiddenError = false;

  for (const event of events) {
    const text = event.t || "";

    // Check forbidden error patterns
    if (expectation.forbiddenErrorPatterns) {
      for (const pattern of expectation.forbiddenErrorPatterns) {
        if (pattern.test(text)) {
          hasForbiddenError = true;
          break;
        }
      }
    }

    if (hasForbiddenError) {
      return {
        status: "failed",
        details: `Forbidden error pattern detected: ${text.substring(0, 100)}`,
      };
    }

    // Check expected patterns
    for (const pattern of expectation.expectedPatterns) {
      if (pattern.test(text)) {
        matchedPatterns.add(pattern.toString());
      }
    }
  }

  // Check if all expected patterns matched
  if (matchedPatterns.size === expectation.expectedPatterns.length) {
    return {
      status: "passed",
      details: `All expected patterns matched (${matchedPatterns.size}/${expectation.expectedPatterns.length})`,
    };
  }

  return {
    status: "running",
    details: `Waiting for patterns (matched ${matchedPatterns.size}/${expectation.expectedPatterns.length})`,
  };
}

