// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

import { useState, useEffect } from "react";

type Mode = "listener" | "dialer";

export type ConnectionProfile = {
  id: string;
  name: string;
  mode: Mode;
  port: number;
  peerAddress: string;
  vpnEnabled: boolean;
};

const STORAGE_KEY = "cryprq_connection_profiles_v1";

function loadProfiles(): ConnectionProfile[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (!stored) return [];
    const parsed = JSON.parse(stored);
    if (Array.isArray(parsed)) {
      return parsed;
    }
    return [];
  } catch (error) {
    console.warn("Failed to load profiles from localStorage:", error);
    return [];
  }
}

function saveProfiles(profiles: ConnectionProfile[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(profiles));
  } catch (error) {
    console.warn("Failed to save profiles to localStorage:", error);
  }
}

type ConnectionProfilesProps = {
  currentMode: Mode;
  currentPort: number;
  currentPeer: string;
  currentVpnMode: boolean;
  onLoadProfile: (profile: ConnectionProfile) => void;
};

export function ConnectionProfiles({
  currentMode,
  currentPort,
  currentPeer,
  currentVpnMode,
  onLoadProfile,
}: ConnectionProfilesProps) {
  const [profiles, setProfiles] = useState<ConnectionProfile[]>([]);
  const [showSaveForm, setShowSaveForm] = useState(false);
  const [profileName, setProfileName] = useState("");
  const [selectedProfileId, setSelectedProfileId] = useState<string>("");

  useEffect(() => {
    setProfiles(loadProfiles());
  }, []);

  const handleSaveProfile = () => {
    if (!profileName.trim()) {
      return;
    }

    const newProfile: ConnectionProfile = {
      id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      name: profileName.trim(),
      mode: currentMode,
      port: currentPort,
      peerAddress: currentPeer,
      vpnEnabled: currentVpnMode,
    };

    const updated = [...profiles, newProfile];
    setProfiles(updated);
    saveProfiles(updated);
    setProfileName("");
    setShowSaveForm(false);
  };

  const handleLoadProfile = () => {
    const profile = profiles.find((p) => p.id === selectedProfileId);
    if (profile) {
      onLoadProfile(profile);
    }
  };

  const handleDeleteProfile = (id: string) => {
    if (!confirm("Delete this profile?")) {
      return;
    }
    const updated = profiles.filter((p) => p.id !== id);
    setProfiles(updated);
    saveProfiles(updated);
    if (selectedProfileId === id) {
      setSelectedProfileId("");
    }
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
        Connection Profiles
      </h3>

      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: 12,
        }}
      >
        {/* Save Profile */}
        <div>
          {!showSaveForm ? (
            <button
              onClick={() => setShowSaveForm(true)}
              style={{
                padding: "8px 16px",
                background: "#333",
                border: "1px solid #555",
                borderRadius: 6,
                color: "#fff",
                fontSize: 12,
                cursor: "pointer",
                fontWeight: 500,
              }}
            >
              Save Current Config as Profile
            </button>
          ) : (
            <div
              style={{
                display: "flex",
                gap: 8,
                alignItems: "center",
              }}
            >
              <input
                type="text"
                value={profileName}
                onChange={(e) => setProfileName(e.target.value)}
                placeholder="Profile name"
                style={{
                  flex: 1,
                  padding: "8px 12px",
                  background: "#0a0a0a",
                  border: "1px solid #444",
                  borderRadius: 6,
                  color: "#fff",
                  fontSize: 12,
                }}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    handleSaveProfile();
                  } else if (e.key === "Escape") {
                    setShowSaveForm(false);
                    setProfileName("");
                  }
                }}
                autoFocus
              />
              <button
                onClick={handleSaveProfile}
                disabled={!profileName.trim()}
                style={{
                  padding: "8px 16px",
                  background: "#333",
                  border: "1px solid #555",
                  borderRadius: 6,
                  color: "#fff",
                  fontSize: 12,
                  cursor: profileName.trim() ? "pointer" : "not-allowed",
                  opacity: profileName.trim() ? 1 : 0.5,
                }}
              >
                Save
              </button>
              <button
                onClick={() => {
                  setShowSaveForm(false);
                  setProfileName("");
                }}
                style={{
                  padding: "8px 16px",
                  background: "#1a1a1a",
                  border: "1px solid #444",
                  borderRadius: 6,
                  color: "#aaa",
                  fontSize: 12,
                  cursor: "pointer",
                }}
              >
                Cancel
              </button>
            </div>
          )}
        </div>

        {/* Load Profile */}
        {profiles.length > 0 && (
          <div
            style={{
              display: "flex",
              gap: 8,
              alignItems: "center",
            }}
          >
            <select
              value={selectedProfileId}
              onChange={(e) => setSelectedProfileId(e.target.value)}
              style={{
                flex: 1,
                padding: "8px 12px",
                background: "#0a0a0a",
                border: "1px solid #444",
                borderRadius: 6,
                color: "#fff",
                fontSize: 12,
                cursor: "pointer",
              }}
            >
              <option value="">Select a profile...</option>
              {profiles.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name} ({p.mode}, port {p.port}
                  {p.vpnEnabled ? ", VPN" : ""})
                </option>
              ))}
            </select>
            <button
              onClick={handleLoadProfile}
              disabled={!selectedProfileId}
              style={{
                padding: "8px 16px",
                background: "#333",
                border: "1px solid #555",
                borderRadius: 6,
                color: "#fff",
                fontSize: 12,
                cursor: selectedProfileId ? "pointer" : "not-allowed",
                opacity: selectedProfileId ? 1 : 0.5,
              }}
            >
              Load
            </button>
            {selectedProfileId && (
              <button
                onClick={() => handleDeleteProfile(selectedProfileId)}
                style={{
                  padding: "8px 16px",
                  background: "#2a0a0a",
                  border: "1px solid #f55",
                  borderRadius: 6,
                  color: "#f55",
                  fontSize: 12,
                  cursor: "pointer",
                }}
              >
                Delete
              </button>
            )}
          </div>
        )}

        {profiles.length === 0 && (
          <div
            style={{
              fontSize: 11,
              color: "#888",
              fontStyle: "italic",
            }}
          >
            No saved profiles. Save your current configuration to create one.
          </div>
        )}
      </div>
    </div>
  );
}

