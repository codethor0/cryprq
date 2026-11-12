// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

import { useEffect, useRef, useState, useCallback } from "react";
import { DebugConsole } from "./DebugConsole";
import { EncryptionStatus } from "./EncryptionStatus";
import { ConnectionHealth } from "./ConnectionHealth";
import { GuidedTests } from "./GuidedTests";
import { TestSummary } from "./TestSummary";
import { ConnectionProfiles, type ConnectionProfile } from "./ConnectionProfiles";
import { CLISnippet } from "./CLISnippet";
import { AdvancedMetrics } from "./AdvancedMetrics";
import { BugReportDraft } from "./BugReportDraft";
import { DiagnosticHints } from "./DiagnosticHints";
import { TimelineView } from "./TimelineView";
import { ScenarioScoreboard } from "./ScenarioScoreboard";
import { SessionHistoryPanel } from "./SessionHistoryPanel";
import { SessionComparePanel } from "./SessionComparePanel";
import { ScenarioPlaybookPanel } from "./ScenarioPlaybookPanel";
import { RegressionGuardPanel } from "./RegressionGuardPanel";
import { ValidationPacketPanel } from "./ValidationPacketPanel";
import { buildSessionSnapshot, buildSessionTimeline } from "./sessionSnapshot";
import { loadStoredSessions, type StoredSession } from "./sessionHistory";
import type {
  ScenarioId,
  ScenarioRecord,
} from "./guidedScenarios";
import {
  findScenarioExpectation,
  evaluateScenario,
} from "./guidedScenarios";

type Mode = "listener" | "dialer";

type Status = {
  connected: boolean;
  encryption: "libp2p transport encryption (test mode)" | "None";
  peerId?: string;
  connectedPeer?: string;
  keyEpoch?: number;
  rotationInterval?: number;
  mode?: "listener" | "dialer";
};

