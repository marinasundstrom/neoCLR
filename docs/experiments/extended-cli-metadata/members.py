"""Synthesized operation references; descriptors never grant execution capabilities."""
from dataclasses import dataclass
import struct

from codec import FormatError
from references import resolve, validate_local

SECTION_KIND = 4
SCHEMA = 1
MAX_MEMBERS = 256
ROW = struct.Struct('<HBBH')
OPERATIONS = {'array_length': 1, 'tuple_element': 2, 'tuple_deconstruct': 3,
              'function_invoke': 4}
OPERATION_NAMES = {value: key for key, value in OPERATIONS.items()}


@dataclass(frozen=True)
class MemberRef:
    operation: str
    owner: int = 1
    element: int = 0


@dataclass(frozen=True)
class MemberDescriptor:
    identity: tuple
    parameters: tuple
    modes: tuple
    result: tuple
    no_result: bool


def validate_row(member):
    if not isinstance(member, MemberRef) or member.operation not in OPERATIONS:
        raise FormatError('unknown structural operation')
    # This bounded profile has one owner: the section-3 signature root.
    if type(member.owner) is not int or member.owner != 1:
        raise FormatError('structural member owner must be signature root 1')
    if type(member.element) is not int or not 0 <= member.element <= 255:
        raise FormatError('invalid structural element index')
    if member.operation != 'tuple_element' and member.element != 0:
        raise FormatError('unexpected structural operation operand')


def encode_members(members):
    members = tuple(members)
    if len(members) > MAX_MEMBERS:
        raise FormatError('too many structural members')
    for member in members:
        validate_row(member)
    if len(set(members)) != len(members):
        raise FormatError('duplicate structural member reference')
    data = bytearray(struct.pack('<H', len(members)))
    for member in members:
        data.extend(ROW.pack(member.owner, OPERATIONS[member.operation], 0, member.element))
    return bytes(data)


def decode_members(data):
    if len(data) < 2:
        raise FormatError('truncated structural member table')
    count, = struct.unpack_from('<H', data)
    if count > MAX_MEMBERS or len(data) != 2 + count * ROW.size:
        raise FormatError('invalid structural member table length/count')
    members = []
    for offset in range(2, len(data), ROW.size):
        owner, operation, flags, element = ROW.unpack_from(data, offset)
        if operation not in OPERATION_NAMES or flags != 0:
            raise FormatError('unknown structural operation or flags')
        members.append(MemberRef(OPERATION_NAMES[operation], owner, element))
    encode_members(members)
    return tuple(members)


def validate_shape(member, root):
    validate_row(member)
    permitted = {
        'array_length': ('array', 'array_ref'),
        'tuple_element': ('tuple',),
        'tuple_deconstruct': ('tuple',),
        'function_invoke': ('function',),
    }
    if root.kind not in permitted[member.operation]:
        raise FormatError('structural operation is incompatible with owner shape')
    if member.operation == 'tuple_element' and member.element >= len(root.children):
        raise FormatError('tuple element outside owner arity')


def read_members(sections, profile):
    matches = [section for section in sections
               if (section.kind, section.version) == (SECTION_KIND, SCHEMA)]
    if not matches:
        return None
    if len(matches) != 1 or not matches[0].required or profile is None:
        raise FormatError('structural members require a mandatory reference profile')
    root, context, bindings = profile
    validate_local(root, context, bindings)
    members = decode_members(matches[0].payload)
    for member in members:
        validate_shape(member, root)
    return members


def resolve_members(members, root, context, bindings, catalog):
    """Derive contracts from a resolved owner; no MethodDef rows or dispatch target."""
    members = tuple(members)
    encode_members(members)
    owner_key = resolve(root, context, bindings, catalog)
    descriptors = []
    for member in members:
        validate_shape(member, root)
        parameters, modes, no_result = (), (), False
        if member.operation == 'array_length':
            # Matches native ArrayLength's UIntPtr result, not a guessed Int32 getter.
            result = ('intrinsic', 'native_uint')
        elif member.operation == 'tuple_element':
            result = owner_key[1][member.element]
        elif member.operation == 'tuple_deconstruct':
            parameters = owner_key[1]
            modes = ('out',) * len(parameters)
            result = ('unit', (), (), False)
            no_result = True
        else:  # function_invoke
            parameters, result = owner_key[1][:-1], owner_key[1][-1]
            modes, no_result = owner_key[2], owner_key[3]
        identity = ('structural_member', SCHEMA, owner_key, member.operation, member.element)
        descriptors.append(MemberDescriptor(identity, parameters, modes, result, no_result))
    return tuple(descriptors)
