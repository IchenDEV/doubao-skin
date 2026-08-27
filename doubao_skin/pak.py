"""Minimal Chromium data-pack (.pak v5) reader/writer.

Layout (little-endian):
    header:  u32 version(=5) | u32 encoding | u16 num_entries | u16 num_aliases
    index:   (num_entries + 1) x { u16 resource_id | u32 offset }
             (the last record is a sentinel with id 0 whose offset is EOF)
    aliases: num_aliases x { u16 resource_id | u16 index }
    payload: raw resource bytes (gzip-compressed for most text resources)
"""
import struct


def parse(path):
    data = open(path, "rb").read()
    version, encoding, num_entries, num_aliases = struct.unpack_from("<IIHH", data, 0)
    if version != 5:
        raise ValueError(f"unsupported pak version: {version}")
    entries, off = [], 12
    for _ in range(num_entries + 1):
        rid, o = struct.unpack_from("<HI", data, off)
        entries.append((rid, o))
        off += 6
    aliases = []
    for _ in range(num_aliases):
        rid, idx = struct.unpack_from("<HH", data, off)
        aliases.append((rid, idx))
        off += 4
    return data, entries, aliases


def iter_blobs(data, entries):
    """Yield (resource_id, payload_bytes) for every entry in file order."""
    for i in range(len(entries) - 1):
        rid, start = entries[i]
        yield rid, data[start:entries[i + 1][1]]


def build(blobs, aliases, encoding=1):
    """Rebuild a pak from [(id, bytes), ...] (file order) and the alias table."""
    num_entries, num_aliases = len(blobs), len(aliases)
    header = struct.pack("<IIHH", 5, encoding, num_entries, num_aliases)
    offset = 12 + (num_entries + 1) * 6 + num_aliases * 4
    index, body = b"", b""
    for rid, blob in blobs:
        index += struct.pack("<HI", rid, offset)
        body += blob
        offset += len(blob)
    index += struct.pack("<HI", 0, offset)  # sentinel
    alias_blob = b"".join(struct.pack("<HH", rid, idx) for rid, idx in aliases)
    return header + index + alias_blob + body