export default function App() {
  const [mode, setMode] = useState<Mode>("listener");
  const [port, setPort] = useState<number>(10000);
  const [peer, setPeer] = useState<string>("/ip4/127.0.0.1/udp/10000/quic-v1");
  const [vpnMode, setVpnMode] = useState<boolean>(false);
  const [events, setEvents] = useState<
    { t: string; level: "status" | "rotation" | "peer" | "info" | "error" }[]
  >([]);
  const [connecting, setConnecting] = useState(false);
  const [serverConnected, setServerConnected] = useState(false);
  const [fileTransferStatus, setFileTransferStatus] = useState<string>("");
  const [fileTransferProgress, setFileTransferProgress] = useState<number>(0);
  
  // File transfer state machine
  type FileTransferState = "idle" | "preparing" | "in_progress" | "completed" | "failed";
  const [fileTransferState, setFileTransferState] = useState<FileTransferState>("idle");
  const [fileTransferFileName, setFileTransferFileName] = useState<string>("");
  const [fileTransferSize, setFileTransferSize] = useState<number>(0);
  const [fileTransferStartTime, setFileTransferStartTime] = useState<number | null>(null);
  const [fileTransferEndTime, setFileTransferEndTime] = useState<number | null>(null);
  const [status, setStatus] = useState<Status>({
    connected: false,
    encryption: "libp2p transport encryption (test mode)", // Encryption is ALWAYS enabled in CrypRQ
    mode: undefined, // Will be set when listener/dialer starts
  });
  const esRef = useRef<EventSource | null>(null);

  // Validation state
  const [portError, setPortError] = useState<string>("");
  const [peerError, setPeerError] = useState<string>("");
  const [vpnError, setVpnError] = useState<string>("");

  // Connection health state
  const [connectionHealth, setConnectionHealth] = useState<{
    connectionState: "idle" | "connecting" | "encrypted-tunnel-active";
    lastHandshakeDirection?: "inbound" | "outbound";
    lastHandshakeTime?: string;
    lastPingLatency?: string;
    currentEpoch?: number;
    rotationIntervalSecs?: number;
  }>({
    connectionState: "idle",
  });

  // Debug console filter state
  const [logFilter, setLogFilter] = useState<
    "all" | "errors" | "rotation" | "peer" | "status"
  >("all");

  // Scenario scoreboard state
  const [scenarios, setScenarios] = useState<Map<ScenarioId, ScenarioRecord>>(
    new Map()
  );

  // Session history state
  const [storedSessions, setStoredSessions] = useState<StoredSession[]>([]);
  const [compareLeftId, setCompareLeftId] = useState<string | undefined>();
  const [compareRightId, setCompareRightId] = useState<string | undefined>();

  useEffect(() => {
    setStoredSessions(loadStoredSessions());
  }, []);

  const handleSelectForCompare = (id: string, side: "left" | "right") => {
    if (side === "left") {
      setCompareLeftId(id);
    } else {
      setCompareRightId(id);
    }
  };

  // Validation functions
  const validatePort = (portValue: number): string => {
    if (isNaN(portValue) || portValue < 1 || portValue > 65535) {
      return "Port must be between 1 and 65535";
    }
    return "";
  };

  const validatePeerAddress = (peerValue: string): string => {
    if (mode === "dialer") {
      if (!peerValue || peerValue.trim() === "") {
        return "Peer address is required for Dialer mode";
      }
      // Basic pattern check - must contain /udp/ and end with /quic-v1
      const looksLikeQuicAddr =
        peerValue.includes("/udp/") && peerValue.endsWith("/quic-v1");
      if (!looksLikeQuicAddr) {
        return "Peer address should be a QUIC multiaddr (e.g., /ip4/127.0.0.1/udp/10000/quic-v1)";
      }
    }
    return "";
  };

  const isFormValid = (): boolean => {
    const portValid = validatePort(port) === "";
    const peerValid = validatePeerAddress(peer) === "";
    return portValid && peerValid;
  };

  // Auto-update peer address when port changes
  useEffect(() => {
    if (mode === "dialer") {
      setPeer(`/ip4/127.0.0.1/udp/${port}/quic-v1`);
    }
  }, [port, mode]);

  // Validate port on change
  useEffect(() => {
    setPortError(validatePort(port));
  }, [port]);

  // Validate peer address on change
  useEffect(() => {
    setPeerError(validatePeerAddress(peer));
  }, [peer, mode]);

  useEffect(() => {
    if (esRef.current) esRef.current.close();
    const es = new EventSource("/events");
    es.onopen = () => {
      setServerConnected(true);
      setEvents((prev) => [
        ...prev,
        { t: "[CONNECTED] Event stream connected", level: "status" },
      ]);
    };
    es.onerror = (err) => {
      setServerConnected(false);
      // EventSource errors are normal during reconnection - don't spam UI
      // Only log to console for debugging
      console.error("EventSource error (will auto-reconnect):", err);
    };
    es.onmessage = (m) => {
      try {
        const e = JSON.parse(m.data);
        setEvents((prev) => [...prev, e]);

        // Parse status from events
        const text = e.t || "";

        // Extract peer ID - CRITICAL: This indicates encryption is active
        if (
          text.includes("Local peer id:") ||
          text.match(/Local peer id:\s*(\S+)/i)
        ) {
          const match = text.match(/Local peer id:\s*(\S+)/i);
          if (match) {
            const peerId = match[1];
            setStatus((prev) => ({
              ...prev,
              peerId: peerId,
              encryption: "libp2p transport encryption (test mode)", // Encryption is active when peer ID exists
              mode: mode,
            }));
            // Log encryption proof
            console.log(
              "[TRANSPORT] Peer ID generated (libp2p transport identity; no CrypRQ handshake in this preview)",
            );
          }
        }

        // CRITICAL: Connection established patterns from actual CrypRQ logs
        // Pattern 1: "Connected to {peer_id}" - dialer successfully connected
        if (text.match(/Connected to\s+(\S+)/i)) {
          const match = text.match(/Connected to\s+(\S+)/i);
          const peerId = match ? match[1] : null;
          setStatus((prev) => ({
            ...prev,
            connected: true,
            connectedPeer: peerId || prev.connectedPeer,
            encryption: "libp2p transport encryption (test mode)",
            mode: mode,
          }));
          setConnectionHealth((prev) => ({
            ...prev,
            connectionState: "encrypted-tunnel-active",
            lastHandshakeDirection: "outbound",
            lastHandshakeTime: new Date().toLocaleTimeString(),
          }));
        }

        // Pattern 2: "Inbound connection established with {peer_id}" - listener received connection
        if (text.match(/Inbound connection established/i)) {
          const match = text.match(
            /Inbound connection established with\s+(\S+)/i,
          );
          const peerId = match ? match[1] : null;
          setStatus((prev) => ({
            ...prev,
            connected: true,
            connectedPeer: peerId || prev.connectedPeer,
            encryption: "libp2p transport encryption (test mode)",
            mode: mode,
          }));
          setConnectionHealth((prev) => ({
            ...prev,
            connectionState: "encrypted-tunnel-active",
            lastHandshakeDirection: "inbound",
            lastHandshakeTime: new Date().toLocaleTimeString(),
          }));
        }

        // Pattern 3: "Listening on {address}" - listener is ready (encryption active, waiting for peer)
        if (text.match(/Listening on\s+/i)) {
          setStatus((prev) => ({
            ...prev,
            mode: "listener",
            encryption: "libp2p transport encryption (test mode)", // Encryption is always active
            // Listener is ready but not connected until peer dials
            // Keep connected status if already connected
          }));
        }

        // Pattern 4: ConnectionEstablished events (from libp2p)
        if (
          text.match(/ConnectionEstablished|connection established/i) &&
          !text.match(/Inbound/i)
        ) {
          setStatus((prev) => ({
            ...prev,
            connected: true,
            encryption: "libp2p transport encryption (test mode)",
            mode: mode,
          }));
        }

        // Pattern 5: Key rotation events indicate encryption is active - CRITICAL PROOF
        if (
          text.match(/key_rotation|rotation/i) &&
          text.match(/success|epoch/i)
        ) {
          const epochMatch = text.match(/epoch=(\d+)/);
          const intervalMatch = text.match(/interval_secs=(\d+)/);
          setStatus((prev) => ({
            ...prev,
            encryption: "libp2p transport encryption (test mode)",
            keyEpoch: epochMatch ? parseInt(epochMatch[1]) : prev.keyEpoch,
            rotationInterval: intervalMatch
              ? parseInt(intervalMatch[1])
              : prev.rotationInterval,
          }));
          // Log encryption proof
          console.log(
            "[TRANSPORT] Local rotation timer event (does not key the traffic of this preview)",
          );
        }

        // Pattern 6: Encryption/decryption events confirm encryption is working
        if (text.match(/ENCRYPT|DECRYPT/i)) {
          setStatus((prev) => ({
            ...prev,
            connected: prev.connected || prev.peerId !== undefined, // If we have peer ID and encryption events, we're connected
            encryption: "libp2p transport encryption (test mode)",
          }));
        }

        // Pattern 7: Post-quantum encryption enabled
        if (text.match(/Post-quantum|post-quantum|ML-KEM|Kyber/i)) {
          setStatus((prev) => ({
            ...prev,
            encryption: "libp2p transport encryption (test mode)",
          }));
        }

        // Pattern 8 removed - handled in Pattern 10 above

        // Pattern 9: Starting listener - encryption is ALWAYS active in CrypRQ
        if (text.match(/Starting listener/i)) {
          setStatus((prev) => ({
            ...prev,
            mode: "listener",
            encryption: "libp2p transport encryption (test mode)", // Encryption is always active
            // Don't set connected=true yet, but encryption is active
          }));
          console.log(
            "[TRANSPORT] Listener starting on the legacy test-mode path (libp2p transport encryption only)",
          );
        }

        // Pattern 10: Dialing peer - encryption is ALWAYS active in CrypRQ
        if (text.match(/Dialing peer/i)) {
          setStatus((prev) => ({
            ...prev,
            mode: "dialer",
            encryption: "libp2p transport encryption (test mode)", // Encryption is always active
            // Don't set connected=true yet, but encryption is active
          }));
        }

        // Pattern 11: Process started - encryption is active
        if (text.match(/Process started|spawn.*CrypRQ/i)) {
          setStatus((prev) => ({
            ...prev,
            encryption: "libp2p transport encryption (test mode)", // Encryption is always active
          }));
        }

        // Extract key rotation info - indicates encryption is active
        if (text.match(/key_rotation/i) && text.match(/epoch=(\d+)/)) {
          const epochMatch = text.match(/epoch=(\d+)/);
          const intervalMatch = text.match(/interval_secs=(\d+)/);
          if (epochMatch) {
            const epoch = parseInt(epochMatch[1]);
            const interval = intervalMatch ? parseInt(intervalMatch[1]) : undefined;
            setStatus((prev) => ({
              ...prev,
              keyEpoch: epoch,
              rotationInterval: interval || prev.rotationInterval,
              encryption: "libp2p transport encryption (test mode)", // Key rotation = encryption active
            }));
            setConnectionHealth((prev) => ({
              ...prev,
              currentEpoch: epoch,
              rotationIntervalSecs: interval || prev.rotationIntervalSecs,
            }));
          }
        }

        // Extract ping latency - format: "Ping event: Event { peer: ..., connection: ..., result: Ok(123.542µs) }"
        if (text.match(/Ping event.*result:\s*Ok\(([\d.]+)(µs|ms|s)\)/i)) {
          const pingMatch = text.match(/result:\s*Ok\(([\d.]+)(µs|ms|s)\)/i);
          if (pingMatch) {
            setConnectionHealth((prev) => ({
              ...prev,
              lastPingLatency: `${pingMatch[1]}${pingMatch[2]}`,
            }));
          }
        }

        // Update connection state based on handshake_complete events
        if (text.match(/event=handshake_complete/i)) {
          const directionMatch = text.match(/direction=(inbound|outbound)/i);
          if (directionMatch) {
            setConnectionHealth((prev) => ({
              ...prev,
              connectionState: "encrypted-tunnel-active",
              lastHandshakeDirection: directionMatch[1].toLowerCase() as "inbound" | "outbound",
              lastHandshakeTime: new Date().toLocaleTimeString(),
            }));
          }
        }

        // Check for VPN privilege errors - only show once to avoid spam
        if (
          text.match(
            /requires root|requires admin|privileges|Failed to create TUN/i,
          )
        ) {
          setVpnError(
            "System-wide VPN mode failed: TUN device not available or admin privileges required. Your encrypted P2P tunnel remains available. Use P2P file transfer or run the CLI with sudo on a system with TUN support for full VPN routing.",
          );
          setEvents((prev) => {
            // Check if we already showed this error
            const alreadyShown = prev.some((e) =>
              e.t.includes("VPN mode requires administrator privileges"),
            );
            if (!alreadyShown) {
              return [
                ...prev,
                {
                  t: `VPN mode requires administrator privileges. Run with sudo or use P2P mode only. P2P encryption works without admin privileges.`,
                  level: "error",
                },
              ];
            }
            return prev;
          });
        }

        // Parse file transfer events from logs
        if (text.match(/FILE_TRANSFER|file transfer|File transfer/i)) {
          // Look for file transfer completion or failure
          if (text.match(/sent successfully|sent securely|completed/i)) {
            if (fileTransferState === "in_progress" || fileTransferState === "preparing") {
              setFileTransferState("completed");
              if (!fileTransferEndTime) {
                setFileTransferEndTime(Date.now());
              }
              setFileTransferProgress(100);
            }
          } else if (text.match(/failed|error|Error/i) && text.match(/file|File/i)) {
            if (fileTransferState === "in_progress" || fileTransferState === "preparing") {
              setFileTransferState("failed");
              if (!fileTransferEndTime) {
                setFileTransferEndTime(Date.now());
              }
              setFileTransferProgress(0);
            }
          }
        }

        // Check for disconnection - only mark disconnected on fatal errors
        if (text.match(/exit\s+[12]|ConnectionClosed|connection closed/i)) {
          if (!text.match(/Address already in use/i)) {
            // Only disconnect on actual errors, not port conflicts
            if (text.match(/fatal|FATAL|panic|error/i)) {
              setStatus((prev) => ({
                ...prev,
                connected: false,
                connectedPeer: undefined,
              }));
            }
          }
        }
      } catch (error) {
        // EventSource errors are handled above - don't add error messages to UI
        // EventSource will auto-reconnect automatically
        console.debug("EventSource error (handled):", error);
      }
    };
    esRef.current = es;
    return () => {
      es.close();
    };
  }, [mode]);

  // Scenario evaluation effect
  useEffect(() => {
    const currentTime = Date.now();
    setScenarios((prev: Map<ScenarioId, ScenarioRecord>) => {
      const updated = new Map(prev);
      let changed = false;

      for (const [id, record] of updated.entries()) {
        if (record.status === "running") {
          const expectation = findScenarioExpectation(id);
          if (expectation) {
            const result = evaluateScenario(record, expectation, events, currentTime);
            const newStatus = result.status as ScenarioRecord["status"];
            if (result.status !== record.status) {
              updated.set(id, {
                ...record,
                status: newStatus,
                details: result.details,
                lastRunAt: newStatus === "passed" || newStatus === "failed" ? new Date().toISOString() : record.lastRunAt,
              });
              changed = true;
            } else if (newStatus === "running" && result.details !== record.details) {
              updated.set(id, {
                ...record,
                details: result.details,
              });
              changed = true;
            }
          }
        }
      }

      return changed ? updated : prev;
    });
  }, [events]);

  // Scenario management functions
  const startScenario = useCallback((id: ScenarioId) => {
    setScenarios((prev: Map<ScenarioId, ScenarioRecord>) => {
      const updated = new Map(prev);
      updated.set(id, {
        id,
        status: "running",
        startTime: Date.now(),
        matchedPatterns: new Set(),
      });
      return updated;
    });
  }, []);

  const handleFileUpload = async (
    event: React.ChangeEvent<HTMLInputElement>,
  ) => {
    const file = event.target.files?.[0];
    if (!file) return;

    // Check if connection is established - either status.connected OR we have peer ID/listening status
    const isConnected =
      status.connected ||
      status.peerId ||
      status.mode === "listener" ||
      status.mode === "dialer";
    if (!isConnected) {
      setFileTransferState("failed");
      setFileTransferStatus(
        "Error: Not connected to peer. Please connect first.",
      );
      return;
    }

    // Initialize file transfer state
    setFileTransferState("preparing");
    setFileTransferFileName(file.name);
    setFileTransferSize(file.size);
    setFileTransferStartTime(Date.now());
    setFileTransferEndTime(null);
    setFileTransferStatus("Reading file...");
    setFileTransferProgress(0);

    const reader = new FileReader();
    reader.onload = async (e) => {
      try {
        const fileContent = e.target?.result as string;
        setFileTransferState("in_progress");
        setFileTransferStatus("Sending file through encrypted tunnel...");
        setFileTransferProgress(50);

        const res = await fetch("/api/send-file", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            filename: file.name,
            content: fileContent,
            size: file.size,
            type: file.type,
          }),
        });

        if (!res.ok) {
          const errorText = await res.text();
          throw new Error(`HTTP ${res.status}: ${errorText}`);
        }

        const data = await res.json();
        const endTime = Date.now();
        setFileTransferEndTime(endTime);
        
        if (data.success) {
          setFileTransferState("completed");
          setFileTransferStatus(
            `File "${file.name}" sent successfully through encrypted tunnel`,
          );
          setFileTransferProgress(100);
          setEvents((prev) => [
            ...prev,
            {
              t: `[FILE TRANSFER] File "${file.name}" (${(file.size / 1024).toFixed(2)} KB) sent securely`,
              level: "info",
            },
          ]);
        } else {
          setFileTransferState("failed");
          setFileTransferStatus(
            `Error: ${data.message || "Failed to send file"}`,
          );
          setFileTransferProgress(0);
        }
      } catch (err) {
        setFileTransferState("failed");
        setFileTransferEndTime(Date.now());
        setFileTransferStatus(
          `Error: ${err instanceof Error ? err.message : "Failed to send file"}`,
        );
        setFileTransferProgress(0);
      }
    };
    reader.readAsDataURL(file);
  };

  async function connect() {
    setConnecting(true);
    setConnectionHealth((prev) => ({
      ...prev,
      connectionState: "connecting",
    }));
    // Set encryption active immediately when connect is initiated - CrypRQ always uses encryption
    setStatus((prev) => ({
      ...prev,
      mode: mode,
      encryption: "libp2p transport encryption (test mode)", // Encryption is always active
    }));
    try {
      const res = await fetch("/connect", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ mode, port, peer, vpn: vpnMode }),
      });
      if (!res.ok) {
        const err = await res.text();
        setEvents((prev) => [
          ...prev,
          { t: `Connect failed: ${err}`, level: "error" },
        ]);
      } else {
        const data = await res.json();
        const vpnMsg = vpnMode ? " (VPN mode: system-wide routing)" : "";
        setEvents((prev) => [
          ...prev,
          {
            t: `Connect initiated: ${mode} on port ${port}${vpnMsg}`,
            level: "status",
          },
        ]);
        // Ensure encryption status is set after successful connect
        setStatus((prev) => ({
          ...prev,
          mode: mode,
          encryption: "libp2p transport encryption (test mode)",
        }));
        if (data.dockerMode) {
          setEvents((prev) => [
            ...prev,
            {
              t: `[DOCKER] Docker mode: Container ${data.containerName || "cryprq-vpn"}`,
              level: "status",
            },
          ]);
          if (data.containerIP) {
            setEvents((prev) => [
              ...prev,
              { t: `Container IP: ${data.containerIP}`, level: "status" },
            ]);
          }
        }
        // For Docker mode, if we got container info, mark as connected
        if (data.dockerMode && data.containerIP) {
          setStatus((prev) => ({
            ...prev,
            mode: mode,
            connected: true,
            encryption: "libp2p transport encryption (test mode)",
          }));
        } else {
          // Keep encryption active - CrypRQ always uses encryption
          setStatus((prev) => ({
            ...prev,
            mode: mode,
            connected: false,
            encryption: "libp2p transport encryption (test mode)", // Ensure encryption is set
          }));
        }
      }
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setEvents((prev) => [
        ...prev,
        { t: `Connect error: ${errorMessage}`, level: "error" },
      ]);
    } finally {
      setConnecting(false);
    }
  }

  return (
    <div
      data-testid="app-root"
      style={{
        paddingBottom: 270,
        padding: 24,
        maxWidth: 1200,
        margin: "0 auto",
        background: "linear-gradient(to bottom, #0a0a0a, #1a1a1a)",
        minHeight: "100vh",
        color: "#e0e0e0",
        fontFamily:
          '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
      }}
    >
      {/* Header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          marginBottom: 24,
          paddingBottom: 16,
          borderBottom: "2px solid #333",
        }}
      >
        <h1
          data-testid="app-title"
          style={{
            display: "flex",
            alignItems: "center",
            gap: 12,
            margin: 0,
            fontSize: 28,
            fontWeight: 600,
            color: "#fff",
          }}
        >
          <img
            src="/icon_master_1024.png"
            alt="CrypRQ Icon"
            style={{
              width: 40,
              height: 40,
              borderRadius: 8,
              boxShadow: "0 2px 8px rgba(0,0,0,0.3)",
            }}
            onError={(e) => {
              e.currentTarget.style.display = "none";
            }}
          />
          CrypRQ Web Tester
        </h1>
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 8,
            fontSize: 14,
            color: serverConnected ? "#4f4" : "#f55",
          }}
        >
          <div
            style={{
              width: 8,
              height: 8,
              borderRadius: "50%",
              background: serverConnected ? "#4f4" : "#f55",
              boxShadow: serverConnected ? "0 0 8px #4f4" : "none",
            }}
          />
          {serverConnected ? "Connected" : "Disconnected"}
        </div>
      </div>

      {/* CLI Snippet */}
      <CLISnippet
        mode={mode}
        port={port}
        peerAddress={peer}
        vpnEnabled={vpnMode}
      />

      {/* Connection Profiles */}
      <ConnectionProfiles
        currentMode={mode}
        currentPort={port}
        currentPeer={peer}
        currentVpnMode={vpnMode}
        onLoadProfile={(profile: ConnectionProfile) => {
          setMode(profile.mode);
          setPort(profile.port);
          setPeer(profile.peerAddress);
          setVpnMode(profile.vpnEnabled);
        }}
      />

      {/* Guided Tests */}
      <GuidedTests
        onSetMode={setMode}
        onSetPort={setPort}
        onSetPeer={setPeer}
        onSetVpnMode={setVpnMode}
        currentMode={mode}
      />
      <ScenarioScoreboard scenarios={scenarios} onStartScenario={startScenario} />
      <SessionHistoryPanel
        currentSnapshot={buildSessionSnapshot({
          mode,
          port,
          peer,
          vpnMode,
          connectionHealth,
          events,
          status,
          fileTransferState,
          fileTransferFileName,
          fileTransferSize,
          fileTransferStartTime,
          fileTransferEndTime,
        })}
        currentTimeline={buildSessionTimeline(
          buildSessionSnapshot({
            mode,
            port,
            peer,
            vpnMode,
            connectionHealth,
            events,
            status,
            fileTransferState,
            fileTransferFileName,
            fileTransferSize,
            fileTransferStartTime,
            fileTransferEndTime,
          }),
          events
        )}
        onSelectForCompare={handleSelectForCompare}
      />
      <SessionComparePanel
        sessions={storedSessions}
        leftSelectedId={compareLeftId}
        rightSelectedId={compareRightId}
        onLeftSelect={setCompareLeftId}
        onRightSelect={setCompareRightId}
      />
      <ScenarioPlaybookPanel
        currentSnapshot={buildSessionSnapshot({
          mode,
          port,
          peer,
          vpnMode,
          connectionHealth,
          events,
          status,
          fileTransferState,
          fileTransferFileName,
          fileTransferSize,
          fileTransferStartTime,
          fileTransferEndTime,
        })}
        currentTimeline={buildSessionTimeline(
          buildSessionSnapshot({
            mode,
            port,
            peer,
            vpnMode,
            connectionHealth,
            events,
            status,
            fileTransferState,
            fileTransferFileName,
            fileTransferSize,
            fileTransferStartTime,
            fileTransferEndTime,
          }),
          events
        )}
        storedSessions={storedSessions}
      />
      <RegressionGuardPanel
        currentSnapshot={buildSessionSnapshot({
          mode,
          port,
          peer,
          vpnMode,
          connectionHealth,
          events,
          status,
          fileTransferState,
          fileTransferFileName,
          fileTransferSize,
          fileTransferStartTime,
          fileTransferEndTime,
        })}
        currentTimeline={buildSessionTimeline(
          buildSessionSnapshot({
            mode,
            port,
            peer,
            vpnMode,
            connectionHealth,
            events,
            status,
            fileTransferState,
            fileTransferFileName,
            fileTransferSize,
            fileTransferStartTime,
            fileTransferEndTime,
          }),
          events
        )}
        storedSessions={storedSessions}
      />
      <ValidationPacketPanel
        currentSnapshot={buildSessionSnapshot({
          mode,
          port,
          peer,
          vpnMode,
          connectionHealth,
          events,
          status,
          fileTransferState,
          fileTransferFileName,
          fileTransferSize,
          fileTransferStartTime,
          fileTransferEndTime,
        })}
        currentTimeline={buildSessionTimeline(
          buildSessionSnapshot({
            mode,
            port,
            peer,
            vpnMode,
            connectionHealth,
            events,
            status,
            fileTransferState,
            fileTransferFileName,
            fileTransferSize,
            fileTransferStartTime,
            fileTransferEndTime,
          }),
          events
        )}
        storedSessions={storedSessions}
      />

      {/* Controls */}
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
            display: "grid",
            gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))",
            gap: 16,
            marginBottom: 16,
          }}
        >
          <div>
            <label
              style={{
                display: "block",
                marginBottom: 8,
                fontSize: 12,
                fontWeight: 600,
                color: "#aaa",
                textTransform: "uppercase",
                letterSpacing: 0.5,
              }}
            >
              Mode
            </label>
            <select
              value={mode}
              onChange={(e) => {
                const normalizedValue = e.target.value.toLowerCase() as Mode;
                if (normalizedValue === "listener" || normalizedValue === "dialer") {
                  setMode(normalizedValue);
                }
              }}
              style={{
                width: "100%",
                padding: "10px 12px",
                background: "#0a0a0a",
                border: "1px solid #444",
                borderRadius: 6,
                color: "#fff",
                fontSize: 14,
                cursor: "pointer",
              }}
            >
              <option value="listener">Listener</option>
              <option value="dialer">Dialer</option>
            </select>
          </div>

          <div>
            <label
              style={{
                display: "block",
                marginBottom: 8,
                fontSize: 12,
                fontWeight: 600,
                color: "#aaa",
                textTransform: "uppercase",
                letterSpacing: 0.5,
              }}
            >
              Port
            </label>
            <input
              type="number"
              value={port}
              onChange={(e) => {
                const newPort = parseInt(e.target.value || "0");
                setPort(newPort);
              }}
              onBlur={() => setPortError(validatePort(port))}
              min={1}
              max={65535}
              style={{
                width: "100%",
                padding: "10px 12px",
                background: "#0a0a0a",
                border: portError ? "1px solid #f55" : "1px solid #444",
                borderRadius: 6,
                color: "#fff",
                fontSize: 14,
              }}
            />
            {portError && (
              <div
                style={{
                  fontSize: 11,
                  color: "#f55",
                  marginTop: 4,
                  marginLeft: 2,
                }}
              >
                {portError}
              </div>
            )}
          </div>

          <div style={{ gridColumn: mode === "dialer" ? "span 1" : "span 2" }}>
            <label
              style={{
                display: "block",
                marginBottom: 8,
                fontSize: 12,
                fontWeight: 600,
                color: "#aaa",
                textTransform: "uppercase",
                letterSpacing: 0.5,
              }}
            >
              Peer Address
            </label>
            <input
              style={{
                width: "100%",
                padding: "10px 12px",
                background: "#0a0a0a",
                border: peerError ? "1px solid #f55" : "1px solid #444",
                borderRadius: 6,
                color: "#fff",
                fontSize: 14,
                fontFamily: "ui-monospace, monospace",
              }}
              value={peer}
              onChange={(e) => setPeer(e.target.value)}
              onBlur={() => setPeerError(validatePeerAddress(peer))}
              placeholder="/ip4/127.0.0.1/udp/10000/quic-v1"
              required={mode === "dialer"}
            />
            {peerError && (
              <div
                style={{
                  fontSize: 11,
                  color: "#f55",
                  marginTop: 4,
                  marginLeft: 2,
                }}
              >
                {peerError}
              </div>
            )}
          </div>
        </div>

        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 16,
            flexWrap: "wrap",
          }}
        >
          <div>
            <label
              style={{
                display: "flex",
                alignItems: "center",
                gap: 8,
                cursor: "pointer",
                fontSize: 14,
                marginBottom: vpnMode ? 4 : 0,
              }}
            >
              <input
                type="checkbox"
                checked={vpnMode}
                onChange={(e) => setVpnMode(e.target.checked)}
                style={{
                  width: 18,
                  height: 18,
                  cursor: "pointer",
                }}
              />
              <span>VPN Mode (system-wide routing)</span>
            </label>
            {vpnMode && (
              <div
                style={{
                  fontSize: 11,
                  color: "#ff8",
                  marginLeft: 26,
                  marginTop: 4,
                  lineHeight: 1.4,
                }}
              >
                VPN mode requires admin/root + TUN support. In this Docker/web demo, VPN mode may fail, but the P2P encrypted tunnel remains available.
              </div>
            )}
            {vpnError && (
              <div
                style={{
                  fontSize: 11,
                  color: "#f55",
                  marginLeft: 26,
                  marginTop: 4,
                  lineHeight: 1.4,
                  padding: "8px",
                  background: "#2a0a0a",
                  borderRadius: 4,
                  border: "1px solid #f55",
                }}
              >
                {vpnError}
              </div>
            )}
          </div>

          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: 8,
              marginTop: 8,
            }}
          >
            <label
              style={{
                display: "flex",
                alignItems: "center",
                gap: 8,
                cursor: status.connected ? "pointer" : "not-allowed",
                fontSize: 14,
                padding: "8px 12px",
                border: "1px solid #444",
                borderRadius: 4,
                backgroundColor: status.connected ? "#2a5" : "#555",
                opacity: status.connected ? 1 : 0.6,
              }}
            >
              <input
                type="file"
                onChange={handleFileUpload}
                disabled={!status.connected}
                style={{ display: "none" }}
                id="file-upload"
              />
              <span>
                {status.connected
                  ? "Send File Securely"
                  : "Connect first to send files"}
              </span>
            </label>
            {fileTransferStatus && (
              <div
                style={{
                  fontSize: 12,
                  color:
                    fileTransferState === "completed"
                      ? "#2f5"
                      : fileTransferState === "failed"
                        ? "#f55"
                        : fileTransferState === "in_progress" || fileTransferState === "preparing"
                          ? "#ff8"
                          : "#aaa",
                  padding: "4px 8px",
                  backgroundColor: "#222",
                  borderRadius: 4,
                }}
              >
                {fileTransferStatus}
                {fileTransferState === "completed" && fileTransferStartTime && fileTransferEndTime && (
                  <div style={{ fontSize: 11, color: "#888", marginTop: 4 }}>
                    {fileTransferFileName && fileTransferSize > 0 && (
                      <>
                        {((fileTransferSize / 1024).toFixed(2))} KB in{" "}
                        {((fileTransferEndTime - fileTransferStartTime) / 1000).toFixed(2)}s
                        {fileTransferEndTime - fileTransferStartTime > 0 && (
                          <>
                            {" "}(~
                            {(
                              (fileTransferSize * 8) /
                              ((fileTransferEndTime - fileTransferStartTime) / 1000) /
                              1000000
                            ).toFixed(2)}{" "}
                            Mbps)
                          </>
                        )}
                      </>
                    )}
                  </div>
                )}
                {fileTransferProgress > 0 && fileTransferProgress < 100 && (
                  <div
                    style={{
                      width: "100%",
                      height: 4,
                      backgroundColor: "#333",
                      borderRadius: 2,
                      marginTop: 4,
                      overflow: "hidden",
                    }}
                  >
                    <div
                      style={{
                        width: `${fileTransferProgress}%`,
                        height: "100%",
                        backgroundColor: "#2a5",
                        transition: "width 0.3s",
                      }}
                    />
                  </div>
                )}
              </div>
            )}
          </div>

          <button
            data-testid="connect-button"
            onClick={connect}
            disabled={connecting || !serverConnected || !isFormValid()}
            style={{
              padding: "12px 24px",
              background:
                connecting || !serverConnected || !isFormValid()
                  ? "#444"
                  : "linear-gradient(135deg, #667eea 0%, #764ba2 100%)",
              border: "none",
              borderRadius: 8,
              color: "#fff",
              fontSize: 14,
              fontWeight: 600,
              cursor:
                connecting || !serverConnected || !isFormValid()
                  ? "not-allowed"
                  : "pointer",
              transition: "all 0.2s",
              boxShadow:
                connecting || !serverConnected || !isFormValid()
                  ? "none"
                  : "0 4px 12px rgba(102, 126, 234, 0.4)",
            }}
          >
            {connecting ? "Connecting..." : "Connect"}
          </button>
        </div>
      </div>

      {/* Instructions */}
      <div
        style={{
          background: "#1a1a1a",
          border: "1px solid #333",
          borderRadius: 8,
          padding: 16,
          marginBottom: 20,
          fontSize: 14,
          color: "#aaa",
          lineHeight: 1.6,
        }}
      >
        <strong style={{ color: "#fff", display: "block", marginBottom: 8 }}>
          How to Test:
        </strong>
        <ol style={{ margin: 0, paddingLeft: 20 }}>
          <li>
            Open this page in{" "}
            <strong style={{ color: "#59f" }}>two browser tabs</strong>
          </li>
          <li>
            Tab 1: Set mode to{" "}
            <strong style={{ color: "#4f4" }}>Listener</strong> and click
            Connect
          </li>
          <li>
            Tab 2: Set mode to <strong style={{ color: "#59f" }}>Dialer</strong>{" "}
            and click Connect
          </li>
          <li>Watch the connection status and debug console below</li>
        </ol>
      </div>

      <EncryptionStatus status={status} />
      <ConnectionHealth health={connectionHealth} />
      <AdvancedMetrics
        events={events}
        fileTransferState={fileTransferState}
        status={status}
      />
      <TestSummary
        events={events}
        connectionHealth={connectionHealth}
        status={status}
        fileTransfer={{
          state: fileTransferState,
          fileName: fileTransferFileName,
          fileSize: fileTransferSize,
          startTime: fileTransferStartTime,
          endTime: fileTransferEndTime,
        }}
      />
      <DiagnosticHints
        summary={buildSessionSnapshot({
          mode,
          port,
          peer,
          vpnMode,
          connectionHealth,
          events,
          status,
          fileTransferState,
          fileTransferFileName,
          fileTransferSize,
          fileTransferStartTime,
          fileTransferEndTime,
        }).diagnostics}
      />
      <TimelineView
        timeline={buildSessionTimeline(
          buildSessionSnapshot({
            mode,
            port,
            peer,
            vpnMode,
            connectionHealth,
            events,
            status,
            fileTransferState,
            fileTransferFileName,
            fileTransferSize,
            fileTransferStartTime,
            fileTransferEndTime,
          }),
          events
        )}
      />
      <BugReportDraft
        sessionSnapshot={buildSessionSnapshot({
          mode,
          port,
          peer,
          vpnMode,
          connectionHealth,
          events,
          status,
          fileTransferState,
          fileTransferFileName,
          fileTransferSize,
          fileTransferStartTime,
          fileTransferEndTime,
        })}
      />
      <DebugConsole
        events={events}
        filter={logFilter}
        onFilterChange={setLogFilter}
        mode={mode}
        port={port}
        vpnMode={vpnMode}
        snapshot={buildSessionSnapshot({
          mode,
          port,
          peer,
          vpnMode,
          connectionHealth,
          events,
          status,
          fileTransferState,
          fileTransferFileName,
          fileTransferSize,
          fileTransferStartTime,
          fileTransferEndTime,
        })}
      />
    </div>
  );
}
