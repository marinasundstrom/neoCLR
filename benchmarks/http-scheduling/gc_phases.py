"""Inject exact phase hooks into diagnostic copies; fail closed on source drift."""
from pathlib import Path

def replace_once(source, old, new):
    if source.count(old) != 1:
        raise ValueError('GC profiling hook moved: '+old)
    return source.replace(old, new)

def instrument(root, out, command):
    collector = root/'docs/experiments/aot-console/native-gc.c'
    probe = root/'docs/experiments/aot-console/root-probe.c'
    assert str(collector) in command and str(probe) in command
    text = (root/'runtime/native/gc/native-gc.c').read_text()
    text = '#include <stdint.h>\nextern void neoclr_gc_phase_begin(uint64_t);\nextern void neoclr_gc_phase_next(unsigned);\nextern void neoclr_gc_phase_end(int32_t);\n' + text
    hook = '    int32_t status = collect_conditional(context, head, conditional, count);'
    text = replace_once(text, hook, '    neoclr_gc_phase_begin(statistics.allocations);\n'+hook+'\n    neoclr_gc_phase_end(status);')
    for phase, marker in enumerate(['    neoclr_probe_storage fault[65];',
        '    /* Each newly marked allocation enters the intrusive worklist exactly once.',
        '    for (neoclr_gc_resource **link = &resource_head; *link;) {',
        '    /* Sweep and coalesce in one forward walk.'], 1):
        text = replace_once(text, marker, f'    neoclr_gc_phase_next({phase});\n'+marker)
    gc_copy = out/'gc-phases-collector.c'
    gc_copy.write_text(text)
    text = (root/'runtime/native/gc/root-probe.c').read_text()
    text = '#include <stdint.h>\nextern uint64_t neoclr_schedule_profile_now(void);\nextern void neoclr_gc_boundary_end(uint64_t);\n' + text
    marker = '    if (!frame || head != frame || instruction >= 8192'
    text = replace_once(text, marker, '    uint64_t phase_start = neoclr_schedule_profile_now();\n'+marker)
    marker = '    /* Complete snapshot: audited allocating helpers additionally protect their temporaries. */'
    text = replace_once(text, marker, '    neoclr_gc_boundary_end(phase_start);\n'+marker)
    probe_copy = out/'gc-phases-probe.c'
    probe_copy.write_text(text)
    command = [str(gc_copy) if v == str(collector) else str(probe_copy) if v == str(probe) else v for v in command]
    support = Path(__file__).with_name('gc-phases.c').resolve()
    command.append(str(support))
    return command, [*sorted((root/'runtime/native').rglob('*.[ch]')), gc_copy, probe_copy, support, Path(__file__).resolve()]
