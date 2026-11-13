// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

import { useCallback, useEffect, useRef, useState } from 'react';
import type { FormEvent, DragEvent } from 'react';
import './App.css';

type Pairing = { version: 1; key: string; address: string };
type Peer = { id: string; name: string; key: string; address: string };
type ReceivedFile = { id: string; name: string; size: number; sha256: string; verified: boolean; receivedAt: string };
type Transfer = { id: string; name: string; size: number; sha256: string; state: string; authenticated: boolean; error?: string };
type PortalState = {
  identity: { key: string; address: string; pairing: Pairing };
  receiver: { status: string; authenticatedPeer?: string; error?: string };
  peers: Peer[]; transfers: Transfer[]; files: ReceivedFile[];
  limits: { maxFileBytes: number };
};
type IconName = 'send' | 'receive' | 'file' | 'copy' | 'check' | 'arrow' | 'download' | 'plus' | 'device';
function Icon({ name, size = 20 }: { name: IconName; size?: number }) {
  const paths: Record<IconName, string> = {
    send: 'm22 2-7 20-4-9-9-4 20-7ZM22 2 11 13',
    receive: 'M12 3v12m-5-5 5 5 5-5M4 15v5h16v-5',
    file: 'M14 2H5v20h14V7l-5-5Zm0 0v6h5M8 13h8M8 17h5',
    copy: 'M8 8h13v13H8V8ZM16 8V3H3v13h5',
    check: 'm5 12 4 4L19 6',
    arrow: 'M4 12h16m-6-6 6 6-6 6',
    download: 'M12 3v12m-5-5 5 5 5-5M4 17v4h16v-4',
    plus: 'M12 5v14M5 12h14',
    device: 'M3 3h18v13H3V3Zm5 18h8m-4-5v5',
  };
  return <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d={paths[name]} /></svg>;
}
function bytes(size: number) {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  return `${(size / (1024 * 1024)).toFixed(1)} MB`;
}
function message(error: unknown) { return error instanceof Error ? error.message : 'The request could not be completed.'; }
function fingerprint(key: string) { return `${key.slice(0, 16)}…${key.slice(-12)}`; }

