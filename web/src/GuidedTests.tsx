// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

type Mode = "listener" | "dialer";

type GuidedTestsProps = {
  onSetMode: (mode: Mode) => void;
  onSetPort: (port: number) => void;
  onSetPeer: (peer: string) => void;
  onSetVpnMode: (enabled: boolean) => void;
  currentMode: Mode;
};

export function GuidedTests({
  onSetMode,
  onSetPort,
  onSetPeer,
  onSetVpnMode,
  currentMode,
}: GuidedTestsProps) {
  const handleLocalLoopback = () => {
    onSetMode("listener");
    onSetPort(10000);
    if (currentMode === "dialer") {
      onSetPeer("/ip4/127.0.0.1/udp/10000/quic-v1");
    }
  };

  const handleTwoTabTest = () => {
    onSetPort(10000);
    onSetPeer("/ip4/127.0.0.1/udp/10000/quic-v1");
  };

  const handleVpnDemo = () => {
    onSetVpnMode(true);
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
        Guided Tests
      </h3>

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))",
          gap: 16,
        }}
      >
        {/* Local Loopback P2P Test */}
        <div
          style={{
            background: "#0a0a0a",
            border: "1px solid #444",
            borderRadius: 8,
            padding: 16,
          }}
        >
          <h4
            style={{
              marginTop: 0,
              marginBottom: 8,
              fontSize: 14,
              fontWeight: 600,
              color: "#59f",
            }}
          >
            Local Loopback P2P Test
          </h4>
          <p
            style={{
              margin: "0 0 12px 0",
              fontSize: 12,
              color: "#aaa",
              lineHeight: 1.5,
            }}
          >
            Use two browser tabs: Tab 1 as Listener, Tab 2 as Dialer using
            127.0.0.1.
          </p>
          <button
            onClick={handleLocalLoopback}
            style={{
              width: "100%",
              padding: "8px 12px",
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
            Apply Settings
          </button>
        </div>

        {/* Two-Tab Web Test */}
        <div
          style={{
            background: "#0a0a0a",
            border: "1px solid #444",
            borderRadius: 8,
            padding: 16,
          }}
        >
          <h4
            style={{
              marginTop: 0,
              marginBottom: 8,
              fontSize: 14,
              fontWeight: 600,
              color: "#4f4",
            }}
          >
            Two-Tab Web Test
          </h4>
          <div
            style={{
              margin: "0 0 12px 0",
              fontSize: 11,
              color: "#aaa",
              lineHeight: 1.6,
            }}
          >
            <div style={{ marginBottom: 4 }}>
              <strong>Tab 1:</strong> Set mode to Listener, port 10000, click
              Connect.
            </div>
            <div>
              <strong>Tab 2:</strong> Set mode to Dialer, port 10000, peer
              /ip4/127.0.0.1/udp/10000/quic-v1, click Connect.
            </div>
          </div>
          <button
            onClick={handleTwoTabTest}
            style={{
              width: "100%",
              padding: "8px 12px",
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
            Pre-fill Settings
          </button>
        </div>

        {/* VPN Demo */}
        <div
          style={{
            background: "#0a0a0a",
            border: "1px solid #444",
            borderRadius: 8,
            padding: 16,
          }}
        >
          <h4
            style={{
              marginTop: 0,
              marginBottom: 8,
              fontSize: 14,
              fontWeight: 600,
              color: "#ff8",
            }}
          >
            VPN Demo (Error Handling)
          </h4>
          <p
            style={{
              margin: "0 0 12px 0",
              fontSize: 12,
              color: "#aaa",
              lineHeight: 1.5,
            }}
          >
            VPN mode requires admin/root + TUN. Will fail in Docker/web
            environment, but demonstrates error handling. P2P encrypted tunnel
            still works without VPN mode.
          </p>
          <button
            onClick={handleVpnDemo}
            style={{
              width: "100%",
              padding: "8px 12px",
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
            Enable VPN Mode
          </button>
        </div>
      </div>
    </div>
  );
}

