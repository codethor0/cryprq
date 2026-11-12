// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

import { useState } from "react";

type Mode = "listener" | "dialer";

type CLISnippetProps = {
  mode: Mode;
  port: number;
  peerAddress: string;
  vpnEnabled: boolean;
};

export function CLISnippet({
  mode,
  port,
  peerAddress,
  vpnEnabled,
}: CLISnippetProps) {
  const [copied, setCopied] = useState(false);

  const generateCommand = (): string => {
    if (mode === "listener") {
      const base = `cryprq --listen /ip4/0.0.0.0/udp/${port}/quic-v1`;
      return vpnEnabled ? `${base} --vpn` : base;
    } else {
      // Dialer mode
      if (!peerAddress || peerAddress.trim() === "") {
        return `cryprq --peer <PEER_ADDRESS>${vpnEnabled ? " --vpn" : ""}`;
      }
      const base = `cryprq --peer ${peerAddress}`;
      return vpnEnabled ? `${base} --vpn` : base;
    }
  };

  const handleCopy = async () => {
    const command = generateCommand();
    try {
      if (navigator.clipboard && navigator.clipboard.writeText) {
        await navigator.clipboard.writeText(command);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      }
    } catch (err) {
      // Clipboard API not available or failed; user can manually copy
      console.debug("Clipboard API not available:", err);
    }
  };

  const command = generateCommand();

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
        Equivalent CLI Command
      </h3>

      <div
        style={{
          display: "flex",
          gap: 8,
          alignItems: "flex-start",
        }}
      >
        <textarea
          readOnly
          value={command}
          style={{
            flex: 1,
            padding: "10px 12px",
            background: "#0a0a0a",
            border: "1px solid #444",
            borderRadius: 6,
            color: "#fff",
            fontSize: 12,
            fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace",
            resize: "none",
            minHeight: "60px",
            lineHeight: 1.5,
          }}
          onClick={(e) => {
            (e.target as HTMLTextAreaElement).select();
          }}
        />
        <button
          onClick={handleCopy}
          style={{
            padding: "10px 16px",
            background: copied ? "#2a5" : "#333",
            border: "1px solid #555",
            borderRadius: 6,
            color: "#fff",
            fontSize: 12,
            cursor: "pointer",
            fontWeight: 500,
            whiteSpace: "nowrap",
            transition: "background 0.2s",
          }}
        >
          {copied ? "Copied!" : "Copy Command"}
        </button>
      </div>

      {copied && (
        <div
          style={{
            fontSize: 11,
            color: "#2a5",
            marginTop: 8,
          }}
        >
          Command copied to clipboard
        </div>
      )}
    </div>
  );
}

