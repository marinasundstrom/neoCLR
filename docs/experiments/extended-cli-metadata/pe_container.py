"""Bounded PE32 test-image editor; not a production loader, linker or signer."""
import struct
from codec import FormatError

MAX_IMAGE = 4 * 1024 * 1024


def align(value, alignment):
    return (value + alignment - 1) & ~(alignment - 1)


def take(data, offset, size):
    if offset < 0 or size < 0 or offset > len(data) or size > len(data) - offset:
        raise FormatError('truncated container range')
    return data[offset:offset + size]


def u16(data, offset):
    return struct.unpack('<H', take(data, offset, 2))[0]


def u32(data, offset):
    return struct.unpack('<I', take(data, offset, 4))[0]


def layout(data):
    if len(data) > MAX_IMAGE or take(data, 0, 2) != b'MZ':
        raise FormatError('unsupported PE image size or DOS signature')
    pe = u32(data, 0x3c)
    if take(data, pe, 4) != b'PE\0\0':
        raise FormatError('invalid PE signature')
    count, optional_size = u16(data, pe + 6), u16(data, pe + 20)
    optional = pe + 24
    if not 1 <= count <= 16 or optional_size != 224 or u16(data, optional) != 0x10b:
        raise FormatError('probe supports bounded PE32 only')
    if u32(data, optional + 92) != 16:
        raise FormatError('unsupported PE directory count')
    directories = optional + 96
    if any(take(data, directories + 4 * 8, 8)):
        raise FormatError('signed images are unsupported')
    if u32(data, optional + 64) != 0:
        raise FormatError('checksummed images are unsupported')
    section_alignment, file_alignment = u32(data, optional + 32), u32(data, optional + 36)
    if (file_alignment < 512 or file_alignment > 65536 or file_alignment & (file_alignment - 1)
            or section_alignment < file_alignment or section_alignment > 65536
            or section_alignment & (section_alignment - 1)):
        raise FormatError('unsupported PE alignment')
    table = optional + optional_size
    headers_size = u32(data, optional + 60)
    if headers_size > len(data) or table + count * 40 > headers_size:
        raise FormatError('invalid section table range')
    sections = []
    for i in range(count):
        offset = table + i * 40
        virtual_size, rva, size, raw = struct.unpack('<IIII', take(data, offset + 8, 16))
        if (raw < headers_size or raw % file_alignment or size % file_alignment
                or rva % section_alignment or rva < align(headers_size, section_alignment)):
            raise FormatError('invalid section alignment/range')
        take(data, raw, size)
        sections.append((rva, virtual_size, raw, size))
    for i, first in enumerate(sections):
        for second in sections[i + 1:]:
            for a, n, b, m in ((first[0], max(first[1], first[3]), second[0], max(second[1], second[3])),
                                (first[2], first[3], second[2], second[3])):
                if max(a, b) < min(a + n, b + m):
                    raise FormatError('overlapping PE sections')
    if max(raw + size for _, _, raw, size in sections) != len(data):
        raise FormatError('PE overlays are unsupported')

    def map_rva(rva, size):
        for start, _, raw, length in sections:
            if start <= rva and rva - start <= length and size <= length - (rva - start):
                return raw + rva - start
        raise FormatError('RVA outside file-backed sections')

    cli_rva, cli_size = struct.unpack('<II', take(data, directories + 14 * 8, 8))
    if cli_size != 72:
        raise FormatError('unsupported CLI header size')
    cli = map_rva(cli_rva, cli_size)
    if u32(data, cli) != 72 or u32(data, cli + 16) != 1:
        raise FormatError('only unsigned IL-only fixture images are supported')
    if any(take(data, cli + 32, 8)) or any(take(data, cli + 64, 8)):
        raise FormatError('strong-name/native images are unsupported')
    metadata_rva, metadata_size = u32(data, cli + 8), u32(data, cli + 12)
    metadata_offset = map_rva(metadata_rva, metadata_size)
    return dict(pe=pe, optional=optional, table=table, count=count, sections=sections,
                cli=cli, metadata=take(data, metadata_offset, metadata_size),
                metadata_offset=metadata_offset, headers_size=headers_size,
                file_alignment=file_alignment, section_alignment=section_alignment)


