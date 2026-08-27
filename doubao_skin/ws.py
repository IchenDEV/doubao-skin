"""Minimal WebSocket client + Chrome DevTools Protocol caller.

Stdlib-only. Client frames are masked per RFC 6455; no Origin header is sent
(Chromium rejects CDP handshakes that carry an unknown Origin).
"""
import base64
import json
import os
import socket
import struct


class WebSocket:
    def __init__(self, url: str, timeout: float = 10.0):
        assert url.startswith("ws://"), url
        rest = url[len("ws://"):]
        hostport, _, path = rest.partition("/")
        host, _, port = hostport.partition(":")
        self.sock = socket.create_connection((host, int(port or 80)), timeout=timeout)
        key = base64.b64encode(os.urandom(16)).decode()
        self.sock.sendall((
            f"GET /{path} HTTP/1.1\r\nHost: {hostport}\r\n"
            f"Upgrade: websocket\r\nConnection: Upgrade\r\n"
            f"Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n"
        ).encode())
        head, self.buf = self._read_head()
        if b" 101 " not in head.split(b"\r\n", 1)[0]:
            raise ConnectionError(f"handshake failed: {head[:200]!r}")

    def _read_head(self):
        data = b""
        while b"\r\n\r\n" not in data:
            chunk = self.sock.recv(4096)
            if not chunk:
                raise ConnectionError("closed during handshake")
            data += chunk
        head, _, rest = data.partition(b"\r\n\r\n")
        return head, rest

    def _read(self, n: int) -> bytes:
        while len(self.buf) < n:
            chunk = self.sock.recv(65536)
            if not chunk:
                raise ConnectionError("connection closed")
            self.buf += chunk
        out, self.buf = self.buf[:n], self.buf[n:]
        return out

    def send_text(self, payload: str):
        data = payload.encode()
        n = len(data)
        head = bytearray([0x81])
        if n < 126:
            head.append(0x80 | n)
        elif n < 65536:
            head += bytes([0x80 | 126]) + struct.pack(">H", n)
        else:
            head += bytes([0x80 | 127]) + struct.pack(">Q", n)
        mask = os.urandom(4)
        head += mask
        self.sock.sendall(bytes(head) + bytes(b ^ mask[i % 4] for i, b in enumerate(data)))

    def recv_text(self, timeout: float = 10.0) -> str:
        self.sock.settimeout(timeout)
        while True:
            opcode, payload = self._frame()
            if opcode == 0x1:
                return payload.decode()
            if opcode == 0x9:  # ping -> pong
                self._send_control(0xA, payload)
            elif opcode == 0x8:
                raise ConnectionError("closed by peer")

    def _send_control(self, opcode: int, payload: bytes):
        mask = os.urandom(4)
        head = bytes([0x80 | opcode, 0x80 | len(payload)]) + mask
        self.sock.sendall(head + bytes(b ^ mask[i % 4] for i, b in enumerate(payload)))

    def _frame(self):
        payload = b""
        while True:
            h = self._read(2)
            fin, opcode = h[0] & 0x80, h[0] & 0x0F
            ln = h[1] & 0x7F
            if ln == 126:
                ln = struct.unpack(">H", self._read(2))[0]
            elif ln == 127:
                ln = struct.unpack(">Q", self._read(8))[0]
            payload += self._read(ln)
            if fin:
                return opcode, payload

    def close(self):
        try:
            self.sock.close()
        except OSError:
            pass


class CDP:
    """Synchronous DevTools-protocol caller (events are ignored)."""

    def __init__(self, ws_url: str, timeout: float = 10.0):
        self.ws = WebSocket(ws_url, timeout)
        self._id = 0

    def call(self, method: str, params: dict | None = None):
        self._id += 1
        self.ws.send_text(json.dumps({"id": self._id, "method": method,
                                      "params": params or {}}))
        while True:
            msg = json.loads(self.ws.recv_text())
            if msg.get("id") == self._id:
                if "error" in msg:
                    raise RuntimeError(f"{method}: {msg['error']}")
                return msg.get("result", {})

    def evaluate(self, expression: str):
        result = self.call("Runtime.evaluate", {
            "expression": expression, "returnByValue": True,
        })
        return result.get("result", {}).get("value")

    def close(self):
        self.ws.close()
