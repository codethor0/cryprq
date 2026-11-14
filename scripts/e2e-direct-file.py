#!/usr/bin/env python3
# Copyright (c) 2026 Thor Thor
# SPDX-License-Identifier: MIT
"""Two-process local direct-file regression test; this is not a WAN witness."""

import hashlib
import os
from pathlib import Path
import shutil
import socket
import subprocess
import sys
import tempfile
import time


def main():
    binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/cryprq").resolve()
    if not binary.is_file():
        raise RuntimeError(f"build the CLI first: {binary}")
    work = Path(tempfile.mkdtemp(prefix="cryprq-direct-e2e-"))
    env = dict(os.environ)
    env.pop("CRYPRQ_TEST_MODE", None)
    env["RUST_LOG"] = "info"
    receiver = None
    passed = False
    checks = 0

    def command(who, *args):
        return [str(binary), "--identity", str(work / who / "identity.key"),
                "--trusted-peers", str(work / who / "trusted_peers"), *args]

    def run(who, *args, ok=True, label="command", extra_env=None):
        result = subprocess.run(command(who, *args), env={**env, **(extra_env or {})},
                                capture_output=True, text=True, timeout=40)
        (work / f"{label}.log").write_text(result.stdout + result.stderr)
        if (result.returncode == 0) != ok:
            raise AssertionError(f"{label}: unexpected exit {result.returncode}")
        return result.stdout

    def start(label, transport, expected="alice"):
        nonlocal receiver
        output = work / f"received-{label}"
        output.mkdir()
        kind = socket.SOCK_DGRAM if transport == "quic" else socket.SOCK_STREAM
        with socket.socket(socket.AF_INET, kind) as probe:
            probe.bind(("127.0.0.1", 0))
            port = probe.getsockname()[1]
        suffix = f"udp/{port}/quic-v1" if transport == "quic" else f"tcp/{port}"
        address = f"/ip4/127.0.0.1/{suffix}"
        log = work / f"{label}-receiver.log"
        with log.open("w") as sink:
            receiver = subprocess.Popen(command(
                "bob", "direct-receive-file", "--listen", address,
                "--peer-key", keys[expected], "--output-dir", str(output)),
                env=env, stdout=sink, stderr=subprocess.STDOUT)
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            if "event=direct_file_listening" in log.read_text():
                return output, address, log
            if receiver.poll() is not None:
                raise AssertionError(f"{label}: receiver exited before listening")
            time.sleep(0.05)
        raise AssertionError(f"{label}: receiver did not become ready")

    def stop():
        nonlocal receiver
        if receiver is not None:
            if receiver.poll() is None:
                receiver.terminate()
                try:
                    receiver.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    receiver.kill()
                    receiver.wait(timeout=5)
                    raise AssertionError("receiver did not stop on cancellation")
            receiver = None

    def check(description):
        nonlocal checks
        checks += 1
        print(f"PASS: {description}", flush=True)

    try:
        keys = {}
        for who in ("alice", "bob", "mallory"):
            (work / who).mkdir(mode=0o700)
            keys[who] = run(who, "identity", "init", label=f"{who}-init").strip()
        for who in keys:
            (work / who / "trusted_peers").write_text("".join(
                f"{key} {peer}\n" for peer, key in keys.items() if peer != who))
        payload = work / "payload.bin"
        payload.write_bytes(bytes(range(256)) * 1025)
        empty = work / "empty.bin"
        empty.write_bytes(b"")

        for transport in ("quic", "tcp"):
            for source in (payload, empty):
                label = f"{transport}-{source.stem}"
                output, address, log = start(label, transport)
                sent = run("alice", "direct-send-file", "--peer", address,
                           "--peer-key", keys["bob"], "--file", str(source), label=label)
                receiver.wait(timeout=15)
                if receiver.returncode != 0:
                    raise AssertionError(f"{label}: receiver failed")
                received = output / source.name
                digest = hashlib.sha256(source.read_bytes()).hexdigest()
                assert received.read_bytes() == source.read_bytes(), label
                assert digest in sent and digest in log.read_text(), label
                assert "event=direct_file_authenticated role=initiator" in (work / f"{label}.log").read_text()
                assert "event=direct_file_authenticated role=responder" in log.read_text()
                assert source.name not in log.read_text(), "receiver leaked source filename"
                assert source.name not in (work / f"{label}.log").read_text(), "sender leaked source filename"
                assert sorted(p.name for p in output.iterdir()) == [source.name], label
                check(f"{label}: authenticated delivery, encrypted receipt, SHA-256 {digest}")
                stop()

        for label, sender, remote_key in (
                ("wrong-sender", "mallory", keys["bob"]),
                ("wrong-receiver", "alice", keys["mallory"])):
            output, address, _ = start(label, "quic")
            run(sender, "direct-send-file", "--peer", address, "--peer-key", remote_key,
                "--file", str(payload), ok=False, label=label)
            stop()
            assert not list(output.iterdir()), label
            check(f"{label}: rejected with no received file")

        for label in ("duplicate", "symlink"):
            output, address, _ = start(label, "quic")
            sentinel = work / f"{label}-sentinel"
            sentinel.write_bytes(b"must survive unchanged")
            if label == "symlink":
                (output / payload.name).symlink_to(sentinel)
            else:
                shutil.copyfile(sentinel, output / payload.name)
            run("alice", "direct-send-file", "--peer", address, "--peer-key", keys["bob"],
                "--file", str(payload), ok=False, label=label)
            stop()
            assert sentinel.read_bytes() == b"must survive unchanged", label
            assert (output / payload.name).read_bytes() == b"must survive unchanged", label
            assert sorted(p.name for p in output.iterdir()) == [payload.name], label
            check(f"{label}: no overwrite and no partial files")

        # Sender-side rejections happen before any dial; the subprocess timeout
        # turns a blocking open (for example on a FIFO) into a failure.
        unused = "/ip4/127.0.0.1/tcp/9"
        fifo = work / "source.fifo"
        os.mkfifo(fifo)
        link = work / "source-link.bin"
        link.symlink_to(payload)
        for label, source in (("source-directory", work), ("source-symlink", link),
                              ("source-fifo", fifo)):
            out = run("alice", "direct-send-file", "--peer", unused, "--peer-key", keys["bob"],
                      "--file", str(source), ok=False, label=label)
            assert "route=direct" not in out, label
            check(f"{label}: non-regular source rejected without blocking")

        for label, flags, extra in (("test-mode-flag", ("--test-mode",), None),
                                    ("test-mode-env", (), {"CRYPRQ_TEST_MODE": "1"})):
            run("alice", *flags, "direct-send-file", "--peer", unused, "--peer-key",
                keys["bob"], "--file", str(payload), ok=False, label=f"{label}-send",
                extra_env=extra)
            output = work / f"{label}-output"
            output.mkdir()
            run("bob", *flags, "direct-receive-file", "--listen", unused, "--peer-key",
                keys["alice"], "--output-dir", str(output), ok=False,
                label=f"{label}-receive", extra_env=extra)
            for side in ("send", "receive"):
                assert "requires authenticated mode" in (work / f"{label}-{side}.log").read_text(), label
            assert not list(output.iterdir()), label
            check(f"{label}: insecure test mode refused for send and receive")

        passed = True
        print(f"DIRECT FILE E2E: PASSED ({checks} checks); route=local; WAN TEST NOT VERIFIED")
    finally:
        stop()
        if passed and os.environ.get("CRYPRQ_E2E_KEEP_LOGS") != "1":
            shutil.rmtree(work)
        else:
            print(f"Direct file E2E evidence: {work}", flush=True)


if __name__ == "__main__":
    main()