def streams(metadata):
    if take(metadata, 0, 4) != b'BSJB':
        raise FormatError('invalid metadata signature')
    version_length = u32(metadata, 12)
    if version_length > 256 or version_length % 4:
        raise FormatError('unsupported metadata version length')
    position = 16 + version_length
    take(metadata, position, 4)
    count = u16(metadata, position + 2)
    if count > 16:
        raise FormatError('too many metadata streams')
    prefix = bytes(metadata[:position + 2])  # includes storage flags, excludes stream count
    position += 4
    entries = []
    for _ in range(count):
        offset, size = struct.unpack('<II', take(metadata, position, 8))
        position += 8
        name_start = position
        name = bytearray()
        for _ in range(32):
            byte = take(metadata, position, 1)[0]
            position += 1
            if byte == 0:
                break
            if byte > 127:
                raise FormatError('non-ASCII metadata stream name')
            name.append(byte)
        else:
            raise FormatError('unterminated metadata stream name')
        padded_end = name_start + align(position - name_start, 4)
        if any(take(metadata, position, padded_end - position)):
            raise FormatError('invalid stream-name padding')
        position = padded_end
        name = bytes(name).decode('ascii')
        if not name or any(entry[0] == name for entry in entries):
            raise FormatError('empty or duplicate stream name')
        entries.append((name, offset, size))
    result = {}
    for name, offset, size in entries:
        if offset < position or offset % 4:
            raise FormatError('invalid metadata stream offset')
        result[name] = bytes(take(metadata, offset, size))
    for i, (_, a, n) in enumerate(entries):
        for _, b, m in entries[i + 1:]:
            if max(a, b) < min(a + n, b + m):
                raise FormatError('overlapping metadata streams')
    return prefix, result


def build_metadata(prefix, values):
    offset = len(prefix) + 2 + sum(8 + align(len(name) + 1, 4) for name in values)
    directory = bytearray(prefix + struct.pack('<H', len(values)))
    payload = bytearray()
    for name, value in values.items():
        encoded_name = name.encode('ascii') + b'\0'
        directory.extend(struct.pack('<II', offset, len(value)))
        directory.extend(encoded_name + b'\0' * (align(len(encoded_name), 4) - len(encoded_name)))
        payload.extend(value)
        padding = align(len(value), 4) - len(value)
        payload.extend(b'\0' * padding)
        offset += len(value) + padding
    return bytes(directory + payload)


def embed(image, neox):
    # Validate payload separately; this function transports bytes, including negative fixtures.
    if len(neox) > 1024 * 1024:
        raise FormatError('extension too large')
    info = layout(image)
    prefix, values = streams(info['metadata'])
    if '#Neo' in values:
        raise FormatError('image already contains #Neo')
    values['#Neo'] = neox + b'\0' * (align(len(neox), 4) - len(neox))
    metadata = build_metadata(prefix, values)
    slot = info['table'] + info['count'] * 40
    if slot + 40 > info['headers_size'] or any(take(image, slot, 40)):
        raise FormatError('no unused PE section-header slot')
    raw = align(len(image), info['file_alignment'])
    raw_size = align(len(metadata), info['file_alignment'])
    rva = align(max(r + max(v, s) for r, v, _, s in info['sections']), info['section_alignment'])
    if raw + raw_size > MAX_IMAGE or rva + raw_size > 0xffffffff:
        raise FormatError('embedded image too large')
    output = bytearray(image)
    output.extend(b'\0' * (raw - len(output)))
    output.extend(metadata)
    output.extend(b'\0' * (raw_size - len(metadata)))
    output[slot:slot + 40] = struct.pack('<8sIIIIIIHHI', b'.neometa', len(metadata), rva,
                                       raw_size, raw, 0, 0, 0, 0, 0x40000040)
    struct.pack_into('<H', output, info['pe'] + 6, info['count'] + 1)
    struct.pack_into('<I', output, info['optional'] + 8, u32(image, info['optional'] + 8) + raw_size)
    struct.pack_into('<I', output, info['optional'] + 56, align(rva + len(metadata), info['section_alignment']))
    struct.pack_into('<II', output, info['cli'] + 8, rva, len(metadata))
    result = bytes(output)
    layout(result)
    return result


def extract(image):
    _, values = streams(layout(image)['metadata'])
    if '#Neo' not in values:
        raise FormatError('missing #Neo stream')
    payload = values['#Neo']
    size = u32(payload, 12)
    if size < 16 or size > len(payload) or len(payload) - size > 3 or any(payload[size:]):
        raise FormatError('invalid #Neo padding/length')
    return payload[:size]
