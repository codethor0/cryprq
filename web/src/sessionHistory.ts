// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// License: MIT (see LICENSE file for details)

import type { SessionSnapshot, SessionTimeline } from "./sessionSnapshot";

export interface StoredSession {
  id: string;
  label: string;
  createdAt: string;
  snapshot: SessionSnapshot;
  timeline?: SessionTimeline;
  version: number;
  isBaseline?: boolean;
  baselineTag?: string;
}

const STORAGE_KEY = "cryprq_session_history_v1";
const MAX_SESSIONS = 50;

export function loadStoredSessions(): StoredSession[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (!stored) {
      return [];
    }
    const parsed = JSON.parse(stored) as StoredSession[];
    if (!Array.isArray(parsed)) {
      console.warn("[SessionHistory] Invalid stored data format, resetting");
      return [];
    }
    return parsed;
  } catch (err) {
    console.warn("[SessionHistory] Failed to load sessions from localStorage:", err);
    return [];
  }
}

export function saveStoredSessions(sessions: StoredSession[]): void {
  try {
    const serialized = JSON.stringify(sessions);
    localStorage.setItem(STORAGE_KEY, serialized);
  } catch (err) {
    if (err instanceof DOMException && err.name === "QuotaExceededError") {
      console.warn("[SessionHistory] localStorage quota exceeded, dropping oldest sessions");
      // Drop oldest sessions and retry
      const trimmed = sessions.slice(0, Math.floor(MAX_SESSIONS * 0.8));
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(trimmed));
      } catch (retryErr) {
        console.error("[SessionHistory] Failed to save even after trimming:", retryErr);
      }
    } else {
      console.error("[SessionHistory] Failed to save sessions:", err);
    }
  }
}

export function addStoredSession(
  session: Omit<StoredSession, "id" | "createdAt" | "version"> & { id?: string }
): StoredSession[] {
  const existing = loadStoredSessions();
  const newSession: StoredSession = {
    id: session.id || `${Date.now()}_${Math.random().toString(36).substring(2, 9)}`,
    label: session.label || `Session ${new Date().toLocaleString()}`,
    createdAt: new Date().toISOString(),
    snapshot: session.snapshot,
    timeline: session.timeline,
    version: 1,
  };
  const updated = [newSession, ...existing];
  // Enforce max limit
  if (updated.length > MAX_SESSIONS) {
    updated.splice(MAX_SESSIONS);
  }
  saveStoredSessions(updated);
  return updated;
}

export function updateSessionLabel(id: string, newLabel: string): StoredSession[] {
  const existing = loadStoredSessions();
  const updated = existing.map((s) => (s.id === id ? { ...s, label: newLabel } : s));
  saveStoredSessions(updated);
  return updated;
}

export function deleteStoredSession(id: string): StoredSession[] {
  const existing = loadStoredSessions();
  const updated = existing.filter((s) => s.id !== id);
  saveStoredSessions(updated);
  return updated;
}

export function isLocalStorageAvailable(): boolean {
  try {
    const test = "__localStorage_test__";
    localStorage.setItem(test, test);
    localStorage.removeItem(test);
    return true;
  } catch {
    return false;
  }
}

export function markSessionAsBaseline(id: string, tag?: string): StoredSession[] {
  const existing = loadStoredSessions();
  const updated = existing.map((s) =>
    s.id === id
      ? { ...s, isBaseline: true, baselineTag: tag || s.label || "baseline" }
      : s
  );
  saveStoredSessions(updated);
  return updated;
}

export function clearBaselineFlag(id: string): StoredSession[] {
  const existing = loadStoredSessions();
  const updated = existing.map((s) =>
    s.id === id ? { ...s, isBaseline: false, baselineTag: undefined } : s
  );
  saveStoredSessions(updated);
  return updated;
}

export function getBaselines(): StoredSession[] {
  const all = loadStoredSessions();
  return all.filter((s) => s.isBaseline === true);
}

