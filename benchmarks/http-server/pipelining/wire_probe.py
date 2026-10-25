"""Checked sequential/pipelined HTTP traffic for the private session fixture."""
import queue
import threading
import socket
import subprocess
import time
import traceback


def run_wire(command, cwd, env, driver=False):
    process = subprocess.Popen(command, cwd=cwd, env=env, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, text=True, bufsize=1)
    idle = []
    held = []
    output = []
    errors = []
    lines = queue.Queue()

    def drain(stream, target, notify=False):
        for line in stream:
            target.append(line)
            if notify:
                lines.put(line)
        if notify:
            lines.put(None)

    stdout_thread = threading.Thread(target=drain, args=(process.stdout, output, True))
    stderr_thread = threading.Thread(target=drain, args=(process.stderr, errors))
    stdout_thread.start()
    stderr_thread.start()
    try:
        deadline = time.monotonic() + 45
        while True:
            line = lines.get(timeout=max(0.01, deadline - time.monotonic()))
            if line is None:
                raise RuntimeError("fixture exited before READY: " + "".join(errors))
            if line.strip() == "READY":
                break
        port = int(lines.get(timeout=10))
        if driver:
            control_port = int(lines.get(timeout=10))
            for _ in range(20):
                idle.append(socket.create_connection(("127.0.0.1", port), timeout=10))
        with socket.create_connection(("127.0.0.1", port), timeout=10) as client:
            client.settimeout(10)
            pending = b""

            def response(expected, close):
                nonlocal pending
                while b"\r\n\r\n" not in pending:
                    block = client.recv(8192)
                    assert block, "EOF in response headers"
                    pending += block
                header, pending = pending.split(b"\r\n\r\n", 1)
                lines = header.split(b"\r\n")
                assert lines[0] == b"HTTP/1.1 200 OK", lines[0]
                fields = {}
                for line in lines[1:]:
                    name, value = line.split(b":", 1)
                    name = name.lower()
                    assert name not in fields, "duplicate response field"
                    fields[name] = value.strip()
                assert fields[b"content-length"] == str(len(expected)).encode()
                assert (fields.get(b"connection") == b"close") == close
                while len(pending) < len(expected):
                    block = client.recv(8192)
                    assert block, "EOF in response body"
                    pending += block
                assert pending[:len(expected)] == expected
                pending = pending[len(expected):]

            # Two coalesced requests, then a sequential request on the same socket.
            client.sendall(b"GET /first HTTP/1.1\r\nHost: localhost\r\n\r\n"
                           b"GET /second HTTP/1.1\r\nHost: localhost\r\nConnection: keep-alive\r\n\r\n")
            response(b"/first", False)
            if driver:
                for idle_client in idle:
                    idle_client.setblocking(False)
                    try:
                        received = idle_client.recv(1)
                    except BlockingIOError:
                        received = None
                    assert received is None, "idle connection closed before admission check"
                    idle_client.close()
                idle.clear()
            response(b"/second", False)
            client.sendall(b"GET /last HTTP/1.1\r\nHost: localhost\r\nConnection: keep-alive, ClOsE\r\n\r\n")
            response(b"/last", True)
            assert not pending and client.recv(1) == b"", "missing final close"
        # The same response checker is rebound to each new socket below.
        with socket.create_connection(("127.0.0.1", port), timeout=10) as client:
            client.settimeout(10)
            pending = b""
            client.sendall(b"POST /body HTTP/1.1\r\nHost: localhost\r\nContent-Length: 2\r\n\r\nAB"
                           b"GET /after HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            response(b"/body", False)
            response(b"/after", True)
            assert not pending and client.recv(1) == b""
        with socket.create_connection(("127.0.0.1", port), timeout=10) as client:
            client.settimeout(10)
            pending = b""
            client.sendall(b"POST /unread HTTP/1.1\r\nHost: localhost\r\nContent-Length: 2\r\n\r\nAB"
                           b"GET /must-not-run HTTP/1.1\r\nHost: localhost\r\n\r\n")
            response(b"/unread", True)
            assert not pending and client.recv(1) == b"", "unread body reused"
        if driver:
            for _ in range(4):
                held_client = socket.create_connection(("127.0.0.1", port), timeout=10)
                held_client.settimeout(10)
                held_client.sendall(b"GET /hold HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                held.append(held_client)
            assert lines.get(timeout=10).strip() == "HELD"
            with socket.create_connection(("127.0.0.1", control_port), timeout=10):
                pass
            for client in held:
                pending = b""
                response(b"/hold", True)
                assert not pending and client.recv(1) == b""
            for _ in range(20):
                idle.append(socket.create_connection(("127.0.0.1", port), timeout=10))
            # Acceptance of this later connection establishes that the idle batch
            # has passed the serial accept queue before shutdown begins.
            with socket.create_connection(("127.0.0.1", port), timeout=10) as client:
                client.settimeout(10)
                pending = b""
                client.sendall(b"GET /shutdown-ready HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                response(b"/shutdown-ready", True)
                assert not pending and client.recv(1) == b""
            with socket.create_connection(("127.0.0.1", control_port), timeout=10):
                pass
            for client in idle:
                client.settimeout(10)
                assert client.recv(1) == b"", "idle connection survives shutdown"
        process.wait(timeout=30)
        stdout_thread.join(timeout=5)
        stderr_thread.join(timeout=5)
        return subprocess.CompletedProcess(command, process.returncode,
                                           "".join(output), "".join(errors))
    except Exception:
        failure = traceback.format_exc()
        try:
            process.wait(timeout=1)
        except subprocess.TimeoutExpired:
            pass
        if process.poll() is None:
            process.kill()
            process.wait(timeout=5)
        stdout_thread.join(timeout=5)
        stderr_thread.join(timeout=5)
        return subprocess.CompletedProcess(command, 1, "".join(output), "".join(errors) + failure)
    finally:
        for client in idle + held:
            client.close()
        if process.poll() is None:
            process.kill()
            process.wait(timeout=5)
        stdout_thread.join(timeout=5)
        stderr_thread.join(timeout=5)
