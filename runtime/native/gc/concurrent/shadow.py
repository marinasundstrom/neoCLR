"""Instrument a copy of the collector with immutable concurrent shadow tracing."""
from pathlib import Path
import argparse

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
source = (Path(__file__).resolve().parent.parent / 'native-gc.c').read_text()
start = source.index('            if (b->kind == NEOCLR_GC_INTERN) {', source.index('while (pending)'))
end = source.index('\n        }\n        // Values can reveal', start)
scanner = source[start:end]
scanner = scanner.replace('mark(a, &index, ', 'shadow_mark(s, ').replace(', &pending)', ')')
scanner = scanner.replace('LOOKUP_COUNT(object_words);', '').replace('goto invalid_descriptor;', 'return 0;')
assert 'mark(a,' not in scanner
helper = (Path(__file__).parent / 'shadow.h').read_text()
helper += '\nstatic int shadow_scan(shadow_snapshot *s, const block *b) {\n    const unsigned char *data = (const void *)(b + 1);\n' + scanner + '\n    return 1;\n}\n'
source = source.replace('static int32_t collect_indexed(', helper + '\nstatic int32_t collect_indexed(', 1)
source = source.replace('    /* Each newly marked reference-bearing allocation enters the worklist once.', '    shadow_begin(context, conditional, conditional_count);\n    /* Each newly marked reference-bearing allocation enters the worklist once.', 1)
needle = '    for (neoclr_gc_resource **link = &resource_head; *link;) {\n        neoclr_gc_resource *r = *link;'
assert source.count(needle) == 1
source = source.replace(needle, '    shadow_reference(context);\n' + needle, 1)
source = 'static void shadow_stop(void);\n' + source
source = source.replace('int32_t neoclr_gc_heap_release_v1(neoclr_aot_text_arena *arena) {', 'int32_t neoclr_gc_heap_release_v1(neoclr_aot_text_arena *arena) {\n    shadow_stop();', 1)
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(source)