export default function App() {
  const [state, setState] = useState<PortalState | null>(null);
  const [connectionError, setConnectionError] = useState('');
  const [actionError, setActionError] = useState('');
  const [notice, setNotice] = useState('');
  const [name, setName] = useState('');
  const [pairingInput, setPairingInput] = useState('');
  const [confirmed, setConfirmed] = useState(false);
  const [selectedPeer, setSelectedPeer] = useState('');
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [working, setWorking] = useState(false);
  const [dragging, setDragging] = useState(false);
  const [downloading, setDownloading] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);
  const token = useRef('');
  const advanced = window.location.pathname === '/advanced';

  const request = useCallback(async (url: string, options: RequestInit = {}) => {
    const authorize = async () => {
      const response = await fetch('/api/session', { cache: 'no-store' });
      if (!response.ok) throw new Error('The local portal is unavailable.');
      token.current = (await response.json()).token;
    };
    if (!token.current) await authorize();
    const perform = () => fetch(url, { ...options, cache: 'no-store', headers: { ...options.headers, 'X-Portal-Token': token.current } });
    let response = await perform();
    if (response.status === 401) { await authorize(); response = await perform(); }
    if (!response.ok) {
      const body = await response.json().catch(() => ({ error: 'The local portal could not complete this request.' }));
      throw new Error(body.error || `Request failed (${response.status}).`);
    }
    return response;
  }, []);

  const refresh = useCallback(async () => {
    const response = await request('/api/state');
    setState(await response.json());
    setConnectionError('');
  }, [request]);

  useEffect(() => {
    let active = true;
    let timer: ReturnType<typeof setTimeout>;
    async function poll() {
      try {
        const response = await request('/api/state');
        const next: PortalState = await response.json();
        if (active) { setState(next); setConnectionError(''); }
      } catch (error) { if (active) setConnectionError(message(error)); }
      if (active) timer = setTimeout(poll, 650);
    }
    void poll();
    return () => { active = false; clearTimeout(timer); };
  }, [request]);

  async function pair(event: FormEvent) {
    event.preventDefault();
    setActionError(''); setNotice('');
    if (!confirmed) { setActionError('Compare the public key with the other device, then confirm it below.'); return; }
    setWorking(true);
    try {
      const pairing: unknown = JSON.parse(pairingInput);
      await request('/api/peers', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ name, pairing }) });
      setName(''); setPairingInput(''); setConfirmed(false);
      setNotice('Device pinned. Pair this device on the other portal too.');
      await refresh();
    } catch (error) { setActionError(error instanceof SyntaxError ? 'Paste the complete pairing code from the other device.' : message(error)); }
    finally { setWorking(false); }
  }
  async function removePeer(peer: Peer) {
    setWorking(true); setActionError(''); setNotice('');
    try { await request(`/api/peers/${peer.id}`, { method: 'DELETE' }); await refresh(); setNotice(`${peer.name} removed from trusted devices.`); }
    catch (error) { setActionError(message(error)); }
    finally { setWorking(false); }
  }
  function chooseFile(file?: File) {
    setActionError(''); setNotice('');
    if (!file) return;
    if (state && file.size > state.limits.maxFileBytes) { setSelectedFile(null); setActionError(`Choose a file up to ${bytes(state.limits.maxFileBytes)}.`); return; }
    setSelectedFile(file);
  }
  function dropFile(event: DragEvent) { event.preventDefault(); setDragging(false); chooseFile(event.dataTransfer.files[0]); }
  const destination = state?.peers.find(peer => peer.id === selectedPeer) || state?.peers[0];
  const sending = working || !!state?.transfers.some(transfer => transfer.state === 'sending');
  async function send(event: FormEvent) {
    event.preventDefault();
    if (!selectedFile || !destination) return;
    setWorking(true); setActionError(''); setNotice('');
    try {
      await request(`/api/send?peer=${encodeURIComponent(destination.id)}`, { method: 'POST', headers: { 'Content-Type': 'application/octet-stream', 'X-Filename': encodeURIComponent(selectedFile.name) }, body: selectedFile });
      setSelectedFile(null);
      if (inputRef.current) inputRef.current.value = '';
      await refresh();
    } catch (error) { setActionError(message(error)); }
    finally { setWorking(false); }
  }
  async function download(file: ReceivedFile) {
    setDownloading(file.id); setActionError(''); setNotice('');
    try {
      const response = await request(`/api/files/${file.id}`);
      const data = await response.arrayBuffer();
      const hash = Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', data)), byte => byte.toString(16).padStart(2, '0')).join('');
      if (data.byteLength !== file.size || hash !== file.sha256) throw new Error('Download verification failed. The file was not saved.');
      const url = URL.createObjectURL(new Blob([data], { type: 'application/octet-stream' }));
      const anchor = document.createElement('a'); anchor.href = url; anchor.download = file.name;
      document.body.append(anchor); anchor.click(); anchor.remove();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
      setNotice(`Download verified: ${file.name}`);
    } catch (error) { setActionError(message(error)); }
    finally { setDownloading(''); }
  }
  async function copyPairing() {
    if (!state) return;
    try { await navigator.clipboard.writeText(JSON.stringify(state.identity.pairing)); setNotice('Public pairing code copied.'); }
    catch { setNotice('Select and copy the pairing code below.'); }
  }
  const ready = !connectionError && state?.receiver.status === 'listening';
  const statusText = connectionError ? 'Portal unavailable' : !state ? 'Starting device' : ({ 'needs-pairing': 'Pair a device to receive', starting: 'Starting receiver', listening: 'Ready to receive', authenticated: 'Authenticated transfer', error: 'Receiver needs attention' }[state.receiver.status] || 'Checking receiver');

  return <div className="portal" data-testid="app-root">
    <header className="site-header">
      <a className="brand" href="/" aria-label="CrypRQ home"><img src="/icon_master_1024.png" alt="" /><span data-testid="app-title">CrypRQ</span><span className="preview-tag">Research preview</span></a>
      <nav aria-label="Main navigation"><a href="/" aria-current={!advanced ? 'page' : undefined}>Transfer</a><a href="/advanced" aria-current={advanced ? 'page' : undefined}>Diagnostics</a></nav>
    </header>
    <main>
      <div className="page-heading"><div><p className="eyebrow">YOUR FILES. YOUR DEVICES.</p><h1>{advanced ? 'Device diagnostics' : 'A direct connection.'}</h1><p>{advanced ? 'Public device details and transfer verification.' : 'Pair a trusted device. Send a file. Verify it arrived.'}</p></div><span className="local-badge"><span className="status-dot" />Local portal</span></div>
      {connectionError && <div className="feedback error" role="alert">{connectionError} Check that the local adapter is running.</div>}
      {actionError && <div className="feedback error" role="alert">{actionError}</div>}
      {notice && <div className="feedback" role="status">{notice}</div>}
      {advanced ? <div className="diagnostics card">
        <h2>Device and receiver</h2><p>Listening is a readiness state. Authentication is reported only after the CLI confirms the handshake.</p>
        <dl><dt>Receiver</dt><dd>{statusText}</dd><dt>Public identity</dt><dd><code>{state?.identity.key || 'Loading'}</code></dd><dt>Peer address</dt><dd><code>{state?.identity.address || 'Loading'}</code></dd><dt>Authenticated peer</dt><dd><code>{state?.receiver.authenticatedPeer || 'No current authenticated session'}</code></dd><dt>Maximum file</dt><dd>{state ? bytes(state.limits.maxFileBytes) : 'Loading'}</dd></dl>
        <h2>Transfer evidence</h2><p>The protocol has no delivery acknowledgement or retransmission. Verify the file on the receiving device.</p>
        {state?.transfers.map(transfer => <div className="evidence" key={transfer.id}><strong>{transfer.name}</strong><span>{transfer.state} · {bytes(transfer.size)}</span><code>{transfer.sha256}</code></div>)}
        {!state?.transfers.length && <p>No outgoing transfers in this session.</p>}
        <a className="text-link" href="/">Return to file transfer <Icon name="arrow" size={16} /></a>
      </div> : <div className="workspace">
        <aside className="device-column">
          <section className="card device-card" aria-labelledby="device-heading"><div className="section-heading"><span className="icon-tile"><Icon name="device" /></span><h2 id="device-heading">Your device</h2></div>
            <p className={`receiver-status ${ready ? 'ready' : ''}`} data-testid="receiver-status"><span className="status-dot" />{statusText}</p>
            <p className="field-caption">Public identity</p><code className="identity-short" title={state?.identity.key}>{state ? fingerprint(state.identity.key) : 'Loading identity…'}</code>
            <details className="identity-details"><summary>View full public key</summary><code>{state?.identity.key}</code></details>
            <div className="pairing-heading"><label htmlFor="pairing-code">Your pairing code</label><button className="icon-button" onClick={() => void copyPairing()} aria-label="Copy public pairing code" disabled={!state}><Icon name="copy" size={16} /></button></div>
            <textarea id="pairing-code" data-testid="pairing-bundle" readOnly value={state ? JSON.stringify(state.identity.pairing) : ''} rows={3} onFocus={event => event.target.select()} />
            <p className="help">Share this public code with the device you want to pair.</p>
          </section>
          <section className="card pairing-card"><h2>Trusted devices <span className="count">{state?.peers.length || 0}</span></h2>
            {state?.peers.map(peer => <div className="peer" key={peer.id}><div><strong>{peer.name}</strong><code title={peer.key}>{fingerprint(peer.key)}</code></div><button className="remove-peer" disabled={working} onClick={() => void removePeer(peer)} aria-label={`Remove ${peer.name}`}>Remove</button></div>)}
            <details className="pair-form" open={!state?.peers.length}><summary><Icon name="plus" size={16} />Pair device</summary><form onSubmit={event => void pair(event)}>
              <label htmlFor="peer-name">Device name</label><input id="peer-name" data-testid="pair-name" placeholder="e.g. Alex’s laptop" value={name} onChange={event => setName(event.target.value)} maxLength={64} required />
              <label htmlFor="peer-code">Other device’s pairing code</label><textarea id="peer-code" data-testid="pair-peer" placeholder="Paste its public pairing code" value={pairingInput} onChange={event => { setPairingInput(event.target.value); setConfirmed(false); }} rows={3} required />
              <label className="confirmation"><input type="checkbox" data-testid="pair-confirm" checked={confirmed} onChange={event => setConfirmed(event.target.checked)} /> <span>I compared the public key with the other device over a trusted channel.</span></label>
              <button className="button secondary" data-testid="pair-device" disabled={working || !state}>Pin trusted device <Icon name="check" size={16} /></button>
            </form></details>
          </section>
        </aside>
        <div className="transfer-column">
          <section className="card send-card" aria-labelledby="send-heading"><div className="section-heading"><span className="icon-tile accent"><Icon name="send" /></span><div><h2 id="send-heading">Send a file</h2><p>Directly to a device you trust.</p></div><span className="section-number">01</span></div>
            <form onSubmit={event => void send(event)}>
              <div className={`dropzone ${dragging ? 'dragging' : ''} ${selectedFile ? 'has-file' : ''}`} onDragOver={event => { event.preventDefault(); setDragging(true); }} onDragLeave={() => setDragging(false)} onDrop={dropFile}>
                <div className="drop-icon"><Icon name="file" size={28} /></div>
                <strong>{selectedFile ? selectedFile.name : 'Drop your file here'}</strong>
                <p>{selectedFile ? bytes(selectedFile.size) : 'or choose a file from your device'}</p>
                <label className="browse-button" htmlFor="send-file">{selectedFile ? 'Choose another file' : 'Browse files'}</label>
                <input ref={inputRef} className="file-input" id="send-file" data-testid="send-file" type="file" disabled={sending} onChange={event => chooseFile(event.target.files?.[0])} />
                <span className="size-limit">Up to {state ? bytes(state.limits.maxFileBytes) : '…'} per file</span>
              </div>
              <div className="send-actions"><div className="destination"><label htmlFor="destination">Send to</label><select id="destination" value={destination?.id || ''} onChange={event => setSelectedPeer(event.target.value)} disabled={!state?.peers.length || sending}>{!state?.peers.length && <option value="">Pair a device first</option>}{state?.peers.map(peer => <option key={peer.id} value={peer.id}>{peer.name}</option>)}</select></div><button className="button primary" data-testid="send-file-button" disabled={!selectedFile || !destination || sending || !!connectionError}>{sending ? 'Sending…' : 'Send file'}<Icon name="arrow" size={18} /></button></div>
            </form>
            {!!state?.transfers.length && <div className="transfer-list" aria-live="polite">{state.transfers.slice(0, 3).map(transfer => <div className={`transfer-item ${transfer.state === 'failed' ? 'failed' : ''}`} key={transfer.id}><Icon name={transfer.state === 'sent-awaiting-receiver' ? 'check' : 'send'} size={16} /><div><strong>{transfer.name}</strong><span>{transfer.state === 'sending' ? (transfer.authenticated ? 'Authenticated. Sending file…' : 'Establishing authenticated connection…') : transfer.state === 'sent-awaiting-receiver' ? 'Sent. Check the receiving device.' : transfer.error || 'Transfer failed. Try again.'}</span></div></div>)}</div>}
          </section>
          <section className="card receive-card" aria-labelledby="receive-heading"><div className="section-heading"><span className="icon-tile"><Icon name="receive" /></span><div><h2 id="receive-heading">Received files</h2><p>Complete files, verified before they appear here.</p></div><span className="count">{state?.files.length || 0}</span></div>
            {!state?.files.length ? <div className="empty-inbox"><span className="inbox-line" /><Icon name="receive" size={24} /><span className="inbox-line" /><h3>Your inbox is ready</h3><p>Files from paired devices will appear here.</p></div> : <div className="received-list">{state.files.map(file => <article className="received-file" data-testid="received-file" key={file.id}><div className="file-tile"><Icon name="file" /></div><div className="file-information"><h3>{file.name}</h3><p>{bytes(file.size)} <span className="verified"><Icon name="check" size={13} />SHA-256 verified</span></p><details><summary>View SHA-256</summary><code>{file.sha256}</code></details></div><button className="download-button" aria-label={`Download ${file.name}`} disabled={!!downloading || !!connectionError} onClick={() => void download(file)}><Icon name="download" size={17} /><span>{downloading === file.id ? 'Verifying…' : 'Download'}</span></button></article>)}</div>}
          </section>
          <p className="transfer-note">Delivery is confirmed on the receiving device. Compare the downloaded file with the original.</p>
        </div>
      </div>}
    </main>
    <footer><span>CrypRQ <span className="footer-divider">/</span> Explicit trust. Verified files.</span><span>Research software · Keep this portal local.</span></footer>
  </div>;
}
