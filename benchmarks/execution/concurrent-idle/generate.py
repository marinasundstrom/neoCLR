"""Finish idle concurrent cycles without an immediate synchronous recheck."""
import argparse, subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
base=Path(__file__).resolve().parent
subprocess.run(['python3',base.parent/'concurrent-integration/generate.py','--output',a.output],check=True)
s=a.output.read_text()
s=s.replace('candidate_active, candidate_autostart;', 'candidate_active, candidate_autostart, candidate_idle_finish;')
s=s.replace('                if (status) return status;\n                candidate_counts[3]++;', '                if (status || candidate_idle_finish) return status;\n                candidate_counts[3]++;',1)
s=s.replace('    int32_t status = collect_requested(context, boundary, reason);', '    candidate_idle_finish = reason == NEOCLR_GC_TRIGGER_IDLE;\n    int32_t status = collect_requested(context, boundary, reason);\n    candidate_idle_finish = 0;',1)
a.output.write_text(s)
