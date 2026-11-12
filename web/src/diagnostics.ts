// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// License: MIT (see LICENSE file for details)

export type DiagnosticSeverity = "env_limitation" | "product_bug" | "info" | "unknown";

export interface DiagnosticHint {
  id: string;
  title: string;
  severity: DiagnosticSeverity;
  summary: string;
  details?: string;
  logPattern: RegExp;
  suggestedActions: string[];
  classifyAs?: "env_limitation" | "product_bug";
}

export interface DiagnosticSummary {
  matchedHints: DiagnosticHint[];
  countsBySeverity: {
    env_limitation: number;
    product_bug: number;
    info: number;
    unknown: number;
  };
  rawMatches: { hintId: string; eventIndex: number }[];
}

const KNOWN_HINTS: DiagnosticHint[] = [
  {
    id: "handshake_timeout_no_listener",
    title: "Handshake timeout - Listener may not be running",
    severity: "info",
    summary: "Connection attempt timed out during handshake",
    details: "The dialer attempted to connect but did not receive a response from the listener. This usually means the listener is not running, the address/port is incorrect, or there is a firewall blocking UDP traffic.",
    logPattern: /HandshakeTimedOut|Dial error.*Transport.*HandshakeTimedOut/i,
    suggestedActions: [
      "Verify Listener tab is running and connected first",
      "Check that port and address are identical in Listener and Dialer",
      "Check firewall/UDP permissions for the chosen port",
      "Ensure both tabs are using the same port number",
    ],
  },
  {
    id: "vpn_tun_failure_privileges",
    title: "VPN TUN interface failed - Missing admin/root privileges",
    severity: "env_limitation",
    summary: "Failed to create TUN device (requires root/admin privileges)",
    details: "VPN mode requires system-level privileges to create a TUN interface. This is expected in Docker/web environments and does not indicate a product bug.",
    logPattern: /Failed to create TUN (interface|device)|requires root\/admin privileges/i,
    suggestedActions: [
      "Run CLI with sudo or admin privileges for VPN mode",
      "Use P2P mode (file transfer only) when TUN is not available",
      "VPN mode is optional; encrypted P2P tunnel works without it",
    ],
    classifyAs: "env_limitation",
  },
  {
    id: "ssl_ca_cert_issue",
    title: "SSL CA certificate issue - Environment limitation",
    severity: "env_limitation",
    summary: "Problem with SSL CA certificate configuration",
    details: "This is typically an environment configuration issue (missing CA bundle, network restrictions) and does not indicate a problem with QUIC/crypto.",
    logPattern: /error setting certificate verify locations|Problem with the SSL CA cert/i,
    suggestedActions: [
      "Install or configure system CA bundle",
      "Treat this as environment-only; it does not mean QUIC/crypto is failing",
    ],
    classifyAs: "env_limitation",
  },
  {
    id: "npm_eperm",
    title: "npm permission error - Environment limitation",
    severity: "env_limitation",
    summary: "npm operation not permitted",
    details: "This is a file system permission issue in the build environment, not a product bug.",
    logPattern: /EPERM: operation not permitted/i,
    suggestedActions: [
      "Check npm permissions or use a different user/workspace",
      "This is an environment issue, not a CrypRQ bug",
    ],
    classifyAs: "env_limitation",
  },
  {
    id: "connection_closed_unexpectedly",
    title: "Connection closed unexpectedly",
    severity: "info",
    summary: "Connection was closed before completion",
    details: "The connection was established but closed before the operation completed. This could be due to network issues, timeout, or the remote peer closing the connection.",
    logPattern: /ConnectionClosed|connection closed|Connection closed/i,
    suggestedActions: [
      "Check network stability",
      "Verify the remote peer is still running",
      "Review connection logs for more details",
    ],
  },
  {
    id: "address_in_use",
    title: "Port already in use",
    severity: "info",
    summary: "The selected port is already in use by another process",
    details: "Another application or CrypRQ instance is already using this port.",
    logPattern: /Address already in use|address already in use/i,
    suggestedActions: [
      "Choose a different port number",
      "Close the other application using this port",
      "Check if another CrypRQ instance is running",
    ],
  },
];

export function classifyLogEvents(
  rawEvents: Array<{ raw?: string; t?: string }>
): DiagnosticSummary {
  const matchedHintsMap = new Map<string, DiagnosticHint>();
  const rawMatches: { hintId: string; eventIndex: number }[] = [];
  const countsBySeverity = {
    env_limitation: 0,
    product_bug: 0,
    info: 0,
    unknown: 0,
  };

  for (let i = 0; i < rawEvents.length; i++) {
    const event = rawEvents[i];
    const rawText = event.raw || event.t || "";

    for (const hint of KNOWN_HINTS) {
      if (hint.logPattern.test(rawText)) {
        if (!matchedHintsMap.has(hint.id)) {
          matchedHintsMap.set(hint.id, hint);
          const severity = hint.classifyAs || hint.severity;
          if (severity === "env_limitation") {
            countsBySeverity.env_limitation++;
          } else if (severity === "product_bug") {
            countsBySeverity.product_bug++;
          } else if (severity === "info") {
            countsBySeverity.info++;
          } else {
            countsBySeverity.unknown++;
          }
        }
        rawMatches.push({ hintId: hint.id, eventIndex: i });
      }
    }
  }

  return {
    matchedHints: Array.from(matchedHintsMap.values()),
    countsBySeverity,
    rawMatches,
  };
}

