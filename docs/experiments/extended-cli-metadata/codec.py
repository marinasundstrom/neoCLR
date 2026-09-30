"""Standalone experimental metadata framing; not a PE/CLI reader or runtime loader."""
import argparse
from dataclasses import asdict, dataclass
import json
from pathlib import Path
import struct
import sys

MAGIC = b"NEOX"
HEADER = struct.Struct("<4sHHII")
ENTRY = struct.Struct("<HHIII")
MAX_SIZE = 1024 * 1024
MAX_SECTIONS = 64
REQUIRED = 1


class FormatError(ValueError):
    pass


@dataclass(frozen=True)
class Section:
    kind: int
    version: int
    required: bool
    payload: bytes


def decode(data, supported=None):
    """Validate framing and required schemas; preserve optional opaque sections."""
    supported = {} if supported is None else supported
    if len(data) > MAX_SIZE or len(data) < HEADER.size:
        raise FormatError("invalid image size")
    magic, major, minor, count, total = HEADER.unpack_from(data)
    if magic != MAGIC or (major, minor) != (0, 1):
        raise FormatError("unsupported envelope version or magic")
    if total != len(data) or count > MAX_SECTIONS:
        raise FormatError("invalid size or section count")
    end = HEADER.size + count * ENTRY.size
    if end > len(data):
        raise FormatError("truncated directory")
    sections = []
    seen = set()
    for i in range(count):
        kind, version, flags, offset, length = ENTRY.unpack_from(data, HEADER.size + i * ENTRY.size)
        if kind == 0 or version == 0 or kind in seen:
            raise FormatError("invalid or duplicate section kind/version")
        if flags & ~REQUIRED:
            raise FormatError("unknown section flags")
        # Canonical contiguous order makes overlaps, gaps, hidden data and wrapping invalid.
        if offset != end or length > len(data) - offset:
            raise FormatError("invalid section range")
        end = offset + length
        seen.add(kind)
        if flags & REQUIRED and supported.get(kind) != version:
            raise FormatError(f"unsupported required section {kind} schema {version}")
        sections.append(Section(kind, version, bool(flags & REQUIRED), data[offset:end]))
    if end != len(data):
        raise FormatError("trailing bytes")
    return sections


def encode(sections, supported=None):
    sections = list(sections)
    if len(sections) > MAX_SECTIONS:
        raise FormatError("too many sections")
    offset = HEADER.size + len(sections) * ENTRY.size
    total = offset + sum(len(s.payload) for s in sections)
    if total > MAX_SIZE:
        raise FormatError("image too large")
    data = bytearray(HEADER.pack(MAGIC, 0, 1, len(sections), total))
    for section in sections:
        if not (1 <= section.kind <= 65535 and 1 <= section.version <= 65535):
            raise FormatError("section kind/version outside u16 range")
        if type(section.required) is not bool:
            raise FormatError("required must be boolean")
        data.extend(ENTRY.pack(section.kind, section.version, int(section.required), offset, len(section.payload)))
        offset += len(section.payload)
    for section in sections:
        data.extend(section.payload)
    result = bytes(data)
    decode(result, supported)
    return result


def read_image(path):
    # Limit reads before allocating arbitrary input sizes.
    with path.open("rb") as stream:
        data = stream.read(MAX_SIZE + 1)
    if len(data) > MAX_SIZE:
        raise FormatError("image too large")
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("image", type=Path)
    args = parser.parse_args()
    try:
        from signatures import SECTION_KIND, SCHEMA, decode_signature
        sections = decode(read_image(args.image), {SECTION_KIND: SCHEMA})
        inspected = []
        for section in sections:
            item = {"kind": section.kind, "version": section.version,
                    "required": section.required, "payload_hex": section.payload.hex()}
            if (section.kind, section.version) == (SECTION_KIND, SCHEMA):
                root, context = decode_signature(section.payload)
                item.update(signature=asdict(root), context=asdict(context))
            inspected.append(item)
        print(json.dumps({"profile": "NEOX 0.1", "executable": False, "sections": inspected}, indent=2))
    except (OSError, FormatError) as error:
        print(f"metadata: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    from codec import main as inspect_main
    sys.exit(inspect_main())
