"""Explicit development JIT host policy, independent of a reused link recipe."""


def ownership_command(command, owned, windows=False):
    if owned and windows:
        raise ValueError('Owned JIT heaps are currently qualified only on macOS ARM64')
    name = 'NEOCLR_GC_OWNED_HEAP'
    result = []
    index = 0
    while index < len(command):
        arg = str(command[index])
        if arg in ('-D', '/D', '-U', '/U') and index + 1 < len(command):
            definition = str(command[index + 1])
            if definition == name or definition.startswith(name + '='):
                index += 2
                continue
        if any(arg == prefix + name or arg.startswith(prefix + name + '=')
               for prefix in ('-D', '/D', '-U', '/U')):
            index += 1
            continue
        result.append(arg)
        index += 1
    if owned:
        result.insert(1, '-D' + name)
    return result
