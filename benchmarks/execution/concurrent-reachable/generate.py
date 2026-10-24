"""Retain reachable closures instead of every new/changed block at remark."""
import argparse, subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
base=Path(__file__).resolve().parent
subprocess.run(['python3',base.parent/'concurrent-idle/generate.py','--output',a.output],check=True)
s=a.output.read_text()
s=s.replace('    uint64_t used, *offsets, *pending;', '''    uint64_t used, *offsets, *pending, arena_capacity;
    uint64_t *late_words;
    size_t late_count, late_capacity;''',1)
s=s.replace('    s->used = context->text.used;', '    s->used = context->text.used;\n    s->arena_capacity = context->text.capacity;',1)
s=s.replace('    size_t i = shadow_find(s, word);\n    if (i != SIZE_MAX', '''    size_t i = shadow_find(s, word);
    /* A conservative word in a reachable snapshot payload may name storage that
     * becomes allocated later. Preserve it as a candidate, including interior
     * addresses, without reading the live heap from the marker. */
    if (i == SIZE_MAX && word >= s->base && word - s->base < s->arena_capacity) {
        if (s->late_count == s->late_capacity) {
            size_t next = s->late_capacity ? s->late_capacity * 2 : 64;
            if (next < s->late_capacity || next > SIZE_MAX / sizeof(uint64_t)) abort();
            void *words = realloc(s->late_words, next * sizeof(uint64_t));
            if (!words) abort();
            s->late_words = words;
            s->late_capacity = next;
        }
        s->late_words[s->late_count++] = word;
    }
    if (i != SIZE_MAX''',1)
s=s.replace('    free(s->edges); free(s->pending); free(s->offsets);','    free(s->late_words);\n    free(s->edges); free(s->pending); free(s->offsets);')
old='''            int changed = !existed || memcmp(b + 1, old + 1, b->bytes);
            if (changed) {
                /* Conservatively retain all new/changed objects and their current
                 * closure. This also covers native writes and weak-root exposure. */
                mark(a, index, (uintptr_t)(b + 1), pending);
            } else if (s->marked[previous] && !(b->state & MARKED)) {
                /* Its old closure is already in the immutable snapshot marks.
                 * A root already on the current worklist is left queued. */
                b->state = ALLOCATED | MARKED | SCANNED;
            }'''
new='''            if (existed && s->marked[previous]) {
                if (memcmp(b + 1, old + 1, b->bytes)) {
                    /* Revisit changed members of the initial live graph. Current
                     * roots/edges discover new or initially unreachable objects. */
                    mark(a, index, (uintptr_t)(b + 1), pending);
                } else if (!(b->state & MARKED)) {
                    b->state = ALLOCATED | MARKED | SCANNED;
                }
            }'''
assert s.count(old)==1;s=s.replace(old,new,1)
needle='''        at += b->span;
    }
    return 0;
}

static int32_t collect_indexed'''
assert s.count(needle)==1
s=s.replace(needle,'''        at += b->span;
    }
    for (size_t i = 0; i < s->late_count; i++)
        mark(a, index, s->late_words[i], pending);
    return 0;
}

static int32_t collect_indexed''',1)
a.output.write_text(s)
